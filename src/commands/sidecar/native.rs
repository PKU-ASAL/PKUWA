use crate::commands::call::Host;
use bitflags::bitflags;
use event_listener::{Event, Listener};
use libc::{addrinfo, c_char, c_int, c_void, sockaddr, socklen_t};
use once_cell::sync::Lazy;
use std::sync::atomic::{AtomicBool, Ordering};
use wasmtime::{Caller, Linker};

struct PKUSend {
    ret: isize,
    socket: i32,
    buf: u64,
    len: u64,
    flags: i32,
}

impl PKUSend {
    fn new() -> Self {
        Self {
            ret: 0,
            socket: 0,
            buf: 0,
            len: 0,
            flags: 0,
        }
    }

    fn set_struct(&mut self, socket: i32, buf: u64, len: u64, flags: i32) {
        self.socket = socket;
        self.buf = buf;
        self.len = len;
        self.flags = flags;
    }

    fn set_ret(&mut self, ret: isize) {
        self.ret = ret;
    }

    fn get_ret(&self) -> isize {
        self.ret
    }

    fn get_socket(&self) -> i32 {
        self.socket
    }

    fn get_buf(&self) -> u64 {
        self.buf
    }

    fn get_len(&self) -> u64 {
        self.len
    }

    fn get_flags(&self) -> i32 {
        self.flags
    }
}

static SIDECAR_FLAG: AtomicBool = AtomicBool::new(false);
static SEND_WAIT_EVENT: Lazy<Event> = Lazy::new(|| Event::new());
static SEND_FINISH_EVENT: Lazy<Event> = Lazy::new(|| Event::new());
static mut PKUSEND: Lazy<PKUSend> = Lazy::new(|| PKUSend::new());
static mut SHARED_MEMORY: *mut c_void = std::ptr::null_mut();

