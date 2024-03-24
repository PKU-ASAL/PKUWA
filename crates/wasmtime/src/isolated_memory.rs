//! Memory isolation

use std::cell::RefCell;
use wasmtime_runtime::Pku;
use wasmtime_runtime::VMMemoryDefinition;

/// A simple struct of isolated memory
#[derive(Debug)]
pub struct IsolatedMomery {}

thread_local!(static MOMERY: RefCell<Vec<MomeryMap>> = RefCell::new(Vec::new()));

impl IsolatedMomery {
    /// Construct a new init instance of 'IsolatedMomery'.
    pub fn new() -> Self {
        Self {}
    }

    /// Crate memory domain.
    pub fn hook_domain(vm: &VMMemoryDefinition, start: usize, len: usize, prot: i64) {
        let pkey = Pku::pkey_isolated(vm, start, len, prot);
        MOMERY.with(|m| {
            m.borrow_mut().push(MomeryMap::new(start, len, prot, pkey));
        });
    }

    /// Redo memory isolation, becouse wasmtime grow function mmap wasm memory in new virtual memory range.
    pub fn hook_memory(vm: &VMMemoryDefinition) {
        MOMERY.with(|m| {
            for iter in m.take() {
                let mmap = iter.get();
                Pku::pkey_mprotect(vm, mmap.0, mmap.1, mmap.2, mmap.3);
            }
        });
    }

    /// Transition to isolated domain.
    pub fn hook_isolated_domain(pkey: i64) {
        Pku::set_pkey(pkey, 0);
    }

    /// Transition to normal domain.
    pub fn hook_normal_domain(pkey: i64) {
        let prot: i32 = 0x1 | 0x2;
        Pku::set_pkey(pkey, prot);
    }

    /// Trampline to transition domain.
    pub fn hook_transition(pkey: i64, prot: i32) {
        Pku::set_pkey(pkey, prot);
    }
}

#[derive(Debug)]
pub struct MomeryMap {
    offset: usize,
    len: usize,
    prot: i64,
    pkey: i64,
}

impl MomeryMap {
    pub fn new(offset: usize, len: usize, prot: i64, pkey: i64) -> Self {
        Self {
            offset,
            len,
            prot,
            pkey,
        }
    }

    pub fn get(&self) -> (usize, usize, i64, i64) {
        (self.offset, self.len, self.prot, self.pkey)
    }
}
