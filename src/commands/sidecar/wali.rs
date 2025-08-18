use crate::commands::call::Host;
use crate::commands::lhw_get_cx;
use libc::{c_char, c_void, iovec};
use std::panic::{AssertUnwindSafe, catch_unwind};
use std::thread;
use wasmtime::{Caller, ExternType, Linker, Module, Store};

/* For startup environment */
static mut APP_ARGC: i32 = 0;
static mut APP_ARGV: [*const i8; 8] = [std::ptr::null(); 8];
static mut INVOKED_WALI: bool = false;

/* For process exit functionality */
static mut PROC_EXIT_PRIMARY_TID: i32 = -1;
static mut PROC_EXIT_INVOKED: bool = false;

/* WALI State */
const WASI_ENTRY_POINT: &str = "__wasm_thread_start_libc";

static mut CTOR_CALLED: bool = false;
static mut DTOR_CALLED: bool = false;

struct PKUIOVec {
    base: u32,
    len: u32,
}

impl PKUIOVec {
    fn get_base(&self) -> u32 {
        self.base
    }

    fn to_iovec(&self, iov_base: *mut c_void) -> iovec {
        iovec {
            iov_base,
            iov_len: self.len.try_into().unwrap(),
        }
    }
}

fn maddr(caller: &mut Caller<'_, Host>, arg: i32) -> i64 {
    if arg == 0 {
        return 0;
    }
    let extmem = caller.get_export("memory").unwrap();
    let memty = extmem.ty(&*caller);
    if let wasmtime::ExternType::Memory(ty) = memty {
        if ty.is_shared() {
            let memory = extmem.into_shared_memory().unwrap();
            let linear_memory = memory.data();
            unsafe {
                let value = linear_memory.as_ptr().add(arg as usize).cast() as *const c_void;
                return value as i64;
            }
        } else {
            let memory = extmem.into_memory().unwrap();
            let linear_memory = memory.data(&caller);
            unsafe {
                let value = linear_memory.as_ptr().add(arg as usize).cast() as *const c_void;
                return value as i64;
            }
        }
    } else {
        panic!("Expected a memory export, but found: {:?}", extmem);
    }
}

/* Copy iovec structure */
fn copy_iovec(caller: &mut Caller<'_, Host>, wasm_iov: i32, iov_cnt: i32) -> *mut iovec {
    if wasm_iov == 0 {
        return std::ptr::null_mut();
    }
    let cnt = iov_cnt as usize;
    unsafe {
        let ciovs = maddr(caller, wasm_iov) as *const PKUIOVec;
        let new_iov = libc::malloc(cnt * size_of::<iovec>()) as *mut iovec;
        for i in 0..cnt {
            let cbase = (*ciovs.wrapping_add(i)).get_base();
            let rbase = maddr(caller, cbase.try_into().unwrap()) as *mut c_void;
            *new_iov.wrapping_add(i) = (*ciovs.wrapping_add(i)).to_iovec(rbase);
        }
        new_iov
    }
}

// 0
fn wali_syscall_read(mut caller: Caller<'_, Host>, a1: i32, a2: i32, a3: i32) -> i64 {
    unsafe { libc::syscall(libc::SYS_read, a1, maddr(&mut caller, a2), a3) }
}

// 1
fn wali_syscall_write(mut caller: Caller<'_, Host>, a1: i32, a2: i32, a3: i32) -> i64 {
    unsafe { libc::syscall(libc::SYS_write, a1, maddr(&mut caller, a2), a3) }
}

// 2
fn wali_syscall_open(mut caller: Caller<'_, Host>, a1: i32, a2: i32, a3: i32) -> i64 {
    unsafe {
        let filename = maddr(&mut caller, a1) as *const c_char;
        let ret = libc::syscall(libc::SYS_open, filename, a2, a3);
        if ret < 0 {
            println!(
                "wali_syscall_open error: {:?}",
                std::io::Error::last_os_error()
            );
        }
        ret.into()
    }
}

// 3
fn wali_syscall_close(a1: i32) -> i64 {
    unsafe { libc::syscall(libc::SYS_close, a1) }
}

// 4
fn wali_syscall_stat(mut caller: Caller<'_, Host>, a1: i32, a2: i32) -> i64 {
    unsafe {
        libc::syscall(
            libc::SYS_stat,
            maddr(&mut caller, a1),
            maddr(&mut caller, a2),
        )
    }
}

// 5
fn wali_syscall_fstat(mut caller: Caller<'_, Host>, a1: i32, a2: i32) -> i64 {
    unsafe { libc::syscall(libc::SYS_fstat, a1, maddr(&mut caller, a2)) }
}

// 6
fn wali_syscall_lstat(mut caller: Caller<'_, Host>, a1: i32, a2: i32) -> i64 {
    unsafe {
        libc::syscall(
            libc::SYS_lstat,
            maddr(&mut caller, a1),
            maddr(&mut caller, a2),
        )
    }
}

// 7
fn wali_syscall_poll(mut caller: Caller<'_, Host>, a1: i32, a2: i32, a3: i32) -> i64 {
    unsafe { libc::syscall(libc::SYS_poll, maddr(&mut caller, a1), a2, a3) }
}

// 8
fn wali_syscall_lseek(a1: i32, a2: i64, a3: i32) -> i64 {
    unsafe { libc::syscall(libc::SYS_lseek, a1, a2, a3) }
}

// 9
fn wali_syscall_mmap(
    mut caller: Caller<'_, Host>,
    _a1: i32,
    a2: i32,
    _a3: i32,
    _a4: i32,
    _a5: i32,
    _a6: i64,
) -> i64 {
    let extmem = caller.get_export("memory").unwrap();
    let memty = extmem.ty(&caller);

    const PAGE_SIZE: u64 = 65536;
    let delta = a2 as u64 / PAGE_SIZE + 1;

    if let wasmtime::ExternType::Memory(ty) = memty {
        if ty.is_shared() {
            let memory = extmem.into_shared_memory().unwrap();
            let page = memory.grow(delta);
            if page.is_err() {
                let size = memory.data_size();
                println!(
                    "wali_syscall_mmap: Failed to grow memory by {} pages, current size: {}",
                    delta, size
                );
                return -1; // Return -1 on failure
            }
            let size = memory.data_size();
            let retval = size - (delta * PAGE_SIZE) as usize;
            retval.try_into().unwrap()
        } else {
            let memory = extmem.into_memory().unwrap();
            let page = memory.grow(&mut caller, delta);
            if page.is_err() {
                return -1; // Return -1 on failure
            }
            let size = memory.data_size(&caller);
            let retval = size - (delta * PAGE_SIZE) as usize;
            retval.try_into().unwrap()
        }
    } else {
        panic!("Expected a memory export, but found: {:?}", extmem);
    }
}

// 10
fn wali_syscall_mprotect(mut caller: Caller<'_, Host>, a1: i32, a2: i32, a3: i32) -> i64 {
    unsafe { libc::syscall(libc::SYS_mprotect, maddr(&mut caller, a1), a2, a3) }
}

// 11
fn wali_syscall_munmap(_a1: i32, _a2: i32) -> i64 {
    if cfg!(debug_assertions) {
        println!("wali_syscall_munmap is not implemented yet");
    }
    -1
}

// 12
fn wali_syscall_brk(_a1: i32) -> i64 {
    println!("wali_syscall_brk is not implemented yet");
    -1
}

// 13
fn wali_syscall_rt_sigaction(_a1: i32, _a2: i32, _a3: i32, _a4: i32) -> i64 {
    println!("wali_syscall_rt_sigaction is not implemented yet");
    -1
}

// 14
fn wali_syscall_rt_sigprocmask(
    mut caller: Caller<'_, Host>,
    a1: i32,
    a2: i32,
    a3: i32,
    a4: i32,
) -> i64 {
    unsafe {
        libc::syscall(
            libc::SYS_rt_sigprocmask,
            a1,
            maddr(&mut caller, a2),
            maddr(&mut caller, a3),
            a4,
        )
    }
}

// 15: Never directly called; __libc_restore_rt is called by OS
fn wali_syscall_rt_sigreturn(_a1: i64) -> i64 {
    // This function is never directly called in the wasi spec, but is used by the OS to restore signal state.
    // We can return 0 to indicate success, as the OS will handle the rest.
    println!("wali_syscall_rt_sigreturn is not implemented yet");
    -1
}

// 16
fn wali_syscall_ioctl(mut caller: Caller<'_, Host>, a1: i32, a2: i32, a3: i32) -> i64 {
    unsafe { libc::syscall(libc::SYS_ioctl, a1, a2, maddr(&mut caller, a3)) }
}

// 17
fn wali_syscall_pread64(mut caller: Caller<'_, Host>, a1: i32, a2: i32, a3: i32, a4: i64) -> i64 {
    unsafe {
        let ret = libc::syscall(libc::SYS_pread64, a1, maddr(&mut caller, a2), a3, a4);
        if ret < 0 {
            eprintln!(
                "wali_syscall_pread64 error: {:?}",
                std::io::Error::last_os_error()
            );
            eprintln!(
                "wali_syscall_pread64: a1 = {}, a2 = {:x}, a3 = {}, a4 = {}",
                a1, a2, a3, a4
            );
        }
        ret
    }
}

// 18
fn wali_syscall_pwrite64(mut caller: Caller<'_, Host>, a1: i32, a2: i32, a3: i32, a4: i64) -> i64 {
    unsafe { libc::syscall(libc::SYS_pwrite64, a1, maddr(&mut caller, a2), a3, a4) }
}

