use crate::commands::run::Host;
use anyhow::Result;
use wasmtime::Linker;

/// test module
pub mod ei;

use ei::*;

/// Define env function
pub fn define_raiden_function(linker: &mut Linker<Host>) -> Result<()> {
    linker.func_wrap("env", "RaidenAdd", raiden_add)?;
    Ok(())
}
