use crate::wce::*;
use crate::WasiCtx;
use std::os::raw::c_void;
use wiggle::GuestPtr;

#[wiggle::async_trait]
impl wasi_snapshot_preview1::WasiSnapshotPreview1 for WasiCtx {
    fn wasi_for_dynlib(&self, len: u32, domain: u32, prot: u32) -> Result<(), Error> {
        unsafe {
            let mut ptr: *mut c_void = alloc(len as usize);
            let flags = 0;
            let domain = pkucreatedomain(flags);
            let pkey = pkupkeyalloc(flags, prot);
            setpkey(pkey, prot);
            let ret = pku_domain_assign_key(domain, pkey, flags, prot);
            if ret == -1 {
                // ???
            }
            let ret = pku_pkey_mprotect(ptr, len as usize, prot, flags);
            Ok(())
        }
    }
}