bitflags! {
    struct PKUOFlags: u32 {
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

    struct PKUNetFlags: u32 {
        /// AF_INET
        const AF_INET = 1;
        /// AF_INET6
        const AF_INET6 = 2;
        /// AF_UNIX
        const AF_UNIX = 3;
        /// SOCK_DGRAM
        const SOCK_DGRAM = 5;
        /// SOCK_STREAM
        const SOCK_STREAM = 6;
        /// SOCK_NONBLOCK
        const SOCK_NONBLOCK = 0x00004000;
        /// SOCK_CLOEXEC
        const SOCK_CLOEXEC = 0x00002000;
    }
}

impl PKUNetFlags {
    fn from_wasm(flags: u32) -> i32 {
        let mut oflags = 0;

        if flags == PKUNetFlags::AF_INET.bits() {
            oflags = oflags | libc::AF_INET;
        } else if flags == PKUNetFlags::AF_INET6.bits() {
            oflags = oflags | libc::AF_INET6;
        } else if flags == PKUNetFlags::AF_UNIX.bits() {
            oflags = oflags | libc::AF_UNIX;
        }

        if oflags > 0 {
            return oflags;
        }

        if flags == PKUNetFlags::SOCK_DGRAM.bits() {
            oflags = oflags | libc::SOCK_DGRAM;
        } else if flags == PKUNetFlags::SOCK_STREAM.bits() {
            oflags = oflags | libc::SOCK_STREAM;
        } else if flags == PKUNetFlags::SOCK_NONBLOCK.bits() {
            oflags = oflags | libc::SOCK_NONBLOCK;
        } else if flags == PKUNetFlags::SOCK_CLOEXEC.bits() {
            oflags = oflags | libc::SOCK_CLOEXEC;
        }
        oflags
    }
}

fn pku_close(fd: i32) -> i32 {
    unsafe { libc::close(fd) }
}

fn pku_setsockopt(
    mut caller: Caller<'_, Host>,
    fd: i32,
    level: i32,
    optname: i32,
    optval: u32,
    optlen: u32,
) -> i32 {
    let memory = caller.get_export("memory").unwrap().into_memory().unwrap();

    let linear_memory: &[u8] = memory.data(&caller);
    unsafe {
        let value = linear_memory.as_ptr().add(optval as usize).cast() as *const c_void;
        let ret = libc::setsockopt(fd, level, optname, value, optlen);
        if ret < 0 {
            println!("pku_setsockopt error");
        }
        ret
    }
}

fn pku_getaddrinfo(
    mut caller: Caller<'_, Host>,
    name: u32,
    service: u32,
    req: u32,
    pai: u32,
) -> i32 {
    let memory = caller.get_export("memory").unwrap().into_memory().unwrap();

    let linear_memory: &[u8] = memory.data(&caller);
    let node: *const c_char;
    let srv: *const c_char;
    unsafe {
        if name == 0 {
            node = std::ptr::null();
        } else {
            node = linear_memory.as_ptr().add(name as usize).cast() as *const c_char;
        }
        if service == 0 {
            srv = std::ptr::null();
        } else {
            srv = linear_memory.as_ptr().add(service as usize).cast() as *const c_char;
        }
        let hints = linear_memory.as_ptr().add(req as usize).cast() as *const addrinfo;
        let res = linear_memory.as_ptr().add(pai as usize).cast_mut() as *mut *mut addrinfo;
        let ret = libc::getaddrinfo(node, srv, hints, res);
        if ret < 0 {
            println!("pku_getaddrinfo error");
        }
        ret
    }
}

fn pku_freeaddrinfo(mut caller: Caller<'_, Host>, ai: u32) {
    let memory = caller.get_export("memory").unwrap().into_memory().unwrap();

    let linear_memory: &[u8] = memory.data(&caller);
    unsafe {
        let res = linear_memory.as_ptr().add(ai as usize).cast_mut() as *mut addrinfo;
        libc::freeaddrinfo(res);
    }
}

/// 注意：这里已经进行了转换，传入的 domain 和 ty 是 PKUNetFlags 的值，如果发现端口没有被监听，请查看源码中的socket函数
fn pku_socket(domain: i32, ty: i32, protocol: i32) -> i32 {
    let did = PKUNetFlags::from_wasm(domain.try_into().unwrap());
    let tyid = PKUNetFlags::from_wasm(ty.try_into().unwrap());
    unsafe {
        let ret = libc::socket(did, tyid, protocol);
        if ret < 0 {
            println!("pku_socket error");
        }
        ret
    }
}

fn pku_bind(mut caller: Caller<'_, Host>, fd: i32, addr: u32, len: u32) -> i32 {
    let memory = caller.get_export("memory").unwrap().into_memory().unwrap();

    let linear_memory: &[u8] = memory.data(&caller);
    unsafe {
        let address = linear_memory.as_ptr().add(addr as usize).cast() as *const sockaddr;
        let ret = libc::bind(fd, address, len);
        if ret < 0 {
            libc::perror(std::ptr::null());
        }
        ret
    }
}

fn pku_connect(mut caller: Caller<'_, Host>, fd: i32, addr: u32, len: u32) -> i32 {
    let memory = caller.get_export("memory").unwrap().into_memory().unwrap();

    let linear_memory: &[u8] = memory.data(&caller);
    unsafe {
        let address = linear_memory.as_ptr().add(addr as usize).cast() as *const sockaddr;
        let ret = libc::connect(fd, address, len);
        if ret < 0 {
            println!("pku_connect error");
        }
        ret
    }
}

fn pku_listen(fd: i32, n: i32) -> i32 {
    unsafe {
        let ret = libc::listen(fd, n);
        if ret < 0 {
            println!("pku_listen error");
        }
        ret
    }
}

fn pku_accept(mut caller: Caller<'_, Host>, fd: i32, addr: u32, addr_len: u32) -> i32 {
    let memory = caller.get_export("memory").unwrap().into_memory().unwrap();

    let linear_memory: &[u8] = memory.data(&caller);
    unsafe {
        let address = linear_memory.as_ptr().add(addr as usize).cast_mut() as *mut sockaddr;
        let len = linear_memory.as_ptr().add(addr_len as usize).cast_mut() as *mut socklen_t;
        let ret = libc::accept(fd, address, len);
        ret
    }
}

fn pku_getpeername(mut caller: Caller<'_, Host>, fd: i32, addr: u32, len: u32) -> i32 {
    let memory = caller.get_export("memory").unwrap().into_memory().unwrap();

    let linear_memory: &[u8] = memory.data(&caller);
    unsafe {
        let address = linear_memory.as_ptr().add(addr as usize).cast_mut() as *mut sockaddr;
        let addr_len = linear_memory.as_ptr().add(len as usize).cast_mut() as *mut socklen_t;
        let ret = libc::getpeername(fd, address, addr_len);
        if ret < 0 {
            println!("pku_getpeername error");
        }
        ret
    }
}

fn pku_getsockname(mut caller: Caller<'_, Host>, fd: i32, addr: u32, len: u32) -> i32 {
    let memory = caller.get_export("memory").unwrap().into_memory().unwrap();

    let linear_memory: &[u8] = memory.data(&caller);
    unsafe {
        let address = linear_memory.as_ptr().add(addr as usize).cast_mut() as *mut sockaddr;
        let addr_len = linear_memory.as_ptr().add(len as usize).cast_mut() as *mut socklen_t;
        let ret = libc::getsockname(fd, address, addr_len);
        if ret < 0 {
            println!("pku_getsockname error");
        }
        ret
    }
}

fn pku_gethostname(mut caller: Caller<'_, Host>, name: u32, len: u64) -> i32 {
    let memory = caller.get_export("memory").unwrap().into_memory().unwrap();

    let linear_memory: &[u8] = memory.data(&caller);
    unsafe {
        let filename = linear_memory.as_ptr().add(name as usize).cast_mut() as *mut i8;
        let ret = libc::gethostname(filename, len.try_into().unwrap());
        if ret < 0 {
            println!("pku_gethostname error");
        }
        ret
    }
}

fn pku_accept4(mut caller: Caller<'_, Host>, fd: i32, addr: u32, addr_len: u32, flags: i32) -> i32 {
    let memory = caller.get_export("memory").unwrap().into_memory().unwrap();

    let linear_memory: &[u8] = memory.data(&caller);
    unsafe {
        let address = linear_memory.as_ptr().add(addr as usize).cast_mut() as *mut sockaddr;
        let len = linear_memory.as_ptr().add(addr_len as usize).cast_mut() as *mut socklen_t;
        let ret = libc::accept4(fd, address, len, flags);
        if ret < 0 {
            println!("pku_accept4 error");
        }
        ret
    }
}

fn pku_sendto(
    mut caller: Caller<'_, Host>,
    fd: i32,
    buf: u32,
    n: u32,
    flags: i32,
    addr: u32,
    addr_len: u32,
) -> i32 {
    let memory = caller.get_export("memory").unwrap().into_memory().unwrap();

    let linear_memory: &[u8] = memory.data(&caller);
    let ret: isize;
    unsafe {
        if SIDECAR_FLAG.load(Ordering::SeqCst) {
            let buffer = linear_memory.as_ptr().add(buf as usize) as u64;
            (*PKUSEND).set_struct(fd, buffer, n as u64, flags);
            SEND_WAIT_EVENT.notify(1);
            SEND_FINISH_EVENT.listen().wait();
            ret = (*PKUSEND).get_ret();
        } else {
            let buffer: *const c_void = linear_memory.as_ptr().add(buf as usize).cast();
            let address = linear_memory.as_ptr().add(addr as usize).cast_mut() as *mut sockaddr;
            ret = libc::sendto(fd, buffer, n.try_into().unwrap(), flags, address, addr_len);
        }
    }
    if ret < 0 {
        println!("pku_sendto error");
    }
    ret as i32
}

fn pku_socketpair(
    mut caller: Caller<'_, Host>,
    domain: i32,
    ty: i32,
    protocol: i32,
    fds: u32,
) -> i32 {
    let memory = caller.get_export("memory").unwrap().into_memory().unwrap();

    let linear_memory: &[u8] = memory.data(&caller);
    unsafe {
        let fd = linear_memory.as_ptr().add(fds as usize).cast_mut() as *mut c_int;
        let ret = libc::socketpair(domain, ty, protocol, fd);
        if ret < 0 {
            println!("pku_socketpair error");
        }
        ret
    }
}

fn pku_recv(mut caller: Caller<'_, Host>, fd: i32, buf: u32, n: u32, flags: i32) -> i32 {
    let memory = caller.get_export("memory").unwrap().into_memory().unwrap();

    let linear_memory: &[u8] = memory.data(&caller);
    unsafe {
        let buffer = linear_memory.as_ptr().add(buf as usize).cast_mut() as *mut c_void;
        let ret = libc::recv(fd, buffer, n.try_into().unwrap(), flags);
        if ret < 0 {
            println!("pku_recv error");
        }
        ret as i32
    }
}

fn pku_sidecar(_port: u32) {
    SIDECAR_FLAG.store(true, Ordering::SeqCst);
    loop {
        SEND_WAIT_EVENT.listen().wait();
        let pkusend = unsafe { &mut *PKUSEND };
        let fd = pkusend.get_socket();
        let buffer = pkusend.get_buf() as *const c_void;
        let n = pkusend.get_len();
        let flags = pkusend.get_flags();
        unsafe {
            let ret = libc::send(fd, buffer, n.try_into().unwrap(), flags);
            pkusend.set_ret(ret);
        }
        SEND_FINISH_EVENT.notify(1);
        break;
    }
}

fn pku_sharded_memory(mut caller: Caller<'_, Host>) -> i32 {
    let memory = caller.get_export("memory").unwrap().into_memory().unwrap();
    const PAGE_SIZE: usize = 4096;
    unsafe {
        if SHARED_MEMORY.is_null() {
            SHARED_MEMORY = libc::mmap(
                std::ptr::null_mut(),
                PAGE_SIZE,
                libc::PROT_READ | libc::PROT_WRITE,
                libc::MAP_SHARED | libc::MAP_ANONYMOUS,
                -1,
                0,
            );
            if SHARED_MEMORY == libc::MAP_FAILED {
                println!("pku_sharded_memory mmap error");
                return -1;
            } else {
                return 0;
            }
        } else {
            let page = memory.grow(&mut caller, 1);
            match page {
                Ok(p) => {
                    let base = memory.data_ptr(&caller);
                    let ret = base.add(p as usize * PAGE_SIZE);
                    libc::mremap(
                        SHARED_MEMORY,
                        0,
                        PAGE_SIZE,
                        libc::MREMAP_FIXED | libc::MREMAP_MAYMOVE,
                        ret,
                    );
                    return 0;
                }
                Err(e) => {
                    println!("Error in memory.grow function: {e}");
                    return -1;
                }
            }
        }
    }
}

fn pku_create_shareded_memory(mut caller: Caller<'_, Host>, size: u32) -> i32 {
    println!("First time!");
    let host = caller.data_mut();
    let region = host.create_shared_memory(size as usize).unwrap();
    region as i32
}

fn pku_link_shared_memrory(mut caller: Caller<'_, Host>, region: u32) -> i32 {
    let memory = caller.get_export("memory").unwrap().into_memory().unwrap();
    const PAGE_SIZE: usize = 4096;
    let host = caller.data_mut();
    unsafe {
        if let Some(ptr) = host.get_shared_memory(region as u64) {
            let size = host.get_shared_memory_region_size(region as u64);
            if let None = size {
                println!("Shared memory region {} not found", region);
            }
            let size = size.unwrap();

            let p = match memory.grow(&mut caller, size as u64) {
                Ok(p) => p,
                Err(e) => {
                    println!("Error in memory.grow function: {e}");
                    return -1;
                }
            };

            let base = memory.data_ptr(&caller);
            let ret = base.add(p as usize * PAGE_SIZE);
            println!("base: {base:?}, ret: {ret:?}, size: {size}");
            let remaped_ptr =
                libc::mremap(ptr, 0, size, libc::MREMAP_FIXED | libc::MREMAP_MAYMOVE, ret);
            if remaped_ptr == libc::MAP_FAILED {
                println!("pku_shared_memory mremap error");
                return -1;
            }

            let host = caller.data_mut();
            println!("Register host in pku_link_shared_memory: region: {region}, p: {p}");
            match host.register_host(region as u64, p as usize * PAGE_SIZE) {
                Ok(_) => {
                    return 0;
                }
                Err(str) => {
                    println!("Error when register host in pku_link_shared_memory: {str}");
                    return -1;
                }
            }
        } else {
            println!("Shared memory region {} not found", region);
            return -1;
        }
    }
}

fn pku_query_shared_memory(mut caller: Caller<'_, Host>, region: u32) -> i32 {
    let host = caller.data_mut();
    if host.get_shared_memory(region as u64).is_some() {
        return 0; // Shared memory region exists
    }
    -1 // Shared memory region does not exist
}

fn pku_read_shared_memory_i32(mut caller: Caller<'_, Host>, region: u32, offset: u32) -> i32 {
    let memory = caller.get_export("memory").unwrap().into_memory().unwrap();
    let host = caller.data_mut();
    let base = host.get_host_base(region as u64);
    if base.is_none() {
        println!("Read memory region {} error!", region);
        return -1;
    }
    let base = base.unwrap();
    let target_addr = base + offset as usize;
    println!("Read memory region: {region}, base:{base}, offset: {offset}");
    if target_addr + std::mem::size_of::<i32>() > memory.data_size(&caller) {
        return -2;
    }
    unsafe {
        let host_base_ptr = memory.data_ptr(&caller);
        let target_ptr = host_base_ptr.add(target_addr) as *mut i32;
        println!("target_ptr: {target_ptr:?}, target_addr: {target_addr:?}");
        return std::ptr::read(target_ptr);
    }
}

fn pku_read_shared_memory_u32(mut caller: Caller<'_, Host>, region: u32, offset: u32) -> u32 {
    let memory = caller.get_export("memory").unwrap().into_memory().unwrap();
    let host = caller.data_mut();
    let base = host.get_host_base(region as u64);
    if base.is_none() {
        println!("Read memory region {} error!", region);
        return 0;
    }
    let base = base.unwrap();
    let target_addr = base + offset as usize;
    println!("Read memory region: {region}, base:{base}, offset: {offset}");
    if target_addr + std::mem::size_of::<i32>() > memory.data_size(&caller) {
        return 0;
    }
    unsafe {
        let host_base_ptr = memory.data_ptr(&caller);
        let target_ptr = host_base_ptr.add(target_addr) as *mut u32;
        println!("target_ptr: {target_ptr:?}, target_addr: {target_addr:?}");
        return std::ptr::read(target_ptr);
    }
}

fn pku_write_shared_memory_i32(
    mut caller: Caller<'_, Host>,
    region: u32,
    offset: u32,
    value: i32,
) -> i32 {
    let memory = caller.get_export("memory").unwrap().into_memory().unwrap();
    let host = caller.data_mut();
    let base = host.get_host_base(region as u64);
    if base.is_none() {
        println!("Read memory region {} error!", region);
        return -1;
    }
    let base = base.unwrap();
    let target_addr = base + offset as usize;
    if target_addr + std::mem::size_of::<i32>() > memory.data_size(&caller) {
        return -2;
    }
    unsafe {
        let host_base_ptr = memory.data_ptr(&caller);
        let target_ptr = host_base_ptr.add(target_addr) as *mut i32;
        println!("target_ptr: {target_ptr:?}, target_addr: {target_addr:?}");
        std::ptr::write(target_ptr, value);
        return 0;
    }
}

fn pku_write_shared_memory_u32(
    mut caller: Caller<'_, Host>,
    region: u32,
    offset: u32,
    value: u32,
) -> i32 {
    let memory = caller.get_export("memory").unwrap().into_memory().unwrap();
    let host = caller.data_mut();
    let base = host.get_host_base(region as u64);
    if base.is_none() {
        println!("Read memory region {} error!", region);
        return -1;
    }
    let base = base.unwrap();
    let target_addr = base + offset as usize;
    if target_addr + std::mem::size_of::<i32>() > memory.data_size(&caller) {
        return -2;
    }
    unsafe {
        let host_base_ptr = memory.data_ptr(&caller);
        let target_ptr = host_base_ptr.add(target_addr) as *mut u32;
        println!("target_ptr: {target_ptr:?}, target_addr: {target_addr:?}");
        std::ptr::write(target_ptr, value);
        return 0;
    }
}

fn pku_read_shared_memory_buffer(
    mut caller: Caller<'_, Host>,
    region: u32,
    offset: u32,
    buffer_pointer: u32,
    length: u32,
) -> i32 {
    let memory = caller.get_export("memory").unwrap().into_memory().unwrap();
    let host = caller.data_mut();
    let base = match host.get_host_base(region as u64) {
        Some(base) => base,
        None => {
            println!("Read memory region {} error!", region);
            return -1;
        }
    };

    let dest_addr = buffer_pointer as usize;
    if dest_addr + length as usize > memory.data_size(&caller) {
        println!(
            "Read memory region {} error: destination out of bounds",
            region
        );
        return -2;
    }
    let source_addr = base + offset as usize;
    if source_addr + length as usize > memory.data_size(&caller) {
        println!("Read memory region {} error: source out of bounds", region);
        return -3;
    }
    println!(
        "Read buffer from memory region: {region}, base:{base}, offset: {offset}, length: {length}, source_addr: {source_addr}, dest_addr: {dest_addr}"
    );

    let mem_slice = memory.data_mut(&mut caller);
    mem_slice.copy_within(source_addr..source_addr + length as usize, dest_addr);
    0
}

fn pku_write_shared_memory_buffer(
    mut caller: Caller<'_, Host>,
    region: u32,
    offset: u32,
    buffer_pointer: u32,
    length: u32,
) -> i32 {
    let memory = caller.get_export("memory").unwrap().into_memory().unwrap();
    let host = caller.data_mut();
    let base = match host.get_host_base(region as u64) {
        Some(base) => base,
        None => {
            println!("Read memory region {} error!", region);
            return -1;
        }
    };

    let dest_addr = base + offset as usize;
    if dest_addr + length as usize > memory.data_size(&caller) {
        println!(
            "Write memory region {} error: destination out of bounds",
            region
        );
        return -2;
    }
    let source_addr = buffer_pointer as usize;
    if source_addr + length as usize > memory.data_size(&caller) {
        println!("Write memory region {} error: source out of bounds", region);
        return -3;
    }
    println!(
        "Write buffer to memory region: {region}, base:{base}, offset: {offset}, length: {length}, source_addr: {source_addr}, dest_addr: {dest_addr}"
    );

    let mem_slice = memory.data_mut(&mut caller);
    let source_slice = &mem_slice[source_addr..source_addr + length as usize].to_vec();
    let dest_slice = &mut mem_slice[dest_addr..dest_addr + length as usize];
    dest_slice.copy_from_slice(source_slice);
    0
}

fn pku_release_shared_memory(mut caller: Caller<'_, Host>, region: u32) -> i32 {
    let host = caller.data_mut();
    let result = host.release_shared_memory(region as u64);
    match result {
        Ok(_) => 0,
        Err(str) => {
            println!("pku_release_shared_memory error: {str}");
            -1
        }
    }
}

/// Define env function
pub(crate) fn define_native_function(linker: &mut Linker<Host>) {
    linker.func_wrap("env", "PKUClose", pku_close).unwrap();
    linker
        .func_wrap("env", "PKUSetsockopt", pku_setsockopt)
        .unwrap();
    linker
        .func_wrap("env", "PKUGetaddrinfo", pku_getaddrinfo)
        .unwrap();
    linker
        .func_wrap("env", "PKUFreeaddrinfo", pku_freeaddrinfo)
        .unwrap();
    linker.func_wrap("env", "PKUSocket", pku_socket).unwrap();
    linker.func_wrap("env", "PKUBind", pku_bind).unwrap();
    linker.func_wrap("env", "PKUListen", pku_listen).unwrap();
    linker.func_wrap("env", "PKUConnect", pku_connect).unwrap();
    linker.func_wrap("env", "PKUAccept", pku_accept).unwrap();
    linker
        .func_wrap("env", "PKUGetpeername", pku_getpeername)
        .unwrap();
    linker
        .func_wrap("env", "PKUGetsockname", pku_getsockname)
        .unwrap();
    linker
        .func_wrap("env", "PKUGethostname", pku_gethostname)
        .unwrap();
    linker.func_wrap("env", "PKUAccept4", pku_accept4).unwrap();
    linker.func_wrap("env", "PKUSendto", pku_sendto).unwrap();
    linker
        .func_wrap("env", "PKUSocketpair", pku_socketpair)
        .unwrap();
    linker.func_wrap("env", "PKURecv", pku_recv).unwrap();
    linker.func_wrap("env", "PKUSidecar", pku_sidecar).unwrap();
    linker
        .func_wrap("env", "PKUSharedMemory", pku_sharded_memory)
        .unwrap();
    linker
        .func_wrap("env", "PKUCreateSharedMemory", pku_create_shareded_memory)
        .unwrap();
    linker
        .func_wrap("env", "PKULinkSharedMemory", pku_link_shared_memrory)
        .unwrap();
    linker
        .func_wrap("env", "PKUQuerySharedMemory", pku_query_shared_memory)
        .unwrap();
    linker
        .func_wrap("env", "PKUReadSharedMemoryInt", pku_read_shared_memory_i32)
        .unwrap();
    linker
        .func_wrap(
            "env",
            "PKUWriteSharedMemoryInt",
            pku_write_shared_memory_i32,
        )
        .unwrap();
    linker
        .func_wrap(
            "env",
            "PKUReadSharedMemoryUnsignedInt",
            pku_read_shared_memory_u32,
        )
        .unwrap();
    linker
        .func_wrap(
            "env",
            "PKUWriteSharedMemoryUnsignedInt",
            pku_write_shared_memory_u32,
        )
        .unwrap();
    linker
        .func_wrap(
            "env",
            "PKUReadSharedMemoryBuffer",
            pku_read_shared_memory_buffer,
        )
        .unwrap();
    linker
        .func_wrap(
            "env",
            "PKUWriteSharedMemoryBuffer",
            pku_write_shared_memory_buffer,
        )
        .unwrap();
    linker
        .func_wrap("env", "PKUReleaseSharedMemory", pku_release_shared_memory)
        .unwrap();
}