// 19
fn wali_syscall_readv(mut caller: Caller<'_, Host>, a1: i32, a2: i32, a3: i32) -> i64 {
    let native_iov = copy_iovec(&mut caller, a2, a3);
    unsafe {
        let retval = libc::syscall(libc::SYS_readv, a1, native_iov, a3);
        libc::free(native_iov as *mut c_void);
        retval
    }
}

// 20
fn wali_syscall_writev(mut caller: Caller<'_, Host>, a1: i32, a2: i32, a3: i32) -> i64 {
    let native_iov = copy_iovec(&mut caller, a2, a3);
    unsafe {
        let retval = libc::syscall(libc::SYS_writev, a1, native_iov, a3);
        libc::free(native_iov as *mut c_void);
        retval
    }
}

// 21
fn wali_syscall_access(mut caller: Caller<'_, Host>, a1: i32, a2: i32) -> i64 {
    unsafe { libc::syscall(libc::SYS_access, maddr(&mut caller, a1), a2) }
}

// 22
fn wali_syscall_pipe(mut caller: Caller<'_, Host>, a1: i32) -> i64 {
    unsafe { libc::syscall(libc::SYS_pipe, maddr(&mut caller, a1)) }
}

// 23
fn wali_syscall_select(
    mut caller: Caller<'_, Host>,
    a1: i32,
    a2: i32,
    a3: i32,
    a4: i32,
    a5: i32,
) -> i64 {
    unsafe {
        libc::syscall(
            libc::SYS_select,
            a1,
            maddr(&mut caller, a2),
            maddr(&mut caller, a3),
            maddr(&mut caller, a4),
            maddr(&mut caller, a5),
        )
    }
}

// 24
fn wali_syscall_sched_yield() -> i64 {
    unsafe { libc::syscall(libc::SYS_sched_yield) }
}

// 25
fn wali_syscall_mremap(_a1: i32, _a2: i32, _a3: i32, _a4: i32, _a5: i32) -> i64 {
    println!("wali_syscall_mremap is not implemented yet");
    -1
}

// 26
fn wali_syscall_msync(mut caller: Caller<'_, Host>, a1: i32, a2: i32, a3: i32) -> i64 {
    unsafe { libc::syscall(libc::SYS_msync, maddr(&mut caller, a1), a2, a3) }
}

// 28
fn wali_syscall_madvise(mut caller: Caller<'_, Host>, a1: i32, a2: i32, a3: i32) -> i64 {
    unsafe { libc::syscall(libc::SYS_madvise, maddr(&mut caller, a1), a2, a3) }
}

// 32
fn wali_syscall_dup(a1: i32) -> i64 {
    unsafe { libc::syscall(libc::SYS_dup, a1) }
}

// 33
fn wali_syscall_dup2(a1: i32, a2: i32) -> i64 {
    unsafe {
        let ret = libc::syscall(libc::SYS_dup2, a1, a2);
        if ret < 0 {
            println!(
                "wali_syscall_dup2 error: {:?}",
                std::io::Error::last_os_error()
            );
        }
        ret
    }
}

// 35
fn wali_syscall_nanosleep(mut caller: Caller<'_, Host>, a1: i32, a2: i32) -> i64 {
    unsafe {
        libc::syscall(
            libc::SYS_nanosleep,
            maddr(&mut caller, a1),
            maddr(&mut caller, a2),
        )
    }
}

// 37
fn wali_syscall_alarm(a1: i32) -> i64 {
    unsafe { libc::syscall(libc::SYS_alarm, a1) }
}

// 38
fn wali_syscall_setitimer(mut caller: Caller<'_, Host>, a1: i32, a2: i32, a3: i32) -> i64 {
    unsafe {
        libc::syscall(
            libc::SYS_setitimer,
            a1,
            maddr(&mut caller, a2),
            maddr(&mut caller, a3),
        )
    }
}

// 39
fn wali_syscall_getpid() -> i64 {
    unsafe { libc::syscall(libc::SYS_getpid) }
}

// 41
fn wali_syscall_socket(a1: i32, a2: i32, a3: i32) -> i64 {
    unsafe { libc::syscall(libc::SYS_socket, a1, a2, a3) }
}

// 42
fn wali_syscall_connect(mut caller: Caller<'_, Host>, a1: i32, a2: i32, a3: i32) -> i64 {
    unsafe { libc::syscall(libc::SYS_connect, a1, maddr(&mut caller, a2), a3) }
}

// 43
fn wali_syscall_accept(mut caller: Caller<'_, Host>, a1: i32, a2: i32, a3: i32) -> i64 {
    unsafe {
        libc::syscall(
            libc::SYS_accept,
            a1,
            maddr(&mut caller, a2),
            maddr(&mut caller, a3),
        )
    }
}

// 44
fn wali_syscall_sendto(
    mut caller: Caller<'_, Host>,
    a1: i32,
    a2: i32,
    a3: i32,
    a4: i32,
    a5: i32,
    a6: i32,
) -> i64 {
    unsafe {
        libc::syscall(
            libc::SYS_sendto,
            a1,
            maddr(&mut caller, a2),
            a3,
            a4,
            maddr(&mut caller, a5),
            a6,
        )
    }
}

// 45
fn wali_syscall_recvfrom(
    mut caller: Caller<'_, Host>,
    a1: i32,
    a2: i32,
    a3: i32,
    a4: i32,
    a5: i32,
    a6: i32,
) -> i64 {
    unsafe {
        libc::syscall(
            libc::SYS_recvfrom,
            a1,
            maddr(&mut caller, a2),
            a3,
            a4,
            maddr(&mut caller, a5),
            maddr(&mut caller, a6),
        )
    }
}

// 46
fn wali_syscall_sendmsg(mut _caller: Caller<'_, Host>, _a1: i32, _a2: i32, _a3: i32) -> i64 {
    // Addr wasm_msghdr = MADDR(a2);
    // struct msghdr *native_msghdr = copy_msghdr(exec_env, wasm_msghdr);
    // long retval = __syscall3(SYS_sendmsg, a1, native_msghdr, a3);
    // free(native_msghdr);
    // RETURN(retval, "sendmsg", 3, a1, a2, a3);
    println!("wali_syscall_sendmsg is not implemented yet");
    -1
}

// 47
fn wali_syscall_recvmsg(mut _caller: Caller<'_, Host>, _a1: i32, _a2: i32, _a3: i32) -> i64 {
    // Addr wasm_msghdr = MADDR(a2);
    // struct msghdr *native_msghdr = copy_msghdr(exec_env, wasm_msghdr);
    // long retval = __syscall3(SYS_recvmsg, a1, native_msghdr, a3);
    // free(native_msghdr);
    // RETURN(retval, "recvmsg", 3, a1, a2, a3);
    println!("wali_syscall_recvmsg is not implemented yet");
    -1
}

// 48
fn wali_syscall_shutdown(a1: i32, a2: i32) -> i64 {
    unsafe { libc::syscall(libc::SYS_shutdown, a1, a2) }
}

// 49
fn wali_syscall_bind(mut caller: Caller<'_, Host>, a1: i32, a2: i32, a3: i32) -> i64 {
    unsafe { libc::syscall(libc::SYS_bind, a1, maddr(&mut caller, a2), a3) }
}

// 50
fn wali_syscall_listen(a1: i32, a2: i32) -> i64 {
    unsafe { libc::syscall(libc::SYS_listen, a1, a2) }
}

// 51
fn wali_syscall_getsockname(mut caller: Caller<'_, Host>, a1: i32, a2: i32, a3: i32) -> i64 {
    unsafe {
        libc::syscall(
            libc::SYS_getsockname,
            a1,
            maddr(&mut caller, a2),
            maddr(&mut caller, a3),
        )
    }
}

// 52
fn wali_syscall_getpeername(mut caller: Caller<'_, Host>, a1: i32, a2: i32, a3: i32) -> i64 {
    unsafe {
        libc::syscall(
            libc::SYS_getpeername,
            a1,
            maddr(&mut caller, a2),
            maddr(&mut caller, a3),
        )
    }
}

// 53
fn wali_syscall_socketpair(
    mut caller: Caller<'_, Host>,
    a1: i32,
    a2: i32,
    a3: i32,
    a4: i32,
) -> i64 {
    unsafe { libc::syscall(libc::SYS_socketpair, a1, a2, a3, maddr(&mut caller, a4)) }
}

// 54
fn wali_syscall_setsockopt(
    mut caller: Caller<'_, Host>,
    a1: i32,
    a2: i32,
    a3: i32,
    a4: i32,
    a5: i32,
) -> i64 {
    unsafe { libc::syscall(libc::SYS_setsockopt, a1, a2, a3, maddr(&mut caller, a4), a5) }
}

// 55
fn wali_syscall_getsockopt(
    mut caller: Caller<'_, Host>,
    a1: i32,
    a2: i32,
    a3: i32,
    a4: i32,
    a5: i32,
) -> i64 {
    unsafe {
        libc::syscall(
            libc::SYS_getsockopt,
            a1,
            a2,
            a3,
            maddr(&mut caller, a4),
            maddr(&mut caller, a5),
        )
    }
}

// 57
fn wali_syscall_fork() -> i64 {
    unsafe { libc::syscall(libc::SYS_fork) }
}

