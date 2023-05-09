use core::convert::TryFrom;
use GlobalDlmalloc;

/// pkey access prot
pub const PKEY_DISABLE_ACCESS: u32 = 1;
/// pkey write prot
pub const PKEY_DISABLE_WRITE: u32 = 2;

// static mut DOMAINS: [u32; 16] = [0; 16];

/// Pkey syscall interface
pub struct Domain {
    prot: [u32; 16],
    used: [u32; 16],
}

static mut DOMAINS: Domain = Domain::new();

static mut MMAP: *mut u8 = 0 as *mut u8;

impl Domain {
    /// constructor
    pub const fn new() -> Domain {
        Domain {
            prot: [0; 16],
            used: [0; 16],
        }
    }

    fn get_freed_domain() -> usize {
        unsafe {
            for i in 0..16 {
                if DOMAINS.used[i] == 2 {
                    return i;
                }
            }
            return 0;
        }
    }

    /// pkey_alloc
    #[cfg(target_arch = "wasm32")]
    pub fn create_domain(prot: u32) -> usize {
        let ret = CreateDomain!(prot);
        return ret;
    }

    /// pkey_alloc
    #[cfg(target_os = "linux")]
    pub fn create_domain(_prot: i32) -> usize {
        CreateDomain!();
        return 0;
    }

    /// pkey_free
    #[cfg(target_arch = "wasm32")]
    pub fn free_domain(pkey: usize) {
        FreeDomain!(pkey);
    }

    /// pkey_free
    #[cfg(target_os = "linux")]
    pub fn free_domain(_pkey: usize) {
        FreeDomain!();
    }

    /// Read pkru wasi
    pub fn read_pkru() -> u32 {
        let mut buf = [0x0F, 0x01, 0xEE, 0, 0, 0, 0, 0, 0, 0, 0, 0];
        let result = wasicall!(&mut buf);
        match result {
            Ok(_) => {
                let pkru = u32::from_be_bytes(
                    <[u8; 4]>::try_from(&buf[3..7]).expect("u32::from_be_bytes fails"),
                );
                return pkru;
            }
            Err(_) => {
                return 0;
            }
        }
    }

    fn write_pkru(pkru: u32) {
        let mut buf = [0x0F, 0x01, 0xEF, 0, 0, 0, 0, 0, 0, 0, 0, 0];
        buf[3..7].copy_from_slice(&pkru.to_be_bytes());
        let result = wasicall!(&mut buf);
        match result {
            Ok(_) => {}
            Err(_) => {}
        }
    }

    /// Read pkru
    pub fn rdpkru() -> u32 {
        let ecx = 0;
        return ecx;
    }

    fn wrpkru(pkru: u32) -> u32 {
        let eax = pkru;
        return eax;
    }

    fn set_pkey(pkey: usize, prot: u32) {
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
        let mut old_pkru = Self::rdpkru();
        if old_pkru == 0 {
            old_pkru = 0x55555554;
        }
        old_pkru &= !((PKEY_DISABLE_ACCESS | PKEY_DISABLE_WRITE) << pkey_shift);

        /* Write old part along with new part: */
        Self::wrpkru(old_pkru | new_pkru_bits);
    }

    /// trampline to isolated
    pub fn switch_domain(domain: usize, prot: u32) {
        Self::set_pkey(domain, prot);
        GlobalDlmalloc::switch_domain(domain);
    }

    /// trampline to normal
    pub fn restore_domain(source: usize, target: usize, prot: u32) {
        Self::set_pkey(source, prot);
        GlobalDlmalloc::switch_domain(target);
    }

    /// get domain prot
    pub fn get_domain_prot(domain: usize) -> u32 {
        unsafe { DOMAINS.prot[domain] }
    }

    /// pkey_mprotect
    pub fn domain_protect(addr: usize, len: usize) {
        ProtectDomain!(addr, len);
    }

    /// mmap
    #[cfg(target_arch = "wasm32")]
    pub fn mmap(addr: usize, len: usize, prot: u8, flags: u8) -> *mut u8 {
        unsafe {
            if MMAP != 0 as *mut u8 {
                return MMAP;
            }
        }
        return mmapMemory!(addr, len, prot, flags);
    }

    /// mmap
    #[cfg(target_os = "linux")]
    pub fn mmap(_addr: usize, _len: usize, _prot: u8, _flags: u8) -> *mut u8 {
        mmapMemory!();
        0 as *mut u8
    }
}
