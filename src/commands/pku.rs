use crate::commands::define_raiden_function;
use crate::commands::run::Host;
use anyhow::Result;
use bitflags::bitflags;
use libc::{c_char, c_void, stat, timespec, utimbuf, FILE};
use once_cell::sync::Lazy;
use std::collections::HashMap;
use wasmtime::{Caller, Linker};

static mut FILE_MAP: Lazy<HashMap<u32, *mut FILE>> = Lazy::new(|| HashMap::new());

bitflags! {
    /// Rust libc oflags
    pub struct PKUOFlags: u32 {
        /// O_RDONLY
        const RDONLY    = 0x04000000;
        /// O_WRONLY
        const WRONLY    = 0x10000000;
        /// O_RDWR
        const RDWR      = 0x14000000;
        /// O_CREAT
        const CREATE    = 0x1000;
        /// O_EXCL
        const EXCLUSIVE = 0x4000;
        /// O_TRUNC
        const TRUNCATE  = 0x8000;
    }
}

impl PKUOFlags {
    fn from_wasm(flags: u32) -> i32 {
        let mut oflags = 0;
        if flags & PKUOFlags::RDWR.bits() > 0 {
            oflags = oflags | libc::O_RDWR;
        } else {
            if flags & PKUOFlags::WRONLY.bits() > 0 {
                oflags = oflags | libc::O_WRONLY;
            }
        }
        if flags & PKUOFlags::CREATE.bits() > 0 {
            oflags = oflags | libc::O_CREAT;
        }
        if flags & PKUOFlags::EXCLUSIVE.bits() > 0 {
            oflags = oflags | libc::O_EXCL;
        }
        if flags & PKUOFlags::TRUNCATE.bits() > 0 {
            oflags = oflags | libc::O_TRUNC;
        }
        oflags
    }
}

// fn pku_fopen(mut caller: Caller<'_, Host>, pathname: u32, mode: u32) -> u64 {
//     let memory = caller.get_export("memory").unwrap().into_memory().unwrap();

//     let linear_memory: &[u8] = memory.data(&caller);

//     unsafe {
//         let filename: *const c_char = linear_memory.as_ptr().add(pathname as usize).cast();
//         let m: *const c_char = linear_memory.as_ptr().add(mode as usize).cast();
//         let fp = libc::fopen(filename, m);
//         if fp.is_null() {
//             println!("pku_fopen error");
//             0
//         } else {
//             let ptr: u64 = std::mem::transmute(fp);
//             ptr
//         }
//     }
// }

// fn pku_fdopen(mut caller: Caller<'_, Host>, fildes: i32, mode: u32) -> u64 {
//     let memory = caller.get_export("memory").unwrap().into_memory().unwrap();

//     let linear_memory: &[u8] = memory.data(&caller);

//     unsafe {
//         let m: *const c_char = linear_memory.as_ptr().add(mode as usize).cast();
//         let fp = libc::fdopen(fildes, m);
//         if fp.is_null() {
//             println!("pku_fdopen error");
//             0
//         } else {
//             let ptr: u64 = std::mem::transmute(fp);
//             ptr
//         }
//     }
// }

// fn pku_fclose(stream: u64) -> i32 {
//     let ret = unsafe { libc::fclose(stream as *mut FILE) };
//     if ret < 0 {
//         println!("pku_fclose error");
//     }
//     ret
// }

// fn pku_fflush(stream: u64) -> i32 {
//     let ret = unsafe { libc::fflush(stream as *mut FILE) };
//     if ret < 0 {
//         println!("pku_fflush error");
//     }
//     ret
// }

// fn pku_fgetc(stream: u64) -> i32 {
//     let ret = unsafe { libc::fgetc(stream as *mut FILE) };
//     if ret < 0 {
//         println!("pku_fgetc error");
//     }
//     ret
// }

// fn pku_ungetc(c: i32, stream: u64) -> i32 {
//     let ret = unsafe { libc::ungetc(c, stream as *mut FILE) };
//     if ret < 0 {
//         println!("pku_ungetc error");
//     }
//     ret
// }

// fn pku_fread(mut caller: Caller<'_, Host>, ptr: u32, size: u32, nmemb: u32, stream: u64) -> u32 {
//     let memory = caller.get_export("memory").unwrap().into_memory().unwrap();

//     let linear_memory: &[u8] = memory.data(&caller);