// fn create_pass_env_file(envp: *const libc::c_char) {
//     let mut filename: [i8; 100] = [0; 100];
//     unsafe {
//         libc::sprintf(filename.as_mut_ptr(), "/tmp/wali_env.%d".as_ptr().cast(), libc::getpid());
//         let fp = libc::fopen(filename.as_ptr(), "w".as_ptr().cast());
//         // for (char **e = envp; *e; e++) {
//         //     fprintf(fp, "%s\n", *e);
//         // }
//         libc::fclose(fp);
//     }
// }

// 59
fn wali_syscall_execve(mut _caller: Caller<'_, Host>, _a1: i32, _a2: i32, _a3: i32) -> i64 {
    // This function is not implemented in the original code, but it would typically execute a program.
    // For now, we can return an error code to indicate that it is not implemented.
    println!("wali_syscall_execve is not implemented yet");
    -1
}

// 60 TODO
fn wali_syscall_exit(mut _caller: Caller<'_, Host>, _a1: i32) -> i64 {
    // This function is not implemented in the original code, but it would typically terminate the process.
    // For now, we can return an error code to indicate that it is not implemented.
    if cfg!(debug_assertions) {
        println!("wali_syscall_exit is not implemented yet");
    }
    -1
}

// 61
fn wali_syscall_wait4(mut caller: Caller<'_, Host>, a1: i32, a2: i32, a3: i32, a4: i32) -> i64 {
    unsafe {
        libc::syscall(
            libc::SYS_wait4,
            a1,
            maddr(&mut caller, a2),
            a3,
            maddr(&mut caller, a4),
        )
    }
}

// 62
fn wali_syscall_kill(a1: i32, a2: i32) -> i64 {
    unsafe {
        let ret = libc::syscall(libc::SYS_kill, a1, a2);
        if ret < 0 {
            println!(
                "wali_syscall_kill error: {:?}",
                std::io::Error::last_os_error()
            );
        }
        println!("wali_syscall_kill: a1 = {}, a2 = {}", a1, a2);
        ret
    }
}

// 63
fn wali_syscall_uname(mut caller: Caller<'_, Host>, a1: i32) -> i64 {
    unsafe { libc::syscall(libc::SYS_uname, maddr(&mut caller, a1)) }
}

// 72
fn wali_syscall_fcntl(mut caller: Caller<'_, Host>, a1: i32, a2: i32, a3: i32) -> i64 {
    /* Swap open flags only on F_GETFL and F_SETFL mode for aarch64 */
    unsafe {
        match a2 {
            libc::F_GETLK | libc::F_SETLK => {
                libc::syscall(libc::SYS_fcntl, a1, a2, maddr(&mut caller, a3))
            }
            _ => libc::syscall(libc::SYS_fcntl, a1, a2, a3),
        }
    }
}

// 73
fn wali_syscall_flock(a1: i32, a2: i32) -> i64 {
    unsafe { libc::syscall(libc::SYS_flock, a1, a2) }
}

// 74
fn wali_syscall_fsync(a1: i32) -> i64 {
    unsafe { libc::syscall(libc::SYS_fsync, a1) }
}

// 75
fn wali_syscall_fdatasync(a1: i32) -> i64 {
    unsafe { libc::syscall(libc::SYS_fdatasync, a1) }
}

// 77
fn wali_syscall_ftruncate(a1: i32, a2: i64) -> i64 {
    unsafe { libc::syscall(libc::SYS_ftruncate, a1, a2) }
}

// 78
fn wali_syscall_getdents(mut caller: Caller<'_, Host>, a1: i32, a2: i32, a3: i32) -> i64 {
    unsafe { libc::syscall(libc::SYS_getdents, a1, maddr(&mut caller, a2), a3) }
}

// 79
fn wali_syscall_getcwd(mut caller: Caller<'_, Host>, a1: i32, a2: i32) -> i64 {
    unsafe { libc::syscall(libc::SYS_getcwd, maddr(&mut caller, a1), a2) }
}

// 80
fn wali_syscall_chdir(mut caller: Caller<'_, Host>, a1: i32) -> i64 {
    unsafe { libc::syscall(libc::SYS_chdir, maddr(&mut caller, a1)) }
}

// 81
fn wali_syscall_fchdir(a1: i32) -> i64 {
    unsafe { libc::syscall(libc::SYS_fchdir, a1) }
}

// 82
fn wali_syscall_rename(mut caller: Caller<'_, Host>, a1: i32, a2: i32) -> i64 {
    unsafe {
        libc::syscall(
            libc::SYS_rename,
            maddr(&mut caller, a1),
            maddr(&mut caller, a2),
        )
    }
}

// 83
fn wali_syscall_mkdir(mut caller: Caller<'_, Host>, a1: i32, a2: i32) -> i64 {
    unsafe { libc::syscall(libc::SYS_mkdir, maddr(&mut caller, a1), a2) }
}

// 84
fn wali_syscall_rmdir(mut caller: Caller<'_, Host>, a1: i32) -> i64 {
    unsafe { libc::syscall(libc::SYS_rmdir, maddr(&mut caller, a1)) }
}

// 86
fn wali_syscall_link(mut caller: Caller<'_, Host>, a1: i32, a2: i32) -> i64 {
    unsafe {
        libc::syscall(
            libc::SYS_link,
            maddr(&mut caller, a1),
            maddr(&mut caller, a2),
        )
    }
}

// 87
fn wali_syscall_unlink(mut caller: Caller<'_, Host>, a1: i32) -> i64 {
    unsafe { libc::syscall(libc::SYS_unlink, maddr(&mut caller, a1)) }
}

// 88
fn wali_syscall_symlink(mut caller: Caller<'_, Host>, a1: i32, a2: i32) -> i64 {
    unsafe {
        libc::syscall(
            libc::SYS_symlink,
            maddr(&mut caller, a1),
            maddr(&mut caller, a2),
        )
    }
}

// 89
fn wali_syscall_readlink(mut caller: Caller<'_, Host>, a1: i32, a2: i32, a3: i32) -> i64 {
    unsafe {
        libc::syscall(
            libc::SYS_readlink,
            maddr(&mut caller, a1),
            maddr(&mut caller, a2),
            a3,
        )
    }
}

// 90
fn wali_syscall_chmod(mut caller: Caller<'_, Host>, a1: i32, a2: i32) -> i64 {
    unsafe { libc::syscall(libc::SYS_chmod, maddr(&mut caller, a1), a2) }
}

// 91
fn wali_syscall_fchmod(a1: i32, a2: i32) -> i64 {
    unsafe { libc::syscall(libc::SYS_fchmod, a1, a2) }
}

// 92
fn wali_syscall_chown(mut caller: Caller<'_, Host>, a1: i32, a2: i32, a3: i32) -> i64 {
    unsafe { libc::syscall(libc::SYS_chown, maddr(&mut caller, a1), a2, a3) }
}

// 93
fn wali_syscall_fchown(a1: i32, a2: i32, a3: i32) -> i64 {
    unsafe { libc::syscall(libc::SYS_fchown, a1, a2, a3) }
}

// 95
fn wali_syscall_umask(a1: i32) -> i64 {
    unsafe { libc::syscall(libc::SYS_umask, a1) }
}

// 96
fn wali_syscall_gettimeofday(_a1: i32, _a2: i32) -> i64 {
    -1
}

// 97
fn wali_syscall_getrlimit(mut caller: Caller<'_, Host>, a1: i32, a2: i32) -> i64 {
    unsafe { libc::syscall(libc::SYS_getrlimit, a1, maddr(&mut caller, a2)) }
}

// 98
fn wali_syscall_getrusage(mut caller: Caller<'_, Host>, a1: i32, a2: i32) -> i64 {
    unsafe { libc::syscall(libc::SYS_getrusage, a1, maddr(&mut caller, a2)) }
}

// 99
fn wali_syscall_sysinfo(mut caller: Caller<'_, Host>, a1: i32) -> i64 {
    unsafe { libc::syscall(libc::SYS_sysinfo, maddr(&mut caller, a1)) }
}

// 102
fn wali_syscall_getuid() -> i64 {
    unsafe { libc::syscall(libc::SYS_getuid) }
}

// 104
fn wali_syscall_getgid() -> i64 {
    unsafe { libc::syscall(libc::SYS_getgid) }
}

// 105
fn wali_syscall_setuid(a1: i32) -> i64 {
    unsafe { libc::syscall(libc::SYS_setuid, a1) }
}

// 106
fn wali_syscall_setgid(a1: i32) -> i64 {
    unsafe { libc::syscall(libc::SYS_setgid, a1) }
}

// 107
fn wali_syscall_geteuid() -> i64 {
    unsafe { libc::syscall(libc::SYS_geteuid) }
}

// 108
fn wali_syscall_getegid() -> i64 {
    unsafe { libc::syscall(libc::SYS_getegid) }
}

// 109
fn wali_syscall_setpgid(a1: i32, a2: i32) -> i64 {
    unsafe { libc::syscall(libc::SYS_setpgid, a1, a2) }
}

// 110
fn wali_syscall_getppid() -> i64 {
    unsafe { libc::syscall(libc::SYS_getppid) }
}

// 112
fn wali_syscall_setsid() -> i64 {
    unsafe {
        let ret = libc::syscall(libc::SYS_setsid);
        if ret < 0 {
            println!(
                "wali_syscall_setsid error: {:?}",
                std::io::Error::last_os_error()
            );
        }
        ret
    }
}

// 113
fn wali_syscall_setreuid(a1: i32, a2: i32) -> i64 {
    unsafe { libc::syscall(libc::SYS_setreuid, a1, a2) }
}

