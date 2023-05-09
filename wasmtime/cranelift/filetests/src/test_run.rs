//! Test command for running CLIF files and verifying their results
//!
//! The `run` test command compiles each function on the host machine and executes it

use crate::function_runner::{CompiledTestFile, TestFileCompiler};
use crate::runone::FileUpdate;
use crate::runtest_environment::{HeapMemory, RuntestEnvironment};
use crate::subtest::{Context, SubTest};
use anyhow::Context as _;
use cranelift_codegen::data_value::DataValue;
use cranelift_codegen::ir::Type;
use cranelift_codegen::isa::TargetIsa;
use cranelift_codegen::settings::{Configurable, Flags};
use cranelift_codegen::{ir, settings};
use cranelift_reader::TestCommand;
use cranelift_reader::{parse_run_command, TestFile};
use log::{info, trace};
use std::borrow::Cow;

struct TestRun;

pub fn subtest(parsed: &TestCommand) -> anyhow::Result<Box<dyn SubTest>> {
    assert_eq!(parsed.command, "run");
    if !parsed.options.is_empty() {
        anyhow::bail!("No options allowed on {}", parsed);
    }
    Ok(Box::new(TestRun))
}

/// Builds a [TargetIsa] for the current host.
///
/// ISA Flags can be overridden by passing [Value]'s via `isa_flags`.
fn build_host_isa(
    infer_native_flags: bool,
    flags: settings::Flags,
    isa_flags: Vec<settings::Value>,
) -> Box<dyn TargetIsa> {
    let mut builder = cranelift_native::builder_with_options(infer_native_flags)
        .expect("Unable to build a TargetIsa for the current host");

    // Copy ISA Flags
    for value in isa_flags {
        builder.set(value.name, &value.value_string()).unwrap();
    }

    builder.finish(flags).unwrap()
}

/// Checks if the host's ISA is compatible with the one requested by the test.
fn is_isa_compatible(
    file_path: &str,
    host: &dyn TargetIsa,
    requested: &dyn TargetIsa,
) -> Result<(), String> {
    // If this test requests to run on a completely different
    // architecture than the host platform then we skip it entirely,
    // since we won't be able to natively execute machine code.
    let host_arch = host.triple().architecture;
    let requested_arch = requested.triple().architecture;
    if host_arch != requested_arch {
        return Err(format!(
            "skipped {}: host can't run {:?} programs",
            file_path, requested_arch
        ));
    }

    // We need to check that the requested ISA does not have any flags that
    // we can't natively support on the host.
    let requested_flags = requested.isa_flags();
    for req_value in requested_flags {
        if let Some(requested) = req_value.as_bool() {
            let available_in_host = host
                .isa_flags()
                .iter()
                .find(|val| val.name == req_value.name)
                .and_then(|val| val.as_bool())
                .unwrap_or(false);

            if requested && !available_in_host {
                return Err(format!(
                    "skipped {}: host does not support ISA flag {}",
                    file_path, req_value.name
                ));
            }
        } else {
            unimplemented!("ISA flag {} of kind {:?}", req_value.name, req_value.kind());
        }
    }

    Ok(())
}

fn compile_testfile(
    testfile: &TestFile,
    flags: &Flags,
    isa: &dyn TargetIsa,
) -> anyhow::Result<CompiledTestFile> {
    // We can't use the requested ISA directly since it does not contain info
    // about the operating system / calling convention / etc..
    //
    // Copy the requested ISA flags into the host ISA and use that.
    let isa = build_host_isa(false, flags.clone(), isa.isa_flags());

    let mut tfc = TestFileCompiler::new(isa);
    tfc.add_testfile(testfile)?;
    Ok(tfc.compile()?)
}

fn run_test(
    testfile: &CompiledTestFile,
    func: &ir::Function,
    context: &Context,
) -> anyhow::Result<()> {
    let test_env = RuntestEnvironment::parse(&context.details.comments[..])?;

    for comment in context.details.comments.iter() {
        if let Some(command) = parse_run_command(comment.text, &func.signature)? {
            trace!("Parsed run command: {}", command);

            command
                .run(|_, run_args| {
                    test_env.validate_signature(&func)?;
                    let (_heaps, _ctx_struct, vmctx_ptr) =
                        build_vmctx_struct(&test_env, context.isa.unwrap().pointer_type());

                    let mut args = Vec::with_capacity(run_args.len());
                    if test_env.is_active() {
                        args.push(vmctx_ptr);
                    }
                    args.extend_from_slice(run_args);

                    let trampoline = testfile.get_trampoline(func).unwrap();
                    Ok(trampoline.call(&args))
                })
                .map_err(|s| anyhow::anyhow!("{}", s))?;
        }
    }
    Ok(())
}

impl SubTest for TestRun {
    fn name(&self) -> &'static str {
        "run"
    }

    fn is_mutating(&self) -> bool {
        false
    }

    fn needs_isa(&self) -> bool {
        true
    }

    /// Runs the entire subtest for a given target, invokes [Self::run] for running
    /// individual tests.
    fn run_target<'a>(
        &self,
        testfile: &TestFile,
        file_update: &mut FileUpdate,
        file_path: &'a str,
        flags: &'a Flags,
        isa: Option<&'a dyn TargetIsa>,
    ) -> anyhow::Result<()> {
        // Disable runtests with pinned reg enabled.
        // We've had some abi issues that the trampoline isn't quite ready for.
        if flags.enable_pinned_reg() {
            return Err(anyhow::anyhow!([
                "Cannot run runtests with pinned_reg enabled.",
                "See https://github.com/bytecodealliance/wasmtime/issues/4376 for more info"
            ]
            .join("\n")));
        }

        // Check that the host machine can run this test case (i.e. has all extensions)
        let host_isa = build_host_isa(true, flags.clone(), vec![]);
        if let Err(e) = is_isa_compatible(file_path, host_isa.as_ref(), isa.unwrap()) {
            log::info!("{}", e);
            return Ok(());
        }

        let compiled_testfile = compile_testfile(&testfile, flags, isa.unwrap())?;

        for (func, details) in &testfile.functions {
            info!(
                "Test: {}({}) {}",
                self.name(),
                func.name,
                isa.map_or("-", TargetIsa::name)
            );

            let context = Context {
                preamble_comments: &testfile.preamble_comments,
                details,
                flags,
                isa,
                file_path: file_path.as_ref(),
                file_update,
            };

            run_test(&compiled_testfile, &func, &context).context(self.name())?;
        }

        Ok(())
    }

    fn run(&self, _func: Cow<ir::Function>, _context: &Context) -> anyhow::Result<()> {
        unreachable!()
    }
}

/// Build a VMContext struct with the layout described in docs/testing.md.
pub fn build_vmctx_struct(
    test_env: &RuntestEnvironment,
    ptr_ty: Type,
) -> (Vec<HeapMemory>, Vec<u64>, DataValue) {
    let heaps = test_env.allocate_memory();

    let context_struct: Vec<u64> = heaps
        .iter()
        .flat_map(|heap| [heap.as_ptr(), heap.as_ptr().wrapping_add(heap.len())])
        .map(|p| p as usize as u64)
        .collect();

    let ptr = context_struct.as_ptr() as usize as i128;
    let ptr_dv =
        DataValue::from_integer(ptr, ptr_ty).expect("Failed to cast pointer to native target size");

    // Return all these to make sure we don't deallocate the heaps too early
    (heaps, context_struct, ptr_dv)
}