//     unsafe {
//         let buf: *const c_void = linear_memory.as_ptr().add(ptr as usize).cast();
//         let ret = libc::fread(buf as *mut c_void, size as usize, nmemb as usize, stream as *mut FILE);
//         return ret as u32;
//     }
// }

// fn pku_fwrite(mut caller: Caller<'_, Host>, ptr: u32, size: u32, nmemb: u32, stream: u64) -> u32 {
//     let memory = caller.get_export("memory").unwrap().into_memory().unwrap();

//     let linear_memory: &[u8] = memory.data(&caller);

//     unsafe {
//         let buf: *const c_void = linear_memory.as_ptr().add(ptr as usize).cast();
//         let ret = libc::fwrite(buf, size as usize, nmemb as usize, stream as *mut FILE);
//         return ret as u32;
//     }
// }

// fn pku_fseek(stream: u64, offset: i32, whence: i32) -> i32 {
//     let ret = unsafe { libc::fseek(stream as *mut FILE, offset.into(), whence) };
//     if ret < 0 {
//         println!("pku_fseek error");
//     }
//     ret
// }

// fn pku_rewind(stream: u64) {
//     unsafe { libc::rewind(stream as *mut FILE) };
// }

// fn pku_feof(stream: u64) -> i32 {
//     let ret = unsafe { libc::feof(stream as *mut FILE) };
//     if ret < 0 {
//         println!("pku_feof error");
//     }
//     ret
// }

// fn pku_ferror(stream: u64) -> i32 {
//     // let now = std::time::Instant::now();
//     let ret = unsafe { libc::ferror(stream as *mut FILE) };
//     if ret < 0 {
//         println!("pku_ferror error");
//     }
//     // let new_now = now.elapsed().as_nanos();
//     // println!("pku_ferror: {new_now}");
//     ret
// }

// fn pku_fileno(stream: u64) -> i32 {
//     let ret = unsafe { libc::fileno(stream as *mut FILE) };
//     if ret < 0 {
//         println!("pku_fileno error");
//     }
//     ret
// }

fn pku_fopen(mut caller: Caller<'_, Host>, pathname: u32, mode: u32) -> u32 {
    let memory = caller.get_export("memory").unwrap().into_memory().unwrap();

    let linear_memory: &[u8] = memory.data(&caller);

    unsafe {
        let filename: *const c_char = linear_memory.as_ptr().add(pathname as usize).cast();
        let m: *const c_char = linear_memory.as_ptr().add(mode as usize).cast();
        let fp = libc::fopen(filename, m);
        if fp.is_null() {
            println!("pku_fopen error");
            0
        } else {
            let ptr: u64 = std::mem::transmute(fp);
            let ret = ptr as u32;
            FILE_MAP.insert(ret, fp);
            ret
        }
    }
}

fn pku_fdopen(mut caller: Caller<'_, Host>, fildes: i32, mode: u32) -> u32 {
    let memory = caller.get_export("memory").unwrap().into_memory().unwrap();

    let linear_memory: &[u8] = memory.data(&caller);

    unsafe {
        let m: *const c_char = linear_memory.as_ptr().add(mode as usize).cast();
        let fp = libc::fdopen(fildes, m);
        if fp.is_null() {
            println!("pku_fdopen error");
            0
        } else {
            let ptr: u64 = std::mem::transmute(fp);
            let ret = ptr as u32;
            FILE_MAP.insert(ret, fp);
            ret
        }
    }
}

fn pku_fclose(stream: u32) -> i32 {
    let fp = unsafe { FILE_MAP.get(&stream) };
    match fp {
        Some(f) => {
            let ret = unsafe { libc::fclose(*f as *mut FILE) };
            if ret < 0 {
                println!("pku_fclose error");
            }
            ret
        }
        None => {
            println!("FILE pointer error");
            -1
        }
    }
}

fn pku_fflush(stream: u32) -> i32 {
    let fp = unsafe { FILE_MAP.get(&stream) };
    match fp {
        Some(f) => {
            let ret = unsafe { libc::fflush(*f as *mut FILE) };
            if ret < 0 {
                println!("pku_fflush error");
            }
            ret
        }
        None => {
            println!("FILE pointer error");
            -1
        }
    }
}

fn pku_fgetc(stream: u32) -> i32 {
    let fp = unsafe { FILE_MAP.get(&stream) };
    match fp {
        Some(f) => {
            let ret = unsafe { libc::fgetc(*f as *mut FILE) };
            if ret < 0 {
                println!("pku_fgetc error");
            }
            ret
        }
        None => {
            println!("FILE pointer error");
            -1
        }
    }
}