// 114
fn wali_syscall_setregid(a1: i32, a2: i32) -> i64 {
    unsafe { libc::syscall(libc::SYS_setregid, a1, a2) }
}

// 115
fn wali_syscall_getgroups(mut caller: Caller<'_, Host>, a1: i32, a2: i32) -> i64 {
    unsafe { libc::syscall(libc::SYS_getgroups, a1, maddr(&mut caller, a2)) }
}

// 116
fn wali_syscall_setgroups(mut caller: Caller<'_, Host>, a1: i32, a2: i32) -> i64 {
    unsafe { libc::syscall(libc::SYS_setgroups, a1, maddr(&mut caller, a2)) }
}

// 117
fn wali_syscall_setresuid(a1: i32, a2: i32, a3: i32) -> i64 {
    unsafe { libc::syscall(libc::SYS_setresuid, a1, a2, a3) }
}

// 119
fn wali_syscall_setresgid(a1: i32, a2: i32, a3: i32) -> i64 {
    unsafe { libc::syscall(libc::SYS_setresgid, a1, a2, a3) }
}

// 121
fn wali_syscall_getpgid(a1: i32) -> i64 {
    unsafe { libc::syscall(libc::SYS_getpgid, a1) }
}

// 124
fn wali_syscall_getsid(a1: i32) -> i64 {
    unsafe { libc::syscall(libc::SYS_getsid, a1) }
}

// 127
fn wali_syscall_rt_sigpending(mut caller: Caller<'_, Host>, a1: i32, a2: i32) -> i64 {
    unsafe { libc::syscall(libc::SYS_rt_sigpending, maddr(&mut caller, a1), a2) }
}

// 130
fn wali_syscall_rt_sigsuspend(mut caller: Caller<'_, Host>, a1: i32, a2: i32) -> i64 {
    unsafe { libc::syscall(libc::SYS_rt_sigsuspend, maddr(&mut caller, a1), a2) }
}

// 131
fn wali_syscall_sigaltstack(mut _caller: Caller<'_, Host>, _a1: i32, _a2: i32) -> i64 {
    // Addr wasm_ss = MADDR(a1), wasm_old_ss = MADDR(a2);

    // stack_t ss = { 0 }, old_ss = { 0 };
    // stack_t *ss_ptr = copy_sigstack(exec_env, wasm_ss, &ss);
    // stack_t *old_ss_ptr = copy_sigstack(exec_env, wasm_old_ss, &old_ss);

    // RETURN(__syscall2(SYS_sigaltstack, ss_ptr, old_ss_ptr), "sigaltstack", 2,
    //        a1, a2);
    println!("wali_syscall_sigaltstack is not implemented yet");
    -1
}

// 132
fn wali_syscall_utime(_a1: i32, _a2: i32) -> i64 {
    // RETURN(-1, "utime", 2, a1, a2);
    println!("wali_syscall_utime is not implemented yet");
    -1
}

// 137
fn wali_syscall_statfs(mut caller: Caller<'_, Host>, a1: i32, a2: i32) -> i64 {
    unsafe {
        libc::syscall(
            libc::SYS_statfs,
            maddr(&mut caller, a1),
            maddr(&mut caller, a2),
        )
    }
}

// 138
fn wali_syscall_fstatfs(mut caller: Caller<'_, Host>, a1: i32, a2: i32) -> i64 {
    unsafe { libc::syscall(libc::SYS_fstatfs, a1, maddr(&mut caller, a2)) }
}

// 141
fn wali_syscall_setpriority(which: u32, who: u32, prio: i32) -> i64 {
    unsafe { libc::syscall(libc::SYS_setpriority, which, who, prio) }
}

// 144
fn wali_syscall_sched_setscheduler(_a1: i32, _a2: i32, _a3: i32) -> i64 {
    // This syscall is not implemented in the original code, but it would typically set the scheduling policy and parameters for a process.
    // For now, we can return an error code to indicate that it is not implemented.
    println!("wali_syscall_sched_setscheduler is not implemented yet");
    -1
}

// 157
fn wali_syscall_prctl(a1: i32, a2: i64, a3: i64, a4: i64, a5: i64) -> i64 {
    unsafe { libc::syscall(libc::SYS_prctl, a1, a2, a3, a4, a5) }
}

// 160
fn wali_syscall_setrlimit(mut caller: Caller<'_, Host>, a1: i32, a2: i32) -> i64 {
    unsafe { libc::syscall(libc::SYS_setrlimit, a1, maddr(&mut caller, a2)) }
}

// 161
fn wali_syscall_chroot(mut caller: Caller<'_, Host>, a1: i32) -> i64 {
    unsafe { libc::syscall(libc::SYS_chroot, maddr(&mut caller, a1)) }
}

// 186
fn wali_syscall_gettid() -> i64 {
    unsafe { libc::syscall(libc::SYS_gettid) }
}

// 200
fn wali_syscall_tkill(a1: i32, a2: i32) -> i64 {
    unsafe { libc::syscall(libc::SYS_tkill, a1, a2) }
}

// 202
fn wali_syscall_futex(
    mut caller: Caller<'_, Host>,
    a1: i32,
    a2: i32,
    a3: i32,
    a4: i32,
    a5: i32,
    a6: i32,
) -> i64 {
    unsafe {
        libc::syscall(
            libc::SYS_futex,
            maddr(&mut caller, a1),
            a2,
            a3,
            maddr(&mut caller, a4),
            maddr(&mut caller, a5),
            a6,
        )
    }
}

// 204
fn wali_syscall_sched_getaffinity(mut caller: Caller<'_, Host>, a1: i32, a2: i32, a3: i32) -> i64 {
    unsafe { libc::syscall(libc::SYS_sched_getaffinity, a1, a2, maddr(&mut caller, a3)) }
}

// 217
fn wali_syscall_getdents64(mut caller: Caller<'_, Host>, a1: i32, a2: i32, a3: i32) -> i64 {
    unsafe { libc::syscall(libc::SYS_getdents64, a1, maddr(&mut caller, a2), a3) }
}

// 218
fn wali_syscall_set_tid_address(mut caller: Caller<'_, Host>, a1: i32) -> i64 {
    // unsafe { libc::syscall(libc::SYS_set_tid_address, maddr(&mut caller, a1)) }
    let tid_addr = maddr(&mut caller, a1) as *mut i32;
    unsafe {
        let tid = libc::gettid();
        *tid_addr = tid; // Set the thread ID at the address provided
        tid as i64 // Return the thread ID
    }
}

// 221 TODO
fn wali_syscall_fadvise(a1: i32, a2: i64, a3: i64, a4: i32) -> i64 {
    unsafe { libc::syscall(libc::SYS_fadvise64, a1, a2, a3, a4) }
}

// 228
fn wali_syscall_clock_gettime(mut caller: Caller<'_, Host>, a1: i32, a2: i32) -> i64 {
    unsafe { libc::syscall(libc::SYS_clock_gettime, a1, maddr(&mut caller, a2)) }
}

// 229
fn wali_syscall_clock_getres(mut caller: Caller<'_, Host>, a1: i32, a2: i32) -> i64 {
    unsafe { libc::syscall(libc::SYS_clock_getres, a1, maddr(&mut caller, a2)) }
}

// 230
fn wali_syscall_clock_nanosleep(
    mut caller: Caller<'_, Host>,
    a1: i32,
    a2: i32,
    a3: i32,
    a4: i32,
) -> i64 {
    unsafe {
        libc::syscall(
            libc::SYS_clock_nanosleep,
            a1,
            a2,
            maddr(&mut caller, a3),
            maddr(&mut caller, a4),
        )
    }
}

// 231
fn wali_syscall_exit_group(caller: Caller<'_, Host>, a1: i32) -> i64 {
    wali_proc_exit(caller, a1);
    -1
}

// 233
fn wali_syscall_epoll_ctl(
    mut _caller: Caller<'_, Host>,
    _a1: i32,
    _a2: i32,
    _a3: i32,
    _a4: i32,
) -> i64 {
    // struct epoll_event *nev =
    //     copy_epoll_event(exec_env, MADDR(a4), &(struct epoll_event){ 0 });
    // RETURN(__syscall4(SYS_epoll_ctl, a1, a2, a3, nev), "epoll_ctl", 4, a1, a2,
    //        a3, a4);
    println!("wali_syscall_epoll_ctl is not implemented yet");
    -1
}

// 235
fn wali_syscall_utimes(mut caller: Caller<'_, Host>, filename: i32, times: i32) -> i64 {
    let file = maddr(&mut caller, filename);
    let time = maddr(&mut caller, times);
    unsafe { libc::syscall(libc::SYS_utimes, file, time) }
}

// 257
fn wali_syscall_openat(mut caller: Caller<'_, Host>, a1: i32, a2: i32, a3: i32, a4: i32) -> i64 {
    // security check
    let arg: i64 = maddr(&mut caller, a2);
    unsafe {
        if libc::strncmp(arg as *const i8, "/proc/self/mem".as_ptr().cast(), 15) == 0 {
            println!("Unpermitted attempt to open /proc/self/mem.");
            return -1;
        }
        libc::syscall(libc::SYS_openat, a1, arg, a3, a4)
    }
}

// 258
fn wali_syscall_mkdirat(mut caller: Caller<'_, Host>, a1: i32, a2: i32, a3: i32) -> i64 {
    unsafe { libc::syscall(libc::SYS_mkdirat, a1, maddr(&mut caller, a2), a3) }
}

