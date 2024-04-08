use crate::commands::run::Host;
use libc::{c_char, c_void};
use libffi::middle::*;
use once_cell::sync::Lazy;
use std::collections::HashMap;
use std::ffi::{CStr, CString};
use std::sync::OnceLock;
use std::{ptr, slice};
use wasmtime::{Caller, Instance, StoreContextMut};
use wasmtime_runtime::{InstanceHandle, Store, VMContext};

static mut FUNC_GOT: Lazy<HashMap<CString, CodePtr>> = Lazy::new(|| HashMap::new());
static mut LIB_MEMORY: OnceLock<LibraryMemory> = OnceLock::new();
static REGISTER_FLAG: OnceLock<bool> = OnceLock::new();

struct LibraryMemory {
    store: *mut dyn Store,
    caller: *mut VMContext,
}

impl LibraryMemory {
    pub fn new(caller: &Caller<'_, Host>) -> Self {
        Self {
            store: caller.get_store(),
            caller: caller.get_caller(),
        }
    }

    pub fn get_store_and_memory(&self) -> (*mut dyn Store, *mut VMContext) {
        (self.store, self.caller)
    }
}

/// Callback function for grow linear memory
pub unsafe extern "C" fn get_memory() -> *mut c_void {
    let ctx = LIB_MEMORY.get_mut().unwrap();
    let (store, caller) = ctx.get_store_and_memory();
    let instance = InstanceHandle::from_vmctx(caller);
    let mut storectx: StoreContextMut<'_, Host> = StoreContextMut::lhw_from_raw(store);
    let memory = instance
        .host_state()
        .downcast_ref::<Instance>()
        .unwrap()
        .get_export(&mut storectx, "memory")
        .unwrap()
        .into_memory()
        .unwrap();
    let page = memory.grow(&mut storectx, 1);
    match page {
        Ok(p) => {
            let base = memory.data_ptr(&storectx);
            let ret = base.add(p.try_into().unwrap());
            return ret as *mut c_void;
        }
        Err(e) => {
            println!("Error in get_memory() callback function: {e}");
            return ptr::null_mut() as *mut c_void;
        }
    }
}

/// get the global linear memory instance
pub fn set_memory(caller: &Caller<'_, Host>) {
    unsafe {
        LIB_MEMORY.get_or_init(|| LibraryMemory::new(caller));
    }
}

/// call the C function to init libc allocator
pub fn get_flag() {
    REGISTER_FLAG.get_or_init(|| {
        let filename = CString::new("/home/lhw/wasmpku/libpku/libpkulibc.so")
            .expect("CString::new path_string failed");
        let funcname =
            CString::new("RegisterMemoryRegion").expect("CString::new path_string failed");
        let handle = unsafe { libc::dlopen(filename.as_ptr(), libc::RTLD_NOW) };
        if handle.is_null() {
            println!("libc::dlopen error");
            return false;
        } else {
            let func_ptr = unsafe { libc::dlsym(handle, funcname.as_ptr()) };
            if func_ptr.is_null() {
                println!("libc::dlsym error");
                return false;
            } else {
                unsafe {
                    let func: unsafe fn(unsafe extern "C" fn() -> *mut libc::c_void) =
                        std::mem::transmute(func_ptr);
                    func(get_memory);
                }
            }
        }
        true
    });
}

/// find the native function which the wasm application wants to call
pub fn find_func(name: &CStr) -> Option<&CodePtr> {
    unsafe {
        let func_got = FUNC_GOT.get(name);
        match func_got {
            Some(f) => Some(f),
            None => None,
        }
    }
}

/// add the function to the global native function map
pub fn add_func(name: CString, func: *mut libc::c_void) {
    unsafe {
        let func_ptr = CodePtr(func);
        FUNC_GOT.insert(name, func_ptr);
    }
}