fn pku_ungetc(c: i32, stream: u32) -> i32 {
    let fp = unsafe { FILE_MAP.get(&stream) };
    match fp {
        Some(f) => {
            let ret = unsafe { libc::ungetc(c, *f as *mut FILE) };
            if ret < 0 {
                println!("pku_ungetc error");
            }
            ret
        }
        None => {
            println!("FILE pointer error");
            -1
        }
    }
}

fn pku_fread(mut caller: Caller<'_, Host>, ptr: u32, size: u32, nmemb: u32, stream: u32) -> u32 {
    let memory = caller.get_export("memory").unwrap().into_memory().unwrap();

    let linear_memory: &[u8] = memory.data(&caller);

    unsafe {
        let buf: *const c_void = linear_memory.as_ptr().add(ptr as usize).cast();
        let fp = FILE_MAP.get(&stream);
        match fp {
            Some(f) => {
                let ret = libc::fread(
                    buf as *mut c_void,
                    size as usize,
                    nmemb as usize,
                    *f as *mut FILE,
                );
                ret as u32
            }
            None => {
                println!("FILE pointer error");
                0
            }
        }
    }
}

fn pku_fwrite(mut caller: Caller<'_, Host>, ptr: u32, size: u32, nmemb: u32, stream: u32) -> u32 {
    let memory = caller.get_export("memory").unwrap().into_memory().unwrap();

    let linear_memory: &[u8] = memory.data(&caller);

    unsafe {
        let buf: *const c_void = linear_memory.as_ptr().add(ptr as usize).cast();
        let fp = FILE_MAP.get(&stream);
        match fp {
            Some(f) => {
                let ret = libc::fwrite(buf, size as usize, nmemb as usize, *f as *mut FILE);
                ret as u32
            }
            None => {
                println!("FILE pointer error");
                0
            }
        }
    }
}

fn pku_fseek(stream: u32, offset: i32, whence: i32) -> i32 {
    let fp = unsafe { FILE_MAP.get(&stream) };
    match fp {
        Some(f) => {
            let ret = unsafe { libc::fseek(*f as *mut FILE, offset.into(), whence) };
            if ret < 0 {
                println!("pku_fseek error");
            }
            ret
        }
        None => {
            println!("FILE pointer error");
            -1
        }
    }
}

fn pku_rewind(stream: u32) {
    let fp = unsafe { FILE_MAP.get(&stream) };
    match fp {
        Some(f) => {
            unsafe { libc::rewind(*f as *mut FILE) };
        }
        None => {
            println!("FILE pointer error");
        }
    }
}

fn pku_feof(stream: u32) -> i32 {
    let fp = unsafe { FILE_MAP.get(&stream) };
    match fp {
        Some(f) => {
            let ret = unsafe { libc::feof(*f as *mut FILE) };
            if ret < 0 {
                println!("pku_feof error");
            }
            ret
        }
        None => {
            println!("FILE pointer error");
            -1
        }
    }
}

fn pku_ferror(stream: u32) -> i32 {
    // let now = std::time::Instant::now();
    let fp = unsafe { FILE_MAP.get(&stream) };
    match fp {
        Some(f) => {
            let ret = unsafe { libc::ferror(*f as *mut FILE) };
            if ret < 0 {
                println!("pku_ferror error");
            }
            ret
        }
        None => {
            println!("FILE pointer error");
            -1
        }
    }
    // let new_now = now.elapsed().as_nanos();
    // println!("pku_ferror: {new_now}");
}

fn pku_fileno(stream: u32) -> i32 {
    let fp = unsafe { FILE_MAP.get(&stream) };
    match fp {
        Some(f) => {
            let ret = unsafe { libc::fileno(*f as *mut FILE) };
            if ret < 0 {
                println!("pku_fileno error");
            }
            ret
        }
        None => {
            println!("FILE pointer error");
            -1
        }
    }
}

fn pku_open(mut caller: Caller<'_, Host>, pathname: u32, flags: i32, mode: u32) -> i32 {
    let memory = caller.get_export("memory").unwrap().into_memory().unwrap();

    let linear_memory: &[u8] = memory.data(&caller);

    let oflag = PKUOFlags::from_wasm(flags.try_into().unwrap());
    unsafe {
        let filename: *const c_char = linear_memory.as_ptr().add(pathname as usize).cast();
        let fd = libc::open(filename, oflag, mode);
        if fd < 0 {
            println!("pku_open error: {:?}", std::io::Error::last_os_error());
        }
        fd
    }
}

