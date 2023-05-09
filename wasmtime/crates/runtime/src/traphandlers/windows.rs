use crate::traphandlers::{tls, wasmtime_longjmp};
use std::io;
use windows_sys::Win32::Foundation::*;
use windows_sys::Win32::System::Diagnostics::Debug::*;
use windows_sys::Win32::System::Kernel::*;

/// Function which may handle custom signals while processing traps.
pub type SignalHandler<'a> = dyn Fn(*mut EXCEPTION_POINTERS) -> bool + Send + Sync + 'a;

pub unsafe fn platform_init() {
    // our trap handler needs to go first, so that we can recover from
    // wasm faults and continue execution, so pass `1` as a true value
    // here.
    if AddVectoredExceptionHandler(1, Some(exception_handler)).is_null() {
        panic!(
            "failed to add exception handler: {}",
            io::Error::last_os_error()
        );
    }
}

unsafe extern "system" fn exception_handler(exception_info: *mut EXCEPTION_POINTERS) -> i32 {
    // Check the kind of exception, since we only handle a subset within
    // wasm code. If anything else happens we want to defer to whatever
    // the rest of the system wants to do for this exception.
    let record = &*(*exception_info).ExceptionRecord;
    if record.ExceptionCode != EXCEPTION_ACCESS_VIOLATION
        && record.ExceptionCode != EXCEPTION_ILLEGAL_INSTRUCTION
        && record.ExceptionCode != EXCEPTION_INT_DIVIDE_BY_ZERO
        && record.ExceptionCode != EXCEPTION_INT_OVERFLOW
    {
        return ExceptionContinueSearch;
    }

    // FIXME: this is what the previous C++ did to make sure that TLS
    // works by the time we execute this trap handling code. This isn't
    // exactly super easy to call from Rust though and it's not clear we
    // necessarily need to do so. Leaving this here in case we need this
    // in the future, but for now we can probably wait until we see a
    // strange fault before figuring out how to reimplement this in
    // Rust.
    //
    // if (!NtCurrentTeb()->Reserved1[sThreadLocalArrayPointerIndex]) {
    //     return ExceptionContinueSearch;
    // }

    // This is basically the same as the unix version above, only with a
    // few parameters tweaked here and there.
    tls::with(|info| {
        let info = match info {
            Some(info) => info,
            None => return ExceptionContinueSearch,
        };
        cfg_if::cfg_if! {
            if #[cfg(target_arch = "x86_64")] {
                let ip = (*(*exception_info).ContextRecord).Rip as *const u8;
                let fp = (*(*exception_info).ContextRecord).Rbp as usize;
            } else if #[cfg(target_arch = "x86")] {
                let ip = (*(*exception_info).ContextRecord).Eip as *const u8;
                let fp = (*(*exception_info).ContextRecord).Ebp as usize;
            } else {
                compile_error!("unsupported platform");
            }
        }
        let jmp_buf = info.jmp_buf_if_trap(ip, |handler| handler(exception_info));
        if jmp_buf.is_null() {
            ExceptionContinueSearch
        } else if jmp_buf as usize == 1 {
            ExceptionContinueExecution
        } else {
            info.set_jit_trap(ip, fp);
            wasmtime_longjmp(jmp_buf)
        }
    })
}

pub fn lazy_per_thread_init() {
    // Unused on Windows
}