// 260
fn wali_syscall_fchownat(
    mut caller: Caller<'_, Host>,
    a1: i32,
    a2: i32,
    a3: i32,
    a4: i32,
    a5: i32,
) -> i64 {
    unsafe { libc::syscall(libc::SYS_fchownat, a1, maddr(&mut caller, a2), a3, a4, a5) }
}

// 261
fn wali_syscall_futimesat(_a1: i32, _a2: i32, _a3: i32) -> i64 {
    // RETURN(-1, "futimesat", 3, a1, a2, a3);
    println!("wali_syscall_futimesat is not implemented yet");
    -1
}

// 262
fn wali_syscall_fstatat(mut caller: Caller<'_, Host>, a1: i32, a2: i32, a3: i32, a4: i32) -> i64 {
    unsafe {
        libc::syscall(
            libc::SYS_newfstatat,
            a1,
            maddr(&mut caller, a2),
            maddr(&mut caller, a3),
            a4,
        )
    }
}

// 263
fn wali_syscall_unlinkat(mut caller: Caller<'_, Host>, a1: i32, a2: i32, a3: i32) -> i64 {
    unsafe { libc::syscall(libc::SYS_unlinkat, a1, maddr(&mut caller, a2), a3) }
}

// 265
fn wali_syscall_linkat(
    mut caller: Caller<'_, Host>,
    a1: i32,
    a2: i32,
    a3: i32,
    a4: i32,
    a5: i32,
) -> i64 {
    unsafe {
        libc::syscall(
            libc::SYS_linkat,
            a1,
            maddr(&mut caller, a2),
            a3,
            maddr(&mut caller, a4),
            a5,
        )
    }
}

// 266
fn wali_syscall_symlinkat(mut caller: Caller<'_, Host>, a1: i32, a2: i32, a3: i32) -> i64 {
    unsafe {
        libc::syscall(
            libc::SYS_symlinkat,
            maddr(&mut caller, a1),
            a2,
            maddr(&mut caller, a3),
        )
    }
}

// 267
fn wali_syscall_readlinkat(
    mut caller: Caller<'_, Host>,
    a1: i32,
    a2: i32,
    a3: i32,
    a4: i32,
) -> i64 {
    unsafe {
        libc::syscall(
            libc::SYS_readlinkat,
            a1,
            maddr(&mut caller, a2),
            maddr(&mut caller, a3),
            a4,
        )
    }
}

// 268
fn wali_syscall_fchmodat(mut caller: Caller<'_, Host>, a1: i32, a2: i32, a3: i32, a4: i32) -> i64 {
    unsafe { libc::syscall(libc::SYS_fchmodat, a1, maddr(&mut caller, a2), a3, a4) }
}

// 269
fn wali_syscall_faccessat(mut caller: Caller<'_, Host>, a1: i32, a2: i32, a3: i32, a4: i32) -> i64 {
    unsafe { libc::syscall(libc::SYS_faccessat, a1, maddr(&mut caller, a2), a3, a4) }
}

// 270
fn wali_syscall_pselect6(
    mut _caller: Caller<'_, Host>,
    _a1: i32,
    _a2: i32,
    _a3: i32,
    _a4: i32,
    _a5: i32,
    _a6: i32,
) -> i64 {
    // Addr wasm_psel_sm = MADDR(a6);
    // long sm_struct[2];
    // long *sm_struct_ptr =
    //     copy_pselect6_sigmask(exec_env, wasm_psel_sm, sm_struct);
    // RETURN(__syscall6(SYS_pselect6, a1, MADDR(a2), MADDR(a3), MADDR(a4),
    //                   MADDR(a5), sm_struct_ptr),
    //        "pselect6", 6, a1, a2, a3, a4, a5, a6);
    println!("wali_syscall_pselect6 is not implemented yet");
    -1
}

// 271
fn wali_syscall_ppoll(
    mut caller: Caller<'_, Host>,
    a1: i32,
    a2: i32,
    a3: i32,
    a4: i32,
    a5: i32,
) -> i64 {
    unsafe {
        libc::syscall(
            libc::SYS_ppoll,
            maddr(&mut caller, a1),
            a2,
            maddr(&mut caller, a3),
            maddr(&mut caller, a4),
            a5,
        )
    }
}
/* Since poll needs a time conversion on pointer, need to use a different alias
 * call */
// fn wali_syscall_ppoll_aliased(mut caller: Caller<'_, Host>, a1: i32, a2: i32, a3: i32, a4: i32, a5: i32) -> i64 {
//     unsafe { libc::syscall(libc::SYS_ppoll, maddr(&mut caller, a1), a2, a3, maddr(&mut caller, a4), a5) }
// }

// 273
fn wali_syscall_set_robust_list(mut caller: Caller<'_, Host>, a1: i32, a2: i32) -> i64 {
    unsafe { libc::syscall(libc::SYS_set_robust_list, maddr(&mut caller, a1), a2) }
}

// 280
fn wali_syscall_utimensat(mut caller: Caller<'_, Host>, a1: i32, a2: i32, a3: i32, a4: i32) -> i64 {
    unsafe {
        libc::syscall(
            libc::SYS_utimensat,
            a1,
            maddr(&mut caller, a2),
            maddr(&mut caller, a3),
            a4,
        )
    }
}

// 281
fn wali_syscall_epoll_pwait(
    mut _caller: Caller<'_, Host>,
    _a1: i32,
    _a2: i32,
    _a3: i32,
    _a4: i32,
    _a5: i32,
    _a6: i32,
) -> i64 {
    // Addr wasm_epoll = MADDR(a2);
    // struct epoll_event *nev =
    //     copy_epoll_event(exec_env, wasm_epoll, &(struct epoll_event){ 0 });
    // long retval = __syscall6(SYS_epoll_pwait, a1, nev, a3, a4, MADDR(a5), a6);
    // copy2wasm_epoll_event(exec_env, wasm_epoll, nev);
    // RETURN(retval, "epoll_pwait", 6, a1, a2, a3, a4, a5, a6);
    println!("wali_syscall_epoll_pwait is not implemented yet");
    -1
}

// 284
fn wali_syscall_eventfd(a1: i32) -> i64 {
    unsafe { libc::syscall(libc::SYS_eventfd, a1) }
}

// 288
fn wali_syscall_accept4(mut caller: Caller<'_, Host>, a1: i32, a2: i32, a3: i32, a4: i32) -> i64 {
    unsafe {
        libc::syscall(
            libc::SYS_accept4,
            a1,
            maddr(&mut caller, a2),
            maddr(&mut caller, a3),
            a4,
        )
    }
}

// 290 TODO
fn wali_syscall_eventfd2(a1: i32, a2: i32) -> i64 {
    unsafe { libc::syscall(libc::SYS_eventfd2, a1, a2) }
}

// 291
fn wali_syscall_epoll_create1(a1: i32) -> i64 {
    unsafe { libc::syscall(libc::SYS_epoll_create1, a1) }
}

// 292
fn wali_syscall_dup3(a1: i32, a2: i32, a3: i32) -> i64 {
    unsafe { libc::syscall(libc::SYS_dup3, a1, a2, a3) }
}

// 293
fn wali_syscall_pipe2(mut caller: Caller<'_, Host>, a1: i32, a2: i32) -> i64 {
    unsafe { libc::syscall(libc::SYS_pipe2, maddr(&mut caller, a1), a2) }
}

// 296
fn wali_syscall_pwritev(_a1: i32, _a2: i32, _a3: i32, _a4: i64, _a5: i64) -> i64 {
    println!("wali_syscall_pwritev is not implemented yet");
    -1
}

// 302
fn wali_syscall_prlimit64(mut caller: Caller<'_, Host>, a1: i32, a2: i32, a3: i32, a4: i32) -> i64 {
    unsafe {
        libc::syscall(
            libc::SYS_prlimit64,
            a1,
            a2,
            maddr(&mut caller, a3),
            maddr(&mut caller, a4),
        )
    }
}

// 316
fn wali_syscall_renameat2(
    mut caller: Caller<'_, Host>,
    a1: i32,
    a2: i32,
    a3: i32,
    a4: i32,
    a5: i32,
) -> i64 {
    unsafe {
        libc::syscall(
            libc::SYS_renameat2,
            a1,
            maddr(&mut caller, a2),
            a3,
            maddr(&mut caller, a4),
            a5,
        )
    }
}

// 318
fn wali_syscall_getrandom(mut caller: Caller<'_, Host>, a1: i32, a2: i32, a3: i32) -> i64 {
    unsafe { libc::syscall(libc::SYS_getrandom, maddr(&mut caller, a1), a2, a3) }
}

// 332
fn wali_syscall_statx(
    mut caller: Caller<'_, Host>,
    a1: i32,
    a2: i32,
    a3: i32,
    a4: i32,
    a5: i32,
) -> i64 {
    unsafe {
        libc::syscall(
            libc::SYS_statx,
            a1,
            maddr(&mut caller, a2),
            a3,
            a4,
            maddr(&mut caller, a5),
        )
    }
}

// 439
fn wali_syscall_faccessat2(
    mut caller: Caller<'_, Host>,
    a1: i32,
    a2: i32,
    a3: i32,
    a4: i32,
) -> i64 {
    unsafe { libc::syscall(439, a1, maddr(&mut caller, a2), a3, a4) }
}

/***** Startup *****/
fn wali_call_ctors() {
    unsafe {
        INVOKED_WALI = true;
        CTOR_CALLED = true;
    }
}

