//! The module that implements the `wasmtime compile` command.

use anyhow::{bail, Context, Result};
use clap::Parser;
use once_cell::sync::Lazy;
use std::fs;
use std::path::PathBuf;
use target_lexicon::Triple;
use wasmtime::Engine;
use wasmtime_cli_flags::CommonOptions;

static AFTER_HELP: Lazy<String> = Lazy::new(|| {
    format!(
        "By default, no CPU features or presets will be enabled for the compilation.\n\
        \n\
        {}\
        \n\
        Usage examples:\n\
        \n\
        Compiling a WebAssembly module for the current platform:\n\
        \n  \
        wasmtime compile example.wasm
        \n\
        Specifying the output file:\n\
        \n  \
        wasmtime compile -o output.cwasm input.wasm\n\
        \n\
        Compiling for a specific platform (Linux) and CPU preset (Skylake):\n\
        \n  \
        wasmtime compile --target x86_64-unknown-linux --cranelift-enable skylake foo.wasm\n",
        crate::FLAG_EXPLANATIONS.as_str()
    )
});

/// Compiles a WebAssembly module.
#[derive(Parser)]
#[structopt(
    name = "compile",
    version,
    after_help = AFTER_HELP.as_str()
)]
pub struct CompileCommand {
    #[clap(flatten)]
    common: CommonOptions,

    /// The target triple; default is the host triple
    #[clap(long, value_name = "TARGET")]
    target: Option<String>,

    /// The path of the output compiled module; defaults to <MODULE>.cwasm
    #[clap(short = 'o', long, value_name = "OUTPUT", parse(from_os_str))]
    output: Option<PathBuf>,

    /// The path of the WebAssembly to compile
    #[clap(index = 1, value_name = "MODULE", parse(from_os_str))]
    module: PathBuf,
}

impl CompileCommand {
    /// Executes the command.
    pub fn execute(mut self) -> Result<()> {
        self.common.init_logging();

        let target = self
            .target
            .take()
            .unwrap_or_else(|| Triple::host().to_string());

        let config = self.common.config(Some(&target))?;

        let engine = Engine::new(&config)?;

        if self.module.file_name().is_none() {
            bail!(
                "'{}' is not a valid input module path",
                self.module.display()
            );
        }

        let input = fs::read(&self.module).with_context(|| "failed to read input file")?;

        let output = self.output.take().unwrap_or_else(|| {
            let mut output: PathBuf = self.module.file_name().unwrap().into();
            output.set_extension("cwasm");
            output
        });

        fs::write(output, engine.precompile_module(&input)?)?;

        Ok(())
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;
    use wasmtime::{Instance, Module, Store};

    #[test]
    fn test_successful_compile() -> Result<()> {
        let (mut input, input_path) = NamedTempFile::new()?.into_parts();
        input.write_all(
            "(module (func (export \"f\") (param i32) (result i32) local.get 0))".as_bytes(),
        )?;
        drop(input);

        let output_path = NamedTempFile::new()?.into_temp_path();

        let command = CompileCommand::try_parse_from(vec![
            "compile",
            "--disable-logging",
            "-o",
            output_path.to_str().unwrap(),
            input_path.to_str().unwrap(),
        ])?;

        command.execute()?;

        let engine = Engine::default();
        let contents = std::fs::read(output_path)?;
        let module = unsafe { Module::deserialize(&engine, contents)? };
        let mut store = Store::new(&engine, ());
        let instance = Instance::new(&mut store, &module, &[])?;
        let f = instance.get_typed_func::<i32, i32, _>(&mut store, "f")?;
        assert_eq!(f.call(&mut store, 1234).unwrap(), 1234);

        Ok(())
    }

    #[cfg(target_arch = "x86_64")]
    #[test]
    fn test_x64_flags_compile() -> Result<()> {
        let (mut input, input_path) = NamedTempFile::new()?.into_parts();
        input.write_all("(module)".as_bytes())?;
        drop(input);

        let output_path = NamedTempFile::new()?.into_temp_path();

        // Set all the x64 flags to make sure they work
        let command = CompileCommand::try_parse_from(vec![
            "compile",
            "--disable-logging",
            "--cranelift-enable",
            "has_sse3",
            "--cranelift-enable",
            "has_ssse3",
            "--cranelift-enable",
            "has_sse41",
            "--cranelift-enable",
            "has_sse42",
            "--cranelift-enable",
            "has_avx",
            "--cranelift-enable",
            "has_avx2",
            "--cranelift-enable",
            "has_fma",
            "--cranelift-enable",
            "has_avx512dq",
            "--cranelift-enable",
            "has_avx512vl",
            "--cranelift-enable",
            "has_avx512f",
            "--cranelift-enable",
            "has_popcnt",
            "--cranelift-enable",
            "has_bmi1",
            "--cranelift-enable",
            "has_bmi2",
            "--cranelift-enable",
            "has_lzcnt",
            "-o",
            output_path.to_str().unwrap(),
            input_path.to_str().unwrap(),
        ])?;

        command.execute()?;

        Ok(())
    }

    #[cfg(target_arch = "aarch64")]
    #[test]
    fn test_aarch64_flags_compile() -> Result<()> {
        let (mut input, input_path) = NamedTempFile::new()?.into_parts();
        input.write_all("(module)".as_bytes())?;
        drop(input);

        let output_path = NamedTempFile::new()?.into_temp_path();

        // Set all the aarch64 flags to make sure they work
        let command = CompileCommand::try_parse_from(vec![
            "compile",
            "--disable-logging",
            "--cranelift-enable",
            "has_lse",
            "--cranelift-enable",
            "has_pauth",
            "--cranelift-enable",
            "sign_return_address",
            "--cranelift-enable",
            "sign_return_address_all",
            "--cranelift-enable",
            "sign_return_address_with_bkey",
            "-o",
            output_path.to_str().unwrap(),
            input_path.to_str().unwrap(),
        ])?;

        command.execute()?;

        Ok(())
    }

    #[cfg(target_arch = "x86_64")]
    #[test]
    fn test_unsupported_flags_compile() -> Result<()> {
        let (mut input, input_path) = NamedTempFile::new()?.into_parts();
        input.write_all("(module)".as_bytes())?;
        drop(input);

        let output_path = NamedTempFile::new()?.into_temp_path();

        // aarch64 flags should not be supported
        let command = CompileCommand::try_parse_from(vec![
            "compile",
            "--disable-logging",
            "--cranelift-enable",
            "has_lse",
            "-o",
            output_path.to_str().unwrap(),
            input_path.to_str().unwrap(),
        ])?;

        assert_eq!(
            command.execute().unwrap_err().to_string(),
            "No existing setting named 'has_lse'"
        );

        Ok(())
    }

    #[cfg(target_arch = "x86_64")]
    #[test]
    fn test_x64_presets_compile() -> Result<()> {
        let (mut input, input_path) = NamedTempFile::new()?.into_parts();
        input.write_all("(module)".as_bytes())?;
        drop(input);

        let output_path = NamedTempFile::new()?.into_temp_path();

        for preset in &[
            "nehalem",
            "haswell",
            "broadwell",
            "skylake",
            "cannonlake",
            "icelake",
            "znver1",
        ] {
            let command = CompileCommand::try_parse_from(vec![
                "compile",
                "--disable-logging",
                "--cranelift-enable",
                preset,
                "-o",
                output_path.to_str().unwrap(),
                input_path.to_str().unwrap(),
            ])?;

            command.execute()?;
        }

        Ok(())
    }
}