fn pku_read(mut caller: Caller<'_, Host>, fd: i32, ptr: u32, size: u32) -> u32 {
    let memory = caller.get_export("memory").unwrap().into_memory().unwrap();

    let linear_memory: &[u8] = memory.data(&caller);

    unsafe {
        let buf: *const c_void = linear_memory.as_ptr().add(ptr as usize).cast();
        let ret = libc::read(fd, buf as *mut c_void, size as usize);
        return ret as u32;
    }
}

fn pku_write(mut caller: Caller<'_, Host>, fd: i32, ptr: u32, size: u32) -> u32 {
    let memory = caller.get_export("memory").unwrap().into_memory().unwrap();

    let linear_memory: &[u8] = memory.data(&caller);

    unsafe {
        let buf: *const c_void = linear_memory.as_ptr().add(ptr as usize).cast();
        let ret = libc::write(fd, buf as *mut c_void, size as usize);
        return ret as u32;
    }
}

fn pku_close(fd: i32) -> i32 {
    unsafe { libc::close(fd) }
}

fn pku_stat(mut caller: Caller<'_, Host>, filename: u32, buf: u32) -> i32 {
    let memory = caller.get_export("memory").unwrap().into_memory().unwrap();

    let linear_memory: &[u8] = memory.data(&caller);

    unsafe {
        let file: *const c_char = linear_memory.as_ptr().add(filename as usize).cast();
        let buffer = linear_memory.as_ptr().add(buf as usize).cast_mut() as *mut stat;
        let ret = libc::stat(file, buffer);
        if ret < 0 {
            println!("pku_stat error");
        }
        ret
    }
}

fn pku_utime(mut caller: Caller<'_, Host>, filename: u32, times: u32) -> i32 {
    let memory = caller.get_export("memory").unwrap().into_memory().unwrap();

    let linear_memory: &[u8] = memory.data(&caller);

    unsafe {
        let file: *const c_char = linear_memory.as_ptr().add(filename as usize).cast();
        let time: *const utimbuf = linear_memory.as_ptr().add(times as usize).cast();
        let ret = libc::utime(file, time);
        if ret < 0 {
            println!("pku_utime error");
        }
        ret
    }
}

fn pku_lseek(fd: i32, offset: i64, whence: i32) -> i64 {
    unsafe {
        let ret = libc::lseek(fd, offset, whence);
        if ret < 0 {
            println!("pku_lseek error");
        }
        ret
    }
}

fn pku_fsync(fd: i32) -> i32 {
    // let now = std::time::Instant::now();
    let ret = unsafe { libc::fsync(fd) };
    if ret < 0 {
        println!("pku_fsync error");
    }
    // let new_now = now.elapsed().as_nanos();
    // println!("pku_fsync: {new_now}");
    ret
}

fn pku_fdatasync(fd: i32) -> i32 {
    let ret = unsafe { libc::fdatasync(fd) };
    if ret < 0 {
        println!("pku_fdatasync error");
    }
    ret
}

fn pku_fstat(mut caller: Caller<'_, Host>, fd: i32, stat: u32) -> i32 {
    let memory = caller.get_export("memory").unwrap().into_memory().unwrap();

    let linear_memory: &[u8] = memory.data(&caller);

    unsafe {
        let s = linear_memory.as_ptr().add(stat as usize).cast() as *const stat as *mut stat;
        let ret = libc::fstat(fd, s);
        if ret < 0 {
            println!("pku_fstat error");
        }
        ret
    }
}

fn pku_malloc(size: u32) -> u32 {
    unsafe {
        let ptr = libc::malloc(size.try_into().unwrap());
        if ptr.is_null() {
            println!("pku_malloc error");
            0
        } else {
            let ptr: u64 = std::mem::transmute(ptr);
            ptr as u32
        }
    }
}

fn pku_clock_gettime(mut caller: Caller<'_, Host>, clockid: i32, tp: u32) -> i32 {
    let memory = caller.get_export("memory").unwrap().into_memory().unwrap();

    let linear_memory: &[u8] = memory.data(&caller);
    unsafe {
        let spec = linear_memory.as_ptr().add(tp as usize).cast_mut() as *mut timespec;
        let ret = libc::clock_gettime(clockid, spec);
        if ret < 0 {
            println!("pku_clock_gettime error");
        }
        ret
    }
}

