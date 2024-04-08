use crate::commands::run::Host;
use anyhow::Result;
use wasmtime::Linker;

/// test module
pub mod ei;

use ei::*;

/// Define env function
pub fn define_raiden_function(linker: &mut Linker<Host>) -> Result<()> {
    linker.func_wrap("env", "NativeLibraryCall", native_library_call)?;
    linker.func_wrap("env", "RaidenTest", raiden_test)?;
    Ok(())
}
