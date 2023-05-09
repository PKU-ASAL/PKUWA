#[cfg(target_arch = "wasm32")]
use core::arch::wasm32 as wasm;
#[cfg(target_arch = "wasm64")]
use core::arch::wasm64 as wasm;
use core::ptr;
use Allocator;

use Domain;

struct MonitorMemory {
    ptr: *mut u8,
    size: usize,
    prev: *mut MonitorMemory,
    next: *mut MonitorMemory,
}

impl MonitorMemory {
    pub fn new(ptr: *mut u8, size: usize) -> *mut MonitorMemory {
        let mut memory = ptr as *mut MonitorMemory;
        unsafe {
            (*memory).ptr = ptr;
            (*memory).size = size;
            (*memory).prev = memory;
            (*memory).next = memory;
        }
        memory
    }

    pub fn set_prev(&mut self, ptr: *mut MonitorMemory) {
        self.prev = ptr;
    }

    pub fn set_next(&mut self, ptr: *mut MonitorMemory) {
        self.next = ptr;
    }

    pub fn get_prev(&self) -> *mut MonitorMemory {
        self.prev
    }

    pub fn get_next(&self) -> *mut MonitorMemory {
        self.next
    }

    pub fn get_ptr(&self) -> *mut u8 {
        self.ptr
    }

    pub fn get_size(&self) -> usize {
        self.size
    }
}

/// Monitor is a PKU system manager
pub struct Monitor {
    footprint: usize,
    head: *mut MonitorMemory,
}

impl Monitor {
    /// constructor
    pub const fn new() -> Monitor {
        Monitor {
            footprint: 0,
            head: ptr::null_mut(),
        }
    }

    /// insert a node to a list
    pub fn insert(&mut self, ptr: *mut u8, size: usize) {
        let memory = MonitorMemory::new(ptr, size);
        if self.head == ptr::null_mut() {
            self.head = memory;
        } else {
            unsafe {
                (*memory).set_next(self.head);
                (*memory).set_prev((*self.head).get_prev());
                (*(*self.head).get_prev()).set_next(memory);
                (*self.head).set_prev(memory);
            }
            self.head = memory;
        }
        self.footprint += size;
    }

    /// delete a head node in a list
    pub fn delete(&mut self) -> (*mut u8, usize) {
        if self.head != ptr::null_mut() {
            let temp = self.head;
            unsafe {
                if (*self.head).get_next() == self.head {
                    self.head = ptr::null_mut();
                } else {
                    self.head = (*self.head).get_next();
                    (*(*temp).get_next()).set_prev((*temp).get_prev());
                    (*(*temp).get_prev()).set_next((*temp).get_next());
                    (*temp).set_next(ptr::null_mut());
                    (*temp).set_prev(ptr::null_mut());
                }
                self.footprint -= (*temp).get_size();
                ((*temp).get_ptr(), (*temp).get_size())
            }
        } else {
            (ptr::null_mut(), 0)
        }
    }

    /// member function of Monitor struct
    pub fn get_footprint(&self) -> usize {
        self.footprint
    }

    /// non-member function for global Monitor struct
    pub unsafe fn monitor_footprint() -> usize {
        MONITOR.get_footprint()
    }
}

static mut MONITOR: Monitor = Monitor::new();

/// System setting for Wasm
pub struct System {
    _priv: (),
}

impl System {
    pub const fn new() -> System {
        System { _priv: () }
    }
}

unsafe impl Allocator for System {
    fn alloc(&self, size: usize) -> (*mut u8, usize, u32) {
        // unsafe {
        //     let (ptr, psize) = MONITOR.delete();
        //     if ptr != ptr::null_mut() && psize != 0 {
        //         return (ptr, psize, 0);
        //     }
        // }
        let pages = size / self.page_size();
        let prev = wasm::memory_grow(0, pages);
        if prev == usize::max_value() {
            return (ptr::null_mut(), 0, 0);
        }
        Domain::domain_protect(prev * self.page_size(), pages * self.page_size());
        (
            (prev * self.page_size()) as *mut u8,
            pages * self.page_size(),
            0,
        )
    }

    fn remap(&self, _ptr: *mut u8, _oldsize: usize, _newsize: usize, _can_move: bool) -> *mut u8 {
        // TODO: I think this can be implemented near the end?
        ptr::null_mut()
    }

    fn free_part(&self, _ptr: *mut u8, _oldsize: usize, _newsize: usize) -> bool {
        false
    }

    fn free(&self, _ptr: *mut u8, _size: usize) -> bool {
        false
        // unsafe {
        //     MONITOR.insert(ptr, size);
        //     true
        // }
    }

    fn can_release_part(&self, _flags: u32) -> bool {
        false
    }

    fn allocates_zeros(&self) -> bool {
        true
    }

    fn page_size(&self) -> usize {
        64 * 1024
    }
}

#[cfg(feature = "global")]
pub fn acquire_global_lock() {
    // single threaded, no need!
}

#[cfg(feature = "global")]
pub fn release_global_lock() {
    // single threaded, no need!
}

#[cfg(feature = "global")]
/// TODO
pub unsafe fn enable_alloc_after_fork() {
    // single threaded, no need!
}
