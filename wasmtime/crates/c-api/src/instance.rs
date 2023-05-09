use crate::{
    wasm_extern_t, wasm_extern_vec_t, wasm_module_t, wasm_store_t, wasm_trap_t, wasmtime_error_t,
    wasmtime_extern_t, wasmtime_module_t, CStoreContextMut, StoreRef,
};
use std::mem::MaybeUninit;
use wasmtime::{Instance, Trap};

#[derive(Clone)]
pub struct wasm_instance_t {
    store: StoreRef,
    instance: Instance,
}

wasmtime_c_api_macros::declare_ref!(wasm_instance_t);

impl wasm_instance_t {
    pub(crate) fn new(store: StoreRef, instance: Instance) -> wasm_instance_t {
        wasm_instance_t { store, instance }
    }
}

#[no_mangle]
pub unsafe extern "C" fn wasm_instance_new(
    store: &mut wasm_store_t,
    wasm_module: &wasm_module_t,
    imports: *const wasm_extern_vec_t,
    result: Option<&mut *mut wasm_trap_t>,
) -> Option<Box<wasm_instance_t>> {
    let imports = (*imports)
        .as_slice()
        .iter()
        .filter_map(|import| match import {
            Some(i) => Some(i.which.clone()),
            None => None,
        })
        .collect::<Vec<_>>();
    match Instance::new(store.store.context_mut(), &wasm_module.module, &imports) {
        Ok(instance) => Some(Box::new(wasm_instance_t::new(
            store.store.clone(),
            instance,
        ))),
        Err(e) => {
            if let Some(ptr) = result {
                *ptr = Box::into_raw(Box::new(wasm_trap_t::new(e.into())));
            }
            None
        }
    }
}

#[no_mangle]
pub unsafe extern "C" fn wasm_instance_exports(
    instance: &mut wasm_instance_t,
    out: &mut wasm_extern_vec_t,
) {
    let store = instance.store.clone();
    out.set_buffer(
        instance
            .instance
            .exports(instance.store.context_mut())
            .map(|e| {
                Some(Box::new(wasm_extern_t {
                    which: e.into_extern(),
                    store: store.clone(),
                }))
            })
            .collect(),
    );
}

#[no_mangle]
pub unsafe extern "C" fn wasmtime_instance_new(
    store: CStoreContextMut<'_>,
    module: &wasmtime_module_t,
    imports: *const wasmtime_extern_t,
    nimports: usize,
    instance: &mut Instance,
    trap_ptr: &mut *mut wasm_trap_t,
) -> Option<Box<wasmtime_error_t>> {
    let imports = crate::slice_from_raw_parts(imports, nimports)
        .iter()
        .map(|i| i.to_extern())
        .collect::<Vec<_>>();
    handle_instantiate(
        Instance::new(store, &module.module, &imports),
        instance,
        trap_ptr,
    )
}

pub(crate) fn handle_instantiate(
    instance: anyhow::Result<Instance>,
    instance_ptr: &mut Instance,
    trap_ptr: &mut *mut wasm_trap_t,
) -> Option<Box<wasmtime_error_t>> {
    match instance {
        Ok(i) => {
            *instance_ptr = i;
            None
        }
        Err(e) => match e.downcast::<Trap>() {
            Ok(trap) => {
                *trap_ptr = Box::into_raw(Box::new(wasm_trap_t::new(trap)));
                None
            }
            Err(e) => Some(Box::new(e.into())),
        },
    }
}

#[no_mangle]
pub unsafe extern "C" fn wasmtime_instance_export_get(
    store: CStoreContextMut<'_>,
    instance: &Instance,
    name: *const u8,
    name_len: usize,
    item: &mut MaybeUninit<wasmtime_extern_t>,
) -> bool {
    let name = crate::slice_from_raw_parts(name, name_len);
    let name = match std::str::from_utf8(name) {
        Ok(name) => name,
        Err(_) => return false,
    };
    match instance.get_export(store, name) {
        Some(e) => {
            crate::initialize(item, e.into());
            true
        }
        None => false,
    }
}

#[no_mangle]
pub unsafe extern "C" fn wasmtime_instance_export_nth(
    store: CStoreContextMut<'_>,
    instance: &Instance,
    index: usize,
    name_ptr: &mut *const u8,
    name_len: &mut usize,
    item: &mut MaybeUninit<wasmtime_extern_t>,
) -> bool {
    match instance.exports(store).nth(index) {
        Some(e) => {
            *name_ptr = e.name().as_ptr();
            *name_len = e.name().len();
            crate::initialize(item, e.into_extern().into());
            true
        }
        None => false,
    }
}