fn wali_call_dtors() {
    unsafe {
        DTOR_CALLED = true;
    }
}

// TODO
fn wali_proc_exit(_caller: Caller<'_, Host>, v: i32) {
    unsafe {
        /* if destructor is invoked, main ended successfully, do
         * not set exception */
        if !DTOR_CALLED || v != 0 {
            println!("WALI process exit called prematurely");
            std::process::exit(1);
        } else {
            println!("Main ended successfully");
        }
        PROC_EXIT_PRIMARY_TID = libc::gettid();
        PROC_EXIT_INVOKED = true;
    }
}

fn wali_cl_get_argc() -> i32 {
    unsafe { APP_ARGC }
}

fn wali_cl_get_argv_len(arg_idx: i32) -> i32 {
    unsafe { libc::strlen(APP_ARGV[arg_idx as usize]) as i32 }
}

fn wali_cl_copy_argv(mut caller: Caller<'_, Host>, argv_addr: i32, arg_idx: i32) -> i32 {
    let argv = maddr(&mut caller, argv_addr) as *mut i8;
    unsafe { libc::strcpy(argv, APP_ARGV[arg_idx as usize]) };
    0
}

fn wali_get_init_envfile(mut caller: Caller<'_, Host>, faddr: i32, fsize: i32) -> i32 {
    let fbuf = maddr(&mut caller, faddr) as *mut i8;

    /* Check for passthrough env from an execve call */
    let pass_filename = String::from(format!("/tmp/wali_env.{}", unsafe { libc::getpid() }));
    unsafe {
        let execve_invoked = !libc::access(pass_filename.as_ptr().cast(), libc::R_OK);

        let envfile: *const i8 = if execve_invoked != 0 {
            pass_filename.as_ptr().cast()
        } else {
            std::ptr::null()
        };

        if envfile.is_null() {
            println!("No WALI environment file provided");
            return 0;
        }

        if libc::strlen(envfile) + 1 > fsize as usize {
            println!(
                "WALI env initialization filepath too large (max length: {}). Defaulting to NULL",
                fsize
            );
            (*fbuf.wrapping_add(0)) = 0;
        } else {
            libc::strcpy(fbuf, envfile);
            println!("WALI init env file: \'{:?}\'\n", fbuf);
        }
    }
    0
}

/// Check if wasi-threads' `wasi_thread_start` export is present.
fn has_entry_point(module: &Module) -> bool {
    module.get_export(WASI_ENTRY_POINT).is_some()
}

/// Check if the entry function has the correct signature `(i32, i32) -> ()`.
fn has_correct_signature(module: &Module) -> bool {
    match module.get_export(WASI_ENTRY_POINT) {
        Some(ExternType::Func(ty)) => {
            ty.params().len() == 2
                && ty.params().nth(0).unwrap().is_i32()
                && ty.params().nth(1).unwrap().is_i32()
                && ty.results().len() == 0
        }
        _ => false,
    }
}

// fn wali_wasm_thread_spawn(
//     mut caller: Caller<'_, Host>,
//     _setup_fnptr: i32,
//     thread_start_arg: i32,
// ) -> i32 {
//     let ctx = lhw_get_cx(caller.data());
//     let instance_pre = ctx.get_instance_pre();

//     // Check that the thread entry point is present. Why here? If we check
//     // for this too early, then we cannot accept modules that do not have an
//     // entry point but never spawn a thread. As pointed out in
//     // https://github.com/bytecodealliance/wasmtime/issues/6153, checking
//     // the entry point here allows wasi-threads to be compatible with more
//     // modules.
//     //
//     // As defined in the wasi-threads specification, returning a negative
//     // result here indicates to the guest module that the spawn failed.
//     if !has_entry_point(instance_pre.module()) {
//         log::error!(
//             "failed to find a wasi-threads entry point function; expected an export with name: {WASI_ENTRY_POINT}"
//         );
//         return -1;
//     }
//     if !has_correct_signature(instance_pre.module()) {
//         log::error!(
//             "the exported entry point function has an incorrect signature: expected `(i32, i32) -> ()`"
//         );
//         return -1;
//     }

//     let wasi_thread_id = ctx.next_thread_id();
//     if wasi_thread_id.is_none() {
//         log::error!("ran out of valid thread IDs");
//         return -1;
//     }
//     let wasi_thread_id = wasi_thread_id.unwrap();

//     // Start a Rust thread running a new instance of the current module.
//     let builder = thread::Builder::new().name(format!("wasi-thread-{wasi_thread_id}"));
//     let _ = unsafe {
//         builder.spawn_unchecked(move || {
//             // Catch any panic failures in host code; e.g., if a WASI module
//             // were to crash, we want all threads to exit, not just this one.
//             let result = catch_unwind(AssertUnwindSafe(|| {
//                 let thread_entry_point = caller
//                     .get_export(WASI_ENTRY_POINT)
//                     .expect("wasi-threads entry point not found")
//                     .into_func()
//                     .expect("wasi-threads entry point is not a function");

//                 // Start the thread's entry point. Any traps or calls to
//                 // `proc_exit`, by specification, should end execution for all
//                 // threads. This code uses `process::exit` to do so, which is
//                 // what the user expects from the CLI but probably not in a
//                 // Wasmtime embedding.
//                 log::trace!(
//                     "spawned thread id = {}; calling start function `{}` with: {}",
//                     wasi_thread_id,
//                     WASI_ENTRY_POINT,
//                     thread_start_arg
//                 );

//                 let params = [
//                     wasmtime::Val::I32(wasi_thread_id),
//                     wasmtime::Val::I32(thread_start_arg),
//                 ];
//                 let res = if instance_pre.module().engine().is_async() {
//                     println!(
//                         "wasi-thread-{wasi_thread_id} starting execution 1, 0x{:x}",
//                         thread_start_arg
//                     );
//                     wasmtime_wasi::runtime::in_tokio(thread_entry_point.call_async(
//                         caller,
//                         &params,
//                         &mut [],
//                     ))
//                 } else {
//                     thread_entry_point.call(caller, &params, &mut [])
//                 };
//                 println!("wasi-thread-{wasi_thread_id} finished execution 2");
//                 match res {
//                     Ok(_) => log::trace!("exiting thread id = {} normally", wasi_thread_id),
//                     Err(e) => {
//                         log::trace!("exiting thread id = {} due to error", wasi_thread_id);
//                         let e = wasi_common::maybe_exit_on_error(e);
//                         eprintln!("Error: {e:?}");
//                         std::process::exit(1);
//                     }
//                 }
//             }));

//             if let Err(e) = result {
//                 eprintln!("wasi-thread-{wasi_thread_id} panicked: {e:?}");
//                 std::process::exit(1);
//             }
//         })
//     };
//     println!("wasi-thread-{wasi_thread_id} started");
//     wasi_thread_id
// }

fn wali_wasm_thread_spawn(
    caller: Caller<'_, Host>,
    _setup_fnptr: i32,
    thread_start_arg: i32,
) -> i32 {
    let host = caller.data().clone();
    let ctx = lhw_get_cx(caller.data());
    let instance_pre = ctx.get_instance_pre();

    // Check that the thread entry point is present. Why here? If we check
    // for this too early, then we cannot accept modules that do not have an
    // entry point but never spawn a thread. As pointed out in
    // https://github.com/bytecodealliance/wasmtime/issues/6153, checking
    // the entry point here allows wasi-threads to be compatible with more
    // modules.
    //
    // As defined in the wasi-threads specification, returning a negative
    // result here indicates to the guest module that the spawn failed.
    if !has_entry_point(instance_pre.module()) {
        log::error!(
            "failed to find a wasi-threads entry point function; expected an export with name: {WASI_ENTRY_POINT}"
        );
        return -1;
    }
    if !has_correct_signature(instance_pre.module()) {
        log::error!(
            "the exported entry point function has an incorrect signature: expected `(i32, i32) -> ()`"
        );
        return -1;
    }

    let wasi_thread_id = ctx.next_thread_id();
    if wasi_thread_id.is_none() {
        log::error!("ran out of valid thread IDs");
        return -1;
    }
    let wasi_thread_id = wasi_thread_id.unwrap();

    // Start a Rust thread running a new instance of the current module.
    let builder = thread::Builder::new().name(format!("wasi-thread-{wasi_thread_id}"));
    let res = builder.spawn(move || {
        // Catch any panic failures in host code; e.g., if a WASI module
        // were to crash, we want all threads to exit, not just this one.
        let result = catch_unwind(AssertUnwindSafe(|| {
            // Each new instance is created in its own store.
            let mut store = Store::new(&instance_pre.module().engine(), host);

            let instance = if instance_pre.module().engine().is_async() {
                wasmtime_wasi::runtime::in_tokio(instance_pre.instantiate_async(&mut store))
            } else {
                instance_pre.instantiate(&mut store)
            }
            .unwrap();

            let thread_entry_point = instance
                .get_typed_func::<(i32, i32), ()>(&mut store, WASI_ENTRY_POINT)
                .unwrap();

            // Start the thread's entry point. Any traps or calls to
            // `proc_exit`, by specification, should end execution for all
            // threads. This code uses `process::exit` to do so, which is
            // what the user expects from the CLI but probably not in a
            // Wasmtime embedding.
            log::trace!(
                "spawned thread id = {}; calling start function `{}` with: {}",
                wasi_thread_id,
                WASI_ENTRY_POINT,
                thread_start_arg
            );
            let res = if instance_pre.module().engine().is_async() {
                wasmtime_wasi::runtime::in_tokio(
                    thread_entry_point.call_async(&mut store, (wasi_thread_id, thread_start_arg)),
                )
            } else {
                thread_entry_point.call(&mut store, (wasi_thread_id, thread_start_arg))
            };
            match res {
                Ok(_) => log::trace!("exiting thread id = {} normally", wasi_thread_id),
                Err(e) => {
                    log::trace!("exiting thread id = {} due to error", wasi_thread_id);
                    let e = wasi_common::maybe_exit_on_error(e);
                    eprintln!("Error: {e:?}");
                    std::process::exit(1);
                }
            }
        }));

        if let Err(e) = result {
            eprintln!("wasi-thread-{wasi_thread_id} panicked: {e:?}");
            std::process::exit(1);
        }
    });
    match res {
        Ok(_) => {}
        Err(e) => {
            eprintln!("Failed to spawn wasi-thread-{wasi_thread_id}: {e}");
            return -1;
        }
    }

    wasi_thread_id
}