fn pku_dlopen(mut caller: Caller<'_, Host>, pathname: u32) -> u64 {
    let memory = caller.get_export("memory").unwrap().into_memory().unwrap();

    let linear_memory: &[u8] = memory.data(&caller);

    unsafe {
        let filename: *const c_char = linear_memory.as_ptr().add(pathname as usize).cast();
        let fp = libc::dlopen(filename, libc::RTLD_NOW);
        if fp.is_null() {
            println!("pku_dlopen error");
            0
        } else {
            let ptr: u64 = std::mem::transmute(fp);
            ptr
        }
    }
}

fn pku_dlsym(mut caller: Caller<'_, Host>, handle: u64, pathname: u32) -> u64 {
    let memory = caller.get_export("memory").unwrap().into_memory().unwrap();

    let linear_memory: &[u8] = memory.data(&caller);

    unsafe {
        let funcname: *const c_char = linear_memory.as_ptr().add(pathname as usize).cast();
        let fp = libc::dlsym(handle as *mut c_void, funcname);
        if fp.is_null() {
            println!("pku_dlsym error");
            0
        } else {
            let ptr: u64 = std::mem::transmute(fp);
            ptr
        }
    }
}

fn pku_dlcall(mut caller: Caller<'_, Host>, filename: u32, funcname: u32) -> u64 {
    let memory = caller.get_export("memory").unwrap().into_memory().unwrap();

    let linear_memory: &[u8] = memory.data(&caller);

    unsafe {
        let file: *const c_char = linear_memory.as_ptr().add(filename as usize).cast();
        let func: *const c_char = linear_memory.as_ptr().add(funcname as usize).cast();
        let handle = libc::dlopen(file, libc::RTLD_NOW);
        if handle.is_null() {
            println!("libc::dlopen error");
            0
        } else {
            let fp = libc::dlsym(handle, func);
            if fp.is_null() {
                println!("libc::dlsym error");
                0
            } else {
                let ptr: u64 = std::mem::transmute(fp);
                ptr
            }
        }
    }
}

/// Define env function
pub fn define_intrinsic_function(linker: &mut Linker<Host>) -> Result<()> {
    linker.func_wrap("env", "PKUFopen", pku_fopen)?;
    linker.func_wrap("env", "PKUFdopen", pku_fdopen)?;
    linker.func_wrap("env", "PKUFclose", pku_fclose)?;
    linker.func_wrap("env", "PKUFflush", pku_fflush)?;
    linker.func_wrap("env", "PKUFgetc", pku_fgetc)?;
    linker.func_wrap("env", "PKUUngetc", pku_ungetc)?;
    linker.func_wrap("env", "PKUFread", pku_fread)?;
    linker.func_wrap("env", "PKUFwrite", pku_fwrite)?;
    linker.func_wrap("env", "PKUFseek", pku_fseek)?;
    linker.func_wrap("env", "PKURewind", pku_rewind)?;
    linker.func_wrap("env", "PKUFeof", pku_feof)?;
    linker.func_wrap("env", "PKUFerror", pku_ferror)?;
    linker.func_wrap("env", "PKUFileno", pku_fileno)?;
    linker.func_wrap("env", "PKUOpen", pku_open)?;
    linker.func_wrap("env", "PKURead", pku_read)?;
    linker.func_wrap("env", "PKUWrite", pku_write)?;
    linker.func_wrap("env", "PKUClose", pku_close)?;
    linker.func_wrap("env", "PKUStat", pku_stat)?;
    linker.func_wrap("env", "PKUUtime", pku_utime)?;
    linker.func_wrap("env", "PKULseek", pku_lseek)?;
    linker.func_wrap("env", "PKUFsync", pku_fsync)?;
    linker.func_wrap("env", "PKUFdatasync", pku_fdatasync)?;
    linker.func_wrap("env", "PKUFstat", pku_fstat)?;
    linker.func_wrap("env", "PKUMalloc", pku_malloc)?;
    linker.func_wrap("env", "PKUClockGettime", pku_clock_gettime)?;
    linker.func_wrap("env", "PKUDlopen", pku_dlopen)?;
    linker.func_wrap("env", "PKUDlsym", pku_dlsym)?;
    linker.func_wrap("env", "PKUDlcall", pku_dlcall)?;

    define_raiden_function(linker)
}
