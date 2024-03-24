/// test native library function
pub(crate) fn raiden_add(a: u32, b: u32) -> u32 {
    a + b
}

pub fn raiden_call() {
    let filename = CString::new("/home/lhw/wasmpku/libpku/libraiden.so")
        .expect("CString::new path_string failed");
    let funcname = CString::new("CheckerPlusOneSimple").expect("CString::new path_string failed");
    let handle = unsafe { libc::dlopen(filename.as_ptr(), libc::RTLD_NOW) };
    if handle.is_null() {
        println!("libc::dlopen error");
    } else {
        let func_ptr = unsafe { libc::dlsym(handle, funcname.as_ptr()) };
        if func_ptr.is_null() {
            println!("libc::dlsym error");
        } else {
            unsafe {
                let func: unsafe fn(unsafe extern "C" fn() -> *mut libc::c_void) =
                    std::mem::transmute(func_ptr);
                func(get_memory);
            }
        }
    }
}