/// Define wali function
pub(crate) fn define_wali_function(linker: &mut Linker<Host>) {
    linker
        .func_wrap("wali", "SYS_read", wali_syscall_read)
        .unwrap();
    linker
        .func_wrap("wali", "SYS_write", wali_syscall_write)
        .unwrap();
    linker
        .func_wrap("wali", "SYS_open", wali_syscall_open)
        .unwrap();
    linker
        .func_wrap("wali", "SYS_close", wali_syscall_close)
        .unwrap();
    linker
        .func_wrap("wali", "SYS_stat", wali_syscall_stat)
        .unwrap();
    linker
        .func_wrap("wali", "SYS_fstat", wali_syscall_fstat)
        .unwrap();
    linker
        .func_wrap("wali", "SYS_lstat", wali_syscall_lstat)
        .unwrap();
    linker
        .func_wrap("wali", "SYS_poll", wali_syscall_poll)
        .unwrap();
    linker
        .func_wrap("wali", "SYS_lseek", wali_syscall_lseek)
        .unwrap();
    linker
        .func_wrap("wali", "SYS_mmap", wali_syscall_mmap)
        .unwrap();
    linker
        .func_wrap("wali", "SYS_mprotect", wali_syscall_mprotect)
        .unwrap();
    linker
        .func_wrap("wali", "SYS_munmap", wali_syscall_munmap)
        .unwrap();
    linker
        .func_wrap("wali", "SYS_brk", wali_syscall_brk)
        .unwrap();
    linker
        .func_wrap("wali", "SYS_rt_sigaction", wali_syscall_rt_sigaction)
        .unwrap();
    linker
        .func_wrap("wali", "SYS_rt_sigprocmask", wali_syscall_rt_sigprocmask)
        .unwrap();
    linker
        .func_wrap("wali", "SYS_rt_sigreturn", wali_syscall_rt_sigreturn)
        .unwrap();
    linker
        .func_wrap("wali", "SYS_ioctl", wali_syscall_ioctl)
        .unwrap();
    linker
        .func_wrap("wali", "SYS_pread64", wali_syscall_pread64)
        .unwrap();
    linker
        .func_wrap("wali", "SYS_pwrite64", wali_syscall_pwrite64)
        .unwrap();
    linker
        .func_wrap("wali", "SYS_readv", wali_syscall_readv)
        .unwrap();
    linker
        .func_wrap("wali", "SYS_writev", wali_syscall_writev)
        .unwrap();
    linker
        .func_wrap("wali", "SYS_access", wali_syscall_access)
        .unwrap();
    linker
        .func_wrap("wali", "SYS_pipe", wali_syscall_pipe)
        .unwrap();
    linker
        .func_wrap("wali", "SYS_select", wali_syscall_select)
        .unwrap();
    linker
        .func_wrap("wali", "SYS_sched_yield", wali_syscall_sched_yield)
        .unwrap();
    linker
        .func_wrap("wali", "SYS_mremap", wali_syscall_mremap)
        .unwrap();
    linker
        .func_wrap("wali", "SYS_msync", wali_syscall_msync)
        .unwrap();
    linker
        .func_wrap("wali", "SYS_madvise", wali_syscall_madvise)
        .unwrap();
    linker
        .func_wrap("wali", "SYS_dup", wali_syscall_dup)
        .unwrap();
    linker
        .func_wrap("wali", "SYS_dup2", wali_syscall_dup2)
        .unwrap();
    linker
        .func_wrap("wali", "SYS_nanosleep", wali_syscall_nanosleep)
        .unwrap();
    linker
        .func_wrap("wali", "SYS_alarm", wali_syscall_alarm)
        .unwrap();
    linker
        .func_wrap("wali", "SYS_setitimer", wali_syscall_setitimer)
        .unwrap();
    linker
        .func_wrap("wali", "SYS_getpid", wali_syscall_getpid)
        .unwrap();
    linker
        .func_wrap("wali", "SYS_socket", wali_syscall_socket)
        .unwrap();
    linker
        .func_wrap("wali", "SYS_connect", wali_syscall_connect)
        .unwrap();
    linker
        .func_wrap("wali", "SYS_accept", wali_syscall_accept)
        .unwrap();
    linker
        .func_wrap("wali", "SYS_sendto", wali_syscall_sendto)
        .unwrap();
    linker
        .func_wrap("wali", "SYS_recvfrom", wali_syscall_recvfrom)
        .unwrap();
    linker
        .func_wrap("wali", "SYS_sendmsg", wali_syscall_sendmsg)
        .unwrap();
    linker
        .func_wrap("wali", "SYS_recvmsg", wali_syscall_recvmsg)
        .unwrap();
    linker
        .func_wrap("wali", "SYS_shutdown", wali_syscall_shutdown)
        .unwrap();
    linker
        .func_wrap("wali", "SYS_bind", wali_syscall_bind)
        .unwrap();
    linker
        .func_wrap("wali", "SYS_listen", wali_syscall_listen)
        .unwrap();
    linker
        .func_wrap("wali", "SYS_getsockname", wali_syscall_getsockname)
        .unwrap();
    linker
        .func_wrap("wali", "SYS_getpeername", wali_syscall_getpeername)
        .unwrap();
    linker
        .func_wrap("wali", "SYS_socketpair", wali_syscall_socketpair)
        .unwrap();
    linker
        .func_wrap("wali", "SYS_setsockopt", wali_syscall_setsockopt)
        .unwrap();
    linker
        .func_wrap("wali", "SYS_getsockopt", wali_syscall_getsockopt)
        .unwrap();
    linker
        .func_wrap("wali", "SYS_fork", wali_syscall_fork)
        .unwrap();
    linker
        .func_wrap("wali", "SYS_execve", wali_syscall_execve)
        .unwrap();
    linker
        .func_wrap("wali", "SYS_exit", wali_syscall_exit)
        .unwrap();
    linker
        .func_wrap("wali", "SYS_wait4", wali_syscall_wait4)
        .unwrap();
    linker
        .func_wrap("wali", "SYS_kill", wali_syscall_kill)
        .unwrap();
    linker
        .func_wrap("wali", "SYS_uname", wali_syscall_uname)
        .unwrap();
    linker
        .func_wrap("wali", "SYS_fcntl", wali_syscall_fcntl)
        .unwrap();
    linker
        .func_wrap("wali", "SYS_flock", wali_syscall_flock)
        .unwrap();
    linker
        .func_wrap("wali", "SYS_fsync", wali_syscall_fsync)
        .unwrap();
    linker
        .func_wrap("wali", "SYS_fdatasync", wali_syscall_fdatasync)
        .unwrap();
    linker
        .func_wrap("wali", "SYS_ftruncate", wali_syscall_ftruncate)
        .unwrap();
    linker
        .func_wrap("wali", "SYS_getdents", wali_syscall_getdents)
        .unwrap();
    linker
        .func_wrap("wali", "SYS_getcwd", wali_syscall_getcwd)
        .unwrap();
    linker
        .func_wrap("wali", "SYS_chdir", wali_syscall_chdir)
        .unwrap();
    linker
        .func_wrap("wali", "SYS_fchdir", wali_syscall_fchdir)
        .unwrap();
    linker
        .func_wrap("wali", "SYS_rename", wali_syscall_rename)
        .unwrap();
    linker
        .func_wrap("wali", "SYS_mkdir", wali_syscall_mkdir)
        .unwrap();
    linker
        .func_wrap("wali", "SYS_rmdir", wali_syscall_rmdir)
        .unwrap();
    linker
        .func_wrap("wali", "SYS_link", wali_syscall_link)
        .unwrap();
    linker
        .func_wrap("wali", "SYS_unlink", wali_syscall_unlink)
        .unwrap();
    linker
        .func_wrap("wali", "SYS_symlink", wali_syscall_symlink)
        .unwrap();
    linker
        .func_wrap("wali", "SYS_readlink", wali_syscall_readlink)
        .unwrap();
    linker
        .func_wrap("wali", "SYS_chmod", wali_syscall_chmod)
        .unwrap();
    linker
        .func_wrap("wali", "SYS_fchmod", wali_syscall_fchmod)
        .unwrap();
    linker
        .func_wrap("wali", "SYS_chown", wali_syscall_chown)
        .unwrap();
    linker
        .func_wrap("wali", "SYS_fchown", wali_syscall_fchown)
        .unwrap();
    linker
        .func_wrap("wali", "SYS_umask", wali_syscall_umask)
        .unwrap();
    linker
        .func_wrap("wali", "SYS_gettimeofday", wali_syscall_gettimeofday)
        .unwrap();
    linker
        .func_wrap("wali", "SYS_getrlimit", wali_syscall_getrlimit)
        .unwrap();
    linker
        .func_wrap("wali", "SYS_getrusage", wali_syscall_getrusage)
        .unwrap();
    linker
        .func_wrap("wali", "SYS_sysinfo", wali_syscall_sysinfo)
        .unwrap();
    linker
        .func_wrap("wali", "SYS_getuid", wali_syscall_getuid)
        .unwrap();
    linker
        .func_wrap("wali", "SYS_getgid", wali_syscall_getgid)
        .unwrap();
    linker
        .func_wrap("wali", "SYS_setuid", wali_syscall_setuid)
        .unwrap();
    linker
        .func_wrap("wali", "SYS_setgid", wali_syscall_setgid)
        .unwrap();
    linker
        .func_wrap("wali", "SYS_geteuid", wali_syscall_geteuid)
        .unwrap();
    linker
        .func_wrap("wali", "SYS_getegid", wali_syscall_getegid)
        .unwrap();
    linker
        .func_wrap("wali", "SYS_setpgid", wali_syscall_setpgid)
        .unwrap();
    linker
        .func_wrap("wali", "SYS_getppid", wali_syscall_getppid)
        .unwrap();
    linker
        .func_wrap("wali", "SYS_setsid", wali_syscall_setsid)
        .unwrap();
    linker
        .func_wrap("wali", "SYS_setreuid", wali_syscall_setreuid)
        .unwrap();
    linker
        .func_wrap("wali", "SYS_setregid", wali_syscall_setregid)
        .unwrap();
    linker
        .func_wrap("wali", "SYS_getgroups", wali_syscall_getgroups)
        .unwrap();
    linker
        .func_wrap("wali", "SYS_setgroups", wali_syscall_setgroups)
        .unwrap();
    linker
        .func_wrap("wali", "SYS_setresuid", wali_syscall_setresuid)
        .unwrap();
    linker
        .func_wrap("wali", "SYS_setresgid", wali_syscall_setresgid)
        .unwrap();
    linker
        .func_wrap("wali", "SYS_getpgid", wali_syscall_getpgid)
        .unwrap();
    linker
        .func_wrap("wali", "SYS_getsid", wali_syscall_getsid)
        .unwrap();
    linker
        .func_wrap("wali", "SYS_rt_sigpending", wali_syscall_rt_sigpending)
        .unwrap();
    linker
        .func_wrap("wali", "SYS_rt_sigsuspend", wali_syscall_rt_sigsuspend)
        .unwrap();
    linker
        .func_wrap("wali", "SYS_sigaltstack", wali_syscall_sigaltstack)
        .unwrap();
    linker
        .func_wrap("wali", "SYS_utime", wali_syscall_utime)
        .unwrap();
    linker
        .func_wrap("wali", "SYS_statfs", wali_syscall_statfs)
        .unwrap();
    linker
        .func_wrap("wali", "SYS_fstatfs", wali_syscall_fstatfs)
        .unwrap();
    linker
        .func_wrap("wali", "SYS_setpriority", wali_syscall_setpriority)
        .unwrap();
    linker
        .func_wrap(
            "wali",
            "SYS_sched_setscheduler",
            wali_syscall_sched_setscheduler,
        )
        .unwrap();
    linker
        .func_wrap("wali", "SYS_prctl", wali_syscall_prctl)
        .unwrap();
    linker
        .func_wrap("wali", "SYS_setrlimit", wali_syscall_setrlimit)
        .unwrap();
    linker
        .func_wrap("wali", "SYS_chroot", wali_syscall_chroot)
        .unwrap();
    linker
        .func_wrap("wali", "SYS_gettid", wali_syscall_gettid)
        .unwrap();
    linker
        .func_wrap("wali", "SYS_tkill", wali_syscall_tkill)
        .unwrap();
    linker
        .func_wrap("wali", "SYS_futex", wali_syscall_futex)
        .unwrap();
    linker
        .func_wrap(
            "wali",
            "SYS_sched_getaffinity",
            wali_syscall_sched_getaffinity,
        )
        .unwrap();
    linker
        .func_wrap("wali", "SYS_getdents64", wali_syscall_getdents64)
        .unwrap();
    linker
        .func_wrap("wali", "SYS_set_tid_address", wali_syscall_set_tid_address)
        .unwrap();
    linker
        .func_wrap("wali", "SYS_fadvise", wali_syscall_fadvise)
        .unwrap();
    linker
        .func_wrap("wali", "SYS_clock_gettime", wali_syscall_clock_gettime)
        .unwrap();
    linker
        .func_wrap("wali", "SYS_clock_getres", wali_syscall_clock_getres)
        .unwrap();
    linker
        .func_wrap("wali", "SYS_clock_nanosleep", wali_syscall_clock_nanosleep)
        .unwrap();
    linker
        .func_wrap("wali", "SYS_exit_group", wali_syscall_exit_group)
        .unwrap();
    linker
        .func_wrap("wali", "SYS_epoll_ctl", wali_syscall_epoll_ctl)
        .unwrap();
    linker
        .func_wrap("wali", "SYS_utimes", wali_syscall_utimes)
        .unwrap();
    linker
        .func_wrap("wali", "SYS_openat", wali_syscall_openat)
        .unwrap();
    linker
        .func_wrap("wali", "SYS_mkdirat", wali_syscall_mkdirat)
        .unwrap();
    linker
        .func_wrap("wali", "SYS_fchownat", wali_syscall_fchownat)
        .unwrap();
    linker
        .func_wrap("wali", "SYS_futimesat", wali_syscall_futimesat)
        .unwrap();
    linker
        .func_wrap("wali", "SYS_fstatat", wali_syscall_fstatat)
        .unwrap();
    linker
        .func_wrap("wali", "SYS_unlinkat", wali_syscall_unlinkat)
        .unwrap();
    linker
        .func_wrap("wali", "SYS_linkat", wali_syscall_linkat)
        .unwrap();
    linker
        .func_wrap("wali", "SYS_symlinkat", wali_syscall_symlinkat)
        .unwrap();
    linker
        .func_wrap("wali", "SYS_readlinkat", wali_syscall_readlinkat)
        .unwrap();
    linker
        .func_wrap("wali", "SYS_fchmodat", wali_syscall_fchmodat)
        .unwrap();
    linker
        .func_wrap("wali", "SYS_faccessat", wali_syscall_faccessat)
        .unwrap();
    linker
        .func_wrap("wali", "SYS_pselect6", wali_syscall_pselect6)
        .unwrap();
    linker
        .func_wrap("wali", "SYS_ppoll", wali_syscall_ppoll)
        .unwrap();
    linker
        .func_wrap("wali", "SYS_set_robust_list", wali_syscall_set_robust_list)
        .unwrap();
    linker
        .func_wrap("wali", "SYS_utimensat", wali_syscall_utimensat)
        .unwrap();
    linker
        .func_wrap("wali", "SYS_epoll_pwait", wali_syscall_epoll_pwait)
        .unwrap();
    linker
        .func_wrap("wali", "SYS_eventfd", wali_syscall_eventfd)
        .unwrap();
    linker
        .func_wrap("wali", "SYS_accept4", wali_syscall_accept4)
        .unwrap();
    linker
        .func_wrap("wali", "SYS_eventfd2", wali_syscall_eventfd2)
        .unwrap();
    linker
        .func_wrap("wali", "SYS_epoll_create1", wali_syscall_epoll_create1)
        .unwrap();
    linker
        .func_wrap("wali", "SYS_dup3", wali_syscall_dup3)
        .unwrap();
    linker
        .func_wrap("wali", "SYS_pipe2", wali_syscall_pipe2)
        .unwrap();
    linker
        .func_wrap("wali", "SYS_pwritev", wali_syscall_pwritev)
        .unwrap();
    linker
        .func_wrap("wali", "SYS_prlimit64", wali_syscall_prlimit64)
        .unwrap();
    linker
        .func_wrap("wali", "SYS_renameat2", wali_syscall_renameat2)
        .unwrap();
    linker
        .func_wrap("wali", "SYS_getrandom", wali_syscall_getrandom)
        .unwrap();
    linker
        .func_wrap("wali", "SYS_statx", wali_syscall_statx)
        .unwrap();
    linker
        .func_wrap("wali", "SYS_faccessat2", wali_syscall_faccessat2)
        .unwrap();

    /* Libc imports */
    // Threads
    // thread_spawn is the substitute for syscall(clone)
    linker
        .func_wrap("wali", "__wasm_thread_spawn", wali_wasm_thread_spawn)
        .unwrap();

    // Startup
    linker
        .func_wrap("wali", "__call_ctors", wali_call_ctors)
        .unwrap();
    linker
        .func_wrap("wali", "__call_dtors", wali_call_dtors)
        .unwrap();
    linker
        .func_wrap("wali", "__proc_exit", wali_proc_exit)
        .unwrap();
    linker
        .func_wrap("wali", "__cl_get_argc", wali_cl_get_argc)
        .unwrap();
    linker
        .func_wrap("wali", "__cl_get_argv_len", wali_cl_get_argv_len)
        .unwrap();
    linker
        .func_wrap("wali", "__cl_copy_argv", wali_cl_copy_argv)
        .unwrap();
    linker
        .func_wrap("wali", "__get_init_envfile", wali_get_init_envfile)
        .unwrap();
}