/// call the native function
pub fn native_library_call(
    mut caller: Caller<'_, Host>,
    lib_name: u32,
    func_name: u32,
    args_num: u32,
    ret_type: u32,
    args_type: u32,
    args_value: u32,
    ret: u32,
) -> i32 {
    set_memory(&caller);
    get_flag();

    let memory = caller.get_export("memory").unwrap().into_memory().unwrap();

    let linear_memory: &[u8] = memory.data(&caller);

    let base = memory.data_ptr(&caller);

    let func: CodePtr;
    unsafe {
        let lib_string: *const c_char = linear_memory.as_ptr().add(lib_name as usize).cast();
        let func_string: *const c_char = linear_memory.as_ptr().add(func_name as usize).cast();

        let libname = CStr::from_ptr(lib_string);
        let funcname = CStr::from_ptr(func_string);

        let func_got = FUNC_GOT.get(funcname);
        match func_got {
            Some(f) => {
                func = *f;
            }
            None => {
                let handle = libc::dlopen(libname.as_ptr(), libc::RTLD_NOW);
                if handle.is_null() {
                    return -1;
                }
                func = CodePtr(libc::dlsym(handle, funcname.as_ptr()));
                FUNC_GOT.insert(funcname.into(), func);
            }
        }
    }

    let args_type_ptr: *const u32 =
        unsafe { linear_memory.as_ptr().add(args_type as usize).cast() };
    let args_ptr: *const *mut u8 =
        unsafe { linear_memory.as_ptr().add(args_value as usize).cast() };

    let args_type_slice = unsafe { slice::from_raw_parts(args_type_ptr, args_num as usize) };
    let args_slice = unsafe { slice::from_raw_parts(args_ptr, args_num as usize) };

    let mut args: Vec<Type> = Vec::with_capacity(args_num.try_into().unwrap());
    let mut argv: Vec<Arg> = Vec::with_capacity(args_num.try_into().unwrap());

    for ((_, arg_type), argi) in args_type_slice.iter().enumerate().zip(args_slice.iter()) {
        unsafe {
            match *arg_type {
                libffi::raw::FFI_TYPE_UINT8 => {
                    let addr = base.add(*argi as usize);
                    args.push(Type::u8());
                    argv.push(arg(&*addr));
                }
                libffi::raw::FFI_TYPE_SINT8 => {
                    let addr = base.add(*argi as usize) as *mut i8;
                    args.push(Type::i8());
                    argv.push(arg(&*addr));
                }
                libffi::raw::FFI_TYPE_UINT16 => {
                    let addr = base.add(*argi as usize) as *mut u16;
                    args.push(Type::u16());
                    argv.push(arg(&*addr));
                }
                libffi::raw::FFI_TYPE_SINT16 => {
                    let addr = base.add(*argi as usize) as *mut i16;
                    args.push(Type::i16());
                    argv.push(arg(&*addr));
                }
                libffi::raw::FFI_TYPE_UINT32 => {
                    let addr = base.add(*argi as usize) as *mut u32;
                    args.push(Type::u32());
                    argv.push(arg(&*addr));
                }
                libffi::raw::FFI_TYPE_POINTER => {
                    let addr = base.add(*argi as usize);
                    args.push(Type::pointer());
                    argv.push(arg(&addr));
                }
                libffi::raw::FFI_TYPE_SINT32 | libffi::raw::FFI_TYPE_INT => {
                    let addr = base.add(*argi as usize) as *mut i32;
                    args.push(Type::i32());
                    argv.push(arg(&*addr));
                }
                libffi::raw::FFI_TYPE_UINT64 => {
                    let addr = base.add(*argi as usize) as *mut u64;
                    args.push(Type::u64());
                    argv.push(arg(&*addr));
                }
                libffi::raw::FFI_TYPE_SINT64 => {
                    let addr = base.add(*argi as usize) as *mut i64;
                    args.push(Type::i64());
                    argv.push(arg(&*addr));
                }
                libffi::raw::FFI_TYPE_FLOAT => {
                    let addr = base.add(*argi as usize) as *mut f32;
                    args.push(Type::f32());
                    argv.push(arg(&*addr));
                }
                libffi::raw::FFI_TYPE_DOUBLE => {
                    let addr = base.add(*argi as usize) as *mut f64;
                    args.push(Type::f64());
                    argv.push(arg(&*addr));
                }
                _ => {
                    println!("Arguments FFI type not yet implemented: {}", arg_type);
                }
            }
        }
    }

    let return_type: Type;
    match ret_type {
        libffi::raw::FFI_TYPE_UINT8 => return_type = Type::u8(),
        libffi::raw::FFI_TYPE_SINT8 => return_type = Type::i8(),
        libffi::raw::FFI_TYPE_UINT16 => return_type = Type::u16(),
        libffi::raw::FFI_TYPE_SINT16 => return_type = Type::i16(),
        libffi::raw::FFI_TYPE_UINT32 => return_type = Type::u32(),
        libffi::raw::FFI_TYPE_POINTER => return_type = Type::pointer(),
        libffi::raw::FFI_TYPE_SINT32 | libffi::raw::FFI_TYPE_INT => return_type = Type::i32(),
        libffi::raw::FFI_TYPE_UINT64 => return_type = Type::u64(),
        libffi::raw::FFI_TYPE_SINT64 => return_type = Type::i64(),
        libffi::raw::FFI_TYPE_FLOAT => return_type = Type::f32(),
        libffi::raw::FFI_TYPE_DOUBLE => return_type = Type::f64(),
        _ => {
            println!("Return FFI type not yet implemented: {}", ret_type);
            return -1;
        }
    }

    let cif = Cif::new(args.into_iter(), return_type);
    unsafe {
        let ret_raw: *mut c_void = cif.call(func, argv.as_slice());
        let ret_ptr: *const usize = linear_memory.as_ptr().add(ret as usize).cast();
        let ret_write = ret_ptr as *mut usize;
        let ptr = std::mem::transmute(ret_raw);
        ret_write.write(ptr);
    }

    return 0;
}

/// A simple raiden call
pub fn raiden_call() {
    let filename = CString::new("/home/lhw/orbit/userlib/build/lib/libraiden.so")
        .expect("CString::new path_string failed");
    let funcname = CString::new("TestRaidenFunc").expect("CString::new path_string failed");
    let func: CodePtr;
    unsafe {
        let func_got = FUNC_GOT.get(&funcname);
        match func_got {
            Some(f) => {
                func = *f;
            }
            None => {
                let handle = libc::dlopen(filename.as_ptr(), libc::RTLD_NOW);
                if handle.is_null() {
                    println!("libc::dlopen error");
                    return;
                } else {
                    func = CodePtr(libc::dlsym(handle, funcname.as_ptr()));
                    FUNC_GOT.insert(funcname.into(), func);
                }
            }
        }
        let func_ptr = func.as_safe_fun();
        // let func: unsafe fn() = std::mem::transmute(func_ptr);
        func_ptr();
    }
}

/// test native library function
pub(crate) fn raiden_test() {
    raiden_call()
}
