//! Memory isolation of pku

use crate::vmcontext::VMMemoryDefinition;
use libc::{self, SYS_pkey_alloc, SYS_pkey_mprotect};
use std::arch::asm;

/// A simple struct of pku
#[derive(Debug)]
pub struct Pku {}

const PKEY_DISABLE_ACCESS: i32 = 1;
const PKEY_DISABLE_WRITE: i32 = 2;

impl Pku {
    /// Construct a new init instance of 'Pku'.
    pub fn new() -> Self {
        Self {}
    }

    /// Change memory protection key protection permission on the specified page.
    pub fn pkey_mprotect(vm: &VMMemoryDefinition, start: usize, len: usize, prot: i64, pkey: i64) {
        unsafe {
            if libc::syscall(SYS_pkey_mprotect, vm.base.add(start), len, prot, pkey) == -1 {
                println!("error in libc::syscall SYS_pkey_mprotect");
            }
        }
    }

    /// Alloc memory protection key and set protection permission on the specified page.
    pub fn pkey_isolated(vm: &VMMemoryDefinition, start: usize, len: usize, prot: i64) -> i64 {
        unsafe {
            let pkey = libc::syscall(SYS_pkey_alloc, 0, 0);
            if pkey < 0 {
                println!("error in libc::syscall SYS_pkey_alloc");
            }
            Self::pkey_mprotect(vm, start, len, prot, pkey);
            return pkey;
        }
    }

    /// Read pkru value.
    pub fn rdpkru() -> i32 {
        let ecx = 0;
        let mut pkru: i32;

        unsafe {
            asm!(".byte 0x0f,0x01,0xee;",
                out("eax") pkru,
                in("ecx") ecx);
        }

        return pkru;
    }

    /// Write pkru value.
    pub fn wrpkru(pkru: i32) {
        let ecx = 0;
        let edx = 0;

        unsafe {
            asm!(".byte 0x0f,0x01,0xef;",
                in("eax") pkru,
                in("ecx") ecx,
                in("edx") edx);
        }
    }

    /// This will go out and modify PKRU register to set the access rights.
    pub fn set_pkey(pkey: i64, prot: i32) {
        let pkey_shift = pkey * 2;
        let mut new_pkru_bits = 0;

        if prot & PKEY_DISABLE_ACCESS != 0 {
            new_pkru_bits |= PKEY_DISABLE_ACCESS;
        }
        if prot & PKEY_DISABLE_WRITE != 0 {
            new_pkru_bits |= PKEY_DISABLE_WRITE;
        }

        /* Shift the bits in to the correct place in PKRU for pkey: */
        new_pkru_bits <<= pkey_shift;

        /* Get old PKRU and mask off any old bits in place: */
        let mut old_pkru = Pku::rdpkru();
        old_pkru &= !((PKEY_DISABLE_ACCESS | PKEY_DISABLE_WRITE) << pkey_shift);

        /* Write old part along with new part: */
        Pku::wrpkru(old_pkru | new_pkru_bits);
    }
}
