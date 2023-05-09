use alloc::vec::Vec;
use core::alloc::{GlobalAlloc, Layout};
use core::ops::{Deref, DerefMut};
use core::sync::atomic::{AtomicUsize, Ordering};

use Dlmalloc;

#[cfg(target_family = "wasm")]
use Monitor;

pub use sys::enable_alloc_after_fork;

/// An instance of a "global allocator" backed by `Dlmalloc`
///
/// This API requires the `global` feature is activated, and this type
/// implements the `GlobalAlloc` trait in the standard library.
pub struct GlobalDlmalloc;

unsafe impl GlobalAlloc for GlobalDlmalloc {
    #[inline]
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        <Dlmalloc>::malloc(&mut get(), layout.size(), layout.align())
    }

    #[inline]
    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        <Dlmalloc>::free(&mut get(), ptr, layout.size(), layout.align())
    }

    #[inline]
    unsafe fn alloc_zeroed(&self, layout: Layout) -> *mut u8 {
        <Dlmalloc>::calloc(&mut get(), layout.size(), layout.align())
    }

    #[inline]
    unsafe fn realloc(&self, ptr: *mut u8, layout: Layout, new_size: usize) -> *mut u8 {
        <Dlmalloc>::realloc(&mut get(), ptr, layout.size(), layout.align(), new_size)
    }
}

impl GlobalDlmalloc {
    /// Get a global domain id
    pub fn get_domain_id() -> usize {
        DOMAINID.load(Ordering::Relaxed)
    }

    /// Modify a global domain id
    pub fn switch_domain(id: usize) {
        DOMAINID.store(id, Ordering::Relaxed);
    }

    /// Get all memory footprint in allocator
    #[cfg(target_arch = "wasm32")]
    pub fn get_memory_footprint() -> usize {
        unsafe {
            let mut ret = Monitor::monitor_footprint() + DLMALLOC.0.footprint;
            for iter in DOMAINALLOC.iter() {
                ret += iter.0.footprint;
            }
            ret
        }
    }

    /// debug function
    #[cfg(target_arch = "wasm32")]
    pub fn memory_footprint_vector() -> Vec<usize> {
        let mut ret: Vec<usize> = Vec::new();
        unsafe {
            ret.push(Monitor::monitor_footprint());
            ret.push(DLMALLOC.0.footprint);
            for iter in DOMAINALLOC.iter() {
                ret.push(iter.0.footprint);
            }
        }
        ret
    }
}

static mut DLMALLOC: Dlmalloc = Dlmalloc::new();
static mut DOMAINALLOC: Vec<Dlmalloc> = Vec::new();
static DOMAINID: AtomicUsize = AtomicUsize::new(0);

static mut ALLOCFLAG: bool = false;
unsafe fn alloc_allocator(id: usize) {
    while ALLOCFLAG == false && DOMAINALLOC.len() <= id {
        ALLOCFLAG = true;
        DOMAINALLOC.push(Dlmalloc::new());
        ALLOCFLAG = false;
    }
}

struct Instance;

unsafe fn get() -> Instance {
    ::sys::acquire_global_lock();
    Instance
}

impl Deref for Instance {
    type Target = Dlmalloc;
    fn deref(&self) -> &Dlmalloc {
        unsafe {
            let id = DOMAINID.load(Ordering::Relaxed);
            if id == 0 {
                return &DLMALLOC;
            } else {
                alloc_allocator(id - 1);
                match DOMAINALLOC.get(id - 1) {
                    Some(allocator) => return allocator,
                    None => return &DLMALLOC,
                }
            }
            // return &DLMALLOC;
        }
    }
}

impl DerefMut for Instance {
    fn deref_mut(&mut self) -> &mut Dlmalloc {
        unsafe {
            let id = DOMAINID.load(Ordering::Relaxed);
            if id == 0 {
                return &mut DLMALLOC;
            } else {
                alloc_allocator(id - 1);
                match DOMAINALLOC.get_mut(id - 1) {
                    Some(allocator) => return allocator,
                    None => return &mut DLMALLOC,
                }
            }
            // return &mut DLMALLOC;
        }
    }
}

impl Drop for Instance {
    fn drop(&mut self) {
        ::sys::release_global_lock()
    }
}

macro_rules! wasicall {
    ($e:expr) => {{
        getrandom::getrandom($e)
    }};
    () => {};
}

macro_rules! CreateDomain {
    ($prot:expr) => {{
        // let mut ret = Self::get_freed_domain();
        // if ret != 0 {
        //     return ret;
        // }
        let ret;
        let mut buf = [0x01, 0x21, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0];
        let result = wasicall!(&mut buf);
        match result {
            Ok(_) => ret = buf[2] as usize,
            Err(_) => {
                panic!("CreateDomain");
            }
        }
        unsafe {
            DOMAINS.prot[ret] = $prot;
            DOMAINS.used[ret] = 1;
        }
        ret
    }};
    () => {
        wasicall!();
    };
}

macro_rules! FreeDomain {
    ($pkey:expr) => {{
        // unsafe {
        //     DOMAINS.prot[pkey] = 0;
        //     DOMAINS.used[pkey] = 2;
        // }
        let mut buf = [0x01, 0x4B, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0];
        buf[2..6].copy_from_slice(&$pkey.to_be_bytes());
        let result = wasicall!(&mut buf);
        match result {
            Ok(_) => unsafe {
                DOMAINS.prot[$pkey] = 0;
            },
            Err(_) => {
                panic!("FreeDomain");
            }
        }
    }};
    () => {};
}

macro_rules! ProtectDomain {
    ($addr:expr, $len:expr) => {{
        let pkey = GlobalDlmalloc::get_domain_id();
        if pkey != 0 {
            let prot = 3;
            let mut buf = [0x01, 0x20, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0];
            buf[2..6].copy_from_slice(&$addr.to_be_bytes());
            buf[6..10].copy_from_slice(&$len.to_be_bytes());
            buf[10] = prot as u8;
            buf[11] = pkey as u8;
            let result = wasicall!(&mut buf);
            match result {
                Ok(_) => {}
                Err(_) => {
                    panic!("domain_protect");
                }
            }
        }
    }};
}

macro_rules! mmapMemory {
    ($addr:expr, $len:expr, $prot:expr, $flags:expr) => {{
        let mut buf = [0x01, 0x2B, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0];
        buf[2..6].copy_from_slice(&$addr.to_be_bytes());
        buf[6..10].copy_from_slice(&$len.to_be_bytes());
        buf[10] = $prot;
        buf[11] = $flags;
        let result = wasicall!(&mut buf);
        match result {
            Ok(_) => {
                let map_addr = u32::from_be_bytes(
                    <[u8; 4]>::try_from(&buf[2..6]).expect("u32::from_be_bytes fails"),
                );
                map_addr as *mut u8
            }
            Err(_) => {
                panic!("mmap");
            }
        }
    }};
    () => {};
}
