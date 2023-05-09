//! Oracles.
//!
//! Oracles take a test case and determine whether we have a bug. For example,
//! one of the simplest oracles is to take a Wasm binary as our input test case,
//! validate and instantiate it, and (implicitly) check that no assertions
//! failed or segfaults happened. A more complicated oracle might compare the
//! result of executing a Wasm file with and without optimizations enabled, and
//! make sure that the two executions are observably identical.
//!
//! When an oracle finds a bug, it should report it to the fuzzing engine by
//! panicking.

#[cfg(feature = "fuzz-spec-interpreter")]
pub mod diff_spec;
pub mod diff_wasmi;
pub mod diff_wasmtime;
pub mod dummy;
pub mod engine;
mod stacks;

use self::diff_wasmtime::WasmtimeInstance;
use self::engine::{DiffEngine, DiffInstance};
use crate::generators::{self, DiffValue, DiffValueType};
use arbitrary::Arbitrary;
pub use stacks::check_stacks;
use std::cell::Cell;
use std::rc::Rc;
use std::sync::atomic::{AtomicUsize, Ordering::SeqCst};
use std::sync::{Arc, Condvar, Mutex};
use std::time::{Duration, Instant};
use wasmtime::*;
use wasmtime_wast::WastContext;

#[cfg(not(any(windows, target_arch = "s390x")))]
mod diff_v8;

static CNT: AtomicUsize = AtomicUsize::new(0);

/// Logs a wasm file to the filesystem to make it easy to figure out what wasm
/// was used when debugging.
pub fn log_wasm(wasm: &[u8]) {
    super::init_fuzzing();

    if !log::log_enabled!(log::Level::Debug) {
        return;
    }

    let i = CNT.fetch_add(1, SeqCst);
    let name = format!("testcase{}.wasm", i);
    std::fs::write(&name, wasm).expect("failed to write wasm file");
    log::debug!("wrote wasm file to `{}`", name);
    let wat = format!("testcase{}.wat", i);
    match wasmprinter::print_bytes(wasm) {
        Ok(s) => std::fs::write(&wat, s).expect("failed to write wat file"),
        // If wasmprinter failed remove a `*.wat` file, if any, to avoid
        // confusing a preexisting one with this wasm which failed to get
        // printed.
        Err(_) => drop(std::fs::remove_file(&wat)),
    }
}

/// The `T` in `Store<T>` for fuzzing stores, used to limit resource
/// consumption during fuzzing.
#[derive(Clone)]
pub struct StoreLimits(Rc<LimitsState>);

struct LimitsState {
    /// Remaining memory, in bytes, left to allocate
    remaining_memory: Cell<usize>,
    /// Whether or not an allocation request has been denied
    oom: Cell<bool>,
}

impl StoreLimits {
    /// Creates the default set of limits for all fuzzing stores.
    pub fn new() -> StoreLimits {
        StoreLimits(Rc::new(LimitsState {
            // Limits tables/memories within a store to at most 1gb for now to
            // exercise some larger address but not overflow various limits.
            remaining_memory: Cell::new(1 << 30),
            oom: Cell::new(false),
        }))
    }

    fn alloc(&mut self, amt: usize) -> bool {
        match self.0.remaining_memory.get().checked_sub(amt) {
            Some(mem) => {
                self.0.remaining_memory.set(mem);
                true
            }
            None => {
                self.0.oom.set(true);
                false
            }
        }
    }
}

impl ResourceLimiter for StoreLimits {
    fn memory_growing(&mut self, current: usize, desired: usize, _maximum: Option<usize>) -> bool {
        self.alloc(desired - current)
    }

    fn table_growing(&mut self, current: u32, desired: u32, _maximum: Option<u32>) -> bool {
        let delta = (desired - current) as usize * std::mem::size_of::<usize>();
        self.alloc(delta)
    }
}

/// Methods of timing out execution of a WebAssembly module
#[derive(Clone, Debug)]
pub enum Timeout {
    /// No timeout is used, it should be guaranteed via some other means that
    /// the input does not infinite loop.
    None,
    /// Fuel-based timeouts are used where the specified fuel is all that the
    /// provided wasm module is allowed to consume.
    Fuel(u64),
    /// An epoch-interruption-based timeout is used with a sleeping
    /// thread bumping the epoch counter after the specified duration.
    Epoch(Duration),
}

/// Instantiate the Wasm buffer, and implicitly fail if we have an unexpected
/// panic or segfault or anything else that can be detected "passively".
///
/// The engine will be configured using provided config.
pub fn instantiate(wasm: &[u8], known_valid: bool, config: &generators::Config, timeout: Timeout) {
    let mut store = config.to_store();

    let mut timeout_state = SignalOnDrop::default();
    match timeout {
        Timeout::Fuel(fuel) => set_fuel(&mut store, fuel),

        // If a timeout is requested then we spawn a helper thread to wait for
        // the requested time and then send us a signal to get interrupted. We
        // also arrange for the thread's sleep to get interrupted if we return
        // early (or the wasm returns within the time limit), which allows the
        // thread to get torn down.
        //
        // This prevents us from creating a huge number of sleeping threads if
        // this function is executed in a loop, like it does on nightly fuzzing
        // infrastructure.
        Timeout::Epoch(timeout) => {
            let engine = store.engine().clone();
            timeout_state.spawn_timeout(timeout, move || engine.increment_epoch());
        }
        Timeout::None => {}
    }

    if let Some(module) = compile_module(store.engine(), wasm, known_valid, config) {
        instantiate_with_dummy(&mut store, &module);
    }
}

/// Represents supported commands to the `instantiate_many` function.
#[derive(Arbitrary, Debug)]
pub enum Command {
    /// Instantiates a module.
    ///
    /// The value is the index of the module to instantiate.
    ///
    /// The module instantiated will be this value modulo the number of modules provided to `instantiate_many`.
    Instantiate(usize),
    /// Terminates a "running" instance.
    ///
    /// The value is the index of the instance to terminate.
    ///
    /// The instance terminated will be this value modulo the number of currently running
    /// instances.
    ///
    /// If no instances are running, the command will be ignored.
    Terminate(usize),
}

/// Instantiates many instances from the given modules.
///
/// The engine will be configured using the provided config.
///
/// The modules are expected to *not* have start functions as no timeouts are configured.
pub fn instantiate_many(
    modules: &[Vec<u8>],
    known_valid: bool,
    config: &generators::Config,
    commands: &[Command],
) {
    assert!(!config.module_config.config.allow_start_export);

    let engine = Engine::new(&config.to_wasmtime()).unwrap();

    let modules = modules
        .iter()
        .filter_map(|bytes| compile_module(&engine, bytes, known_valid, config))
        .collect::<Vec<_>>();

    // If no modules were valid, we're done
    if modules.is_empty() {
        return;
    }

    // This stores every `Store` where a successful instantiation takes place
    let mut stores = Vec::new();
    let limits = StoreLimits::new();

    for command in commands {
        match command {
            Command::Instantiate(index) => {
                let index = *index % modules.len();
                log::info!("instantiating {}", index);
                let module = &modules[index];
                let mut store = Store::new(&engine, limits.clone());
                config.configure_store(&mut store);

                if instantiate_with_dummy(&mut store, module).is_some() {
                    stores.push(Some(store));
                } else {
                    log::warn!("instantiation failed");
                }
            }
            Command::Terminate(index) => {
                if stores.is_empty() {
                    continue;
                }
                let index = *index % stores.len();

                log::info!("dropping {}", index);
                stores.swap_remove(index);
            }
        }
    }
}

fn compile_module(
    engine: &Engine,
    bytes: &[u8],
    known_valid: bool,
    config: &generators::Config,
) -> Option<Module> {
    log_wasm(bytes);
    match config.compile(engine, bytes) {
        Ok(module) => Some(module),
        Err(_) if !known_valid => None,
        Err(e) => {
            if let generators::InstanceAllocationStrategy::Pooling { .. } =
                &config.wasmtime.strategy
            {
                // When using the pooling allocator, accept failures to compile
                // when arbitrary table element limits have been exceeded as
                // there is currently no way to constrain the generated module
                // table types.
                let string = e.to_string();
                if string.contains("minimum element size") {
                    return None;
                }

                // Allow modules-failing-to-compile which exceed the requested
                // size for each instance. This is something that is difficult
                // to control and ensure it always succeeds, so we simply have a
                // "random" instance size limit and if a module doesn't fit we
                // move on to the next fuzz input.
                if string.contains("instance allocation for this module requires") {
                    return None;
                }
            }

            panic!("failed to compile module: {:?}", e);
        }
    }
}

/// Create a Wasmtime [`Instance`] from a [`Module`] and fill in all imports
/// with dummy values (e.g., zeroed values, immediately-trapping functions).
/// Also, this function catches certain fuzz-related instantiation failures and
/// returns `None` instead of panicking.
///
/// TODO: we should implement tracing versions of these dummy imports that
/// record a trace of the order that imported functions were called in and with
/// what values. Like the results of exported functions, calls to imports should
/// also yield the same values for each configuration, and we should assert
/// that.
pub fn instantiate_with_dummy(store: &mut Store<StoreLimits>, module: &Module) -> Option<Instance> {
    // Creation of imports can fail due to resource limit constraints, and then
    // instantiation can naturally fail for a number of reasons as well. Bundle
    // the two steps together to match on the error below.
    let instance =
        dummy::dummy_linker(store, module).and_then(|l| l.instantiate(&mut *store, module));

    let e = match instance {
        Ok(i) => return Some(i),
        Err(e) => e,
    };

    // If the instantiation hit OOM for some reason then that's ok, it's
    // expected that fuzz-generated programs try to allocate lots of
    // stuff.
    if store.data().0.oom.get() {
        log::debug!("failed to instantiate: OOM");
        return None;
    }

    // Allow traps which can happen normally with `unreachable` or a
    // timeout or such
    if let Some(trap) = e.downcast_ref::<Trap>() {
        log::debug!("failed to instantiate: {}", trap);
        return None;
    }

    let string = e.to_string();
    // Also allow errors related to fuel consumption
    if string.contains("all fuel consumed")
        // Currently we instantiate with a `Linker` which can't instantiate
        // every single module under the sun due to using name-based resolution
        // rather than positional-based resolution
        || string.contains("incompatible import type")
    {
        log::debug!("failed to instantiate: {}", string);
        return None;
    }

    // Also allow failures to instantiate as a result of hitting instance limits
    if string.contains("concurrent instances has been reached") {
        log::debug!("failed to instantiate: {}", string);
        return None;
    }

    // Everything else should be a bug in the fuzzer or a bug in wasmtime
    panic!("failed to instantiate: {:?}", e);
}

/// Evaluate the function identified by `name` in two different engine
/// instances--`lhs` and `rhs`.
///
/// Returns `Ok(true)` if more evaluations can happen or `Ok(false)` if the
/// instances may have drifted apart and no more evaluations can happen.
///
/// # Panics
///
/// This will panic if the evaluation is different between engines (e.g.,
/// results are different, hashed instance is different, one side traps, etc.).
pub fn differential(
    lhs: &mut dyn DiffInstance,
    lhs_engine: &dyn DiffEngine,
    rhs: &mut WasmtimeInstance,
    name: &str,
    args: &[DiffValue],
    result_tys: &[DiffValueType],
) -> anyhow::Result<bool> {
    log::debug!("Evaluating: `{}` with {:?}", name, args);
    let lhs_results = match lhs.evaluate(name, args, result_tys) {
        Ok(Some(results)) => Ok(results),
        Err(e) => Err(e),
        // this engine couldn't execute this type signature, so discard this
        // execution by returning success.
        Ok(None) => return Ok(true),
    };
    log::debug!(" -> results on {}: {:?}", lhs.name(), &lhs_results);

    let rhs_results = rhs
        .evaluate(name, args, result_tys)
        // wasmtime should be able to invoke any signature, so unwrap this result
        .map(|results| results.unwrap());
    log::debug!(" -> results on {}: {:?}", rhs.name(), &rhs_results);

    match (lhs_results, rhs_results) {
        // If the evaluation succeeds, we compare the results.
        (Ok(lhs_results), Ok(rhs_results)) => assert_eq!(lhs_results, rhs_results),

        // Both sides failed. If either one hits a stack overflow then that's an
        // engine defined limit which means we can no longer compare the state
        // of the two instances, so `false` is returned and nothing else is
        // compared.
        //
        // Otherwise, though, the same error should have popped out and this
        // falls through to checking the intermediate state otherwise.
        (Err(lhs), Err(rhs)) => {
            let err = rhs.downcast::<Trap>().expect("not a trap");
            let poisoned = err.trap_code() == Some(TrapCode::StackOverflow)
                || lhs_engine.is_stack_overflow(&lhs);

            if poisoned {
                return Ok(false);
            }
            lhs_engine.assert_error_match(&err, &lhs);
        }
        // A real bug is found if only one side fails.
        (Ok(_), Err(_)) => panic!("only the `rhs` ({}) failed for this input", rhs.name()),
        (Err(_), Ok(_)) => panic!("only the `lhs` ({}) failed for this input", lhs.name()),
    };

    for (global, ty) in rhs.exported_globals() {
        log::debug!("Comparing global `{global}`");
        let lhs = match lhs.get_global(&global, ty) {
            Some(val) => val,
            None => continue,
        };
        let rhs = rhs.get_global(&global, ty).unwrap();
        assert_eq!(lhs, rhs);
    }
    for (memory, shared) in rhs.exported_memories() {
        log::debug!("Comparing memory `{memory}`");
        let lhs = match lhs.get_memory(&memory, shared) {
            Some(val) => val,
            None => continue,
        };
        let rhs = rhs.get_memory(&memory, shared).unwrap();
        if lhs == rhs {
            continue;
        }
        panic!("memories have differing values");
    }

    Ok(true)
}

/// Invoke the given API calls.
pub fn make_api_calls(api: generators::api::ApiCalls) {
    use crate::generators::api::ApiCall;
    use std::collections::HashMap;

    let mut store: Option<Store<StoreLimits>> = None;
    let mut modules: HashMap<usize, Module> = Default::default();
    let mut instances: HashMap<usize, Instance> = Default::default();

    for call in api.calls {
        match call {
            ApiCall::StoreNew(config) => {
                log::trace!("creating store");
                assert!(store.is_none());
                store = Some(config.to_store());
            }

            ApiCall::ModuleNew { id, wasm } => {
                log::debug!("creating module: {}", id);
                log_wasm(&wasm);
                let module = match Module::new(store.as_ref().unwrap().engine(), &wasm) {
                    Ok(m) => m,
                    Err(_) => continue,
                };
                let old = modules.insert(id, module);
                assert!(old.is_none());
            }

            ApiCall::ModuleDrop { id } => {
                log::trace!("dropping module: {}", id);
                drop(modules.remove(&id));
            }

            ApiCall::InstanceNew { id, module } => {
                log::trace!("instantiating module {} as {}", module, id);
                let module = match modules.get(&module) {
                    Some(m) => m,
                    None => continue,
                };

                let store = store.as_mut().unwrap();
                if let Some(instance) = instantiate_with_dummy(store, module) {
                    instances.insert(id, instance);
                }
            }

            ApiCall::InstanceDrop { id } => {
                log::trace!("dropping instance {}", id);
                drop(instances.remove(&id));
            }

            ApiCall::CallExportedFunc { instance, nth } => {
                log::trace!("calling instance export {} / {}", instance, nth);
                let instance = match instances.get(&instance) {
                    Some(i) => i,
                    None => {
                        // Note that we aren't guaranteed to instantiate valid
                        // modules, see comments in `InstanceNew` for details on
                        // that. But the API call generator can't know if
                        // instantiation failed, so we might not actually have
                        // this instance. When that's the case, just skip the
                        // API call and keep going.
                        continue;
                    }
                };
                let store = store.as_mut().unwrap();

                let funcs = instance
                    .exports(&mut *store)
                    .filter_map(|e| match e.into_extern() {
                        Extern::Func(f) => Some(f.clone()),
                        _ => None,
                    })
                    .collect::<Vec<_>>();

                if funcs.is_empty() {
                    continue;
                }

                let nth = nth % funcs.len();
                let f = &funcs[nth];
                let ty = f.ty(&store);
                let params = dummy::dummy_values(ty.params());
                let mut results = vec![Val::I32(0); ty.results().len()];
                let _ = f.call(store, &params, &mut results);
            }
        }
    }
}

/// Executes the wast `test` spectest with the `config` specified.
///
/// Ensures that spec tests pass regardless of the `Config`.
pub fn spectest(mut fuzz_config: generators::Config, test: generators::SpecTest) {
    crate::init_fuzzing();
    fuzz_config.set_spectest_compliant();
    log::debug!("running {:?}", test.file);
    let mut wast_context = WastContext::new(fuzz_config.to_store());
    wast_context.register_spectest().unwrap();
    wast_context
        .run_buffer(test.file, test.contents.as_bytes())
        .unwrap();
}

/// Execute a series of `table.get` and `table.set` operations.
///
/// Returns the number of `gc` operations which occurred throughout the test
/// case -- used to test below that gc happens reasonably soon and eventually.
pub fn table_ops(
    mut fuzz_config: generators::Config,
    ops: generators::table_ops::TableOps,
) -> usize {
    let expected_drops = Arc::new(AtomicUsize::new(ops.num_params as usize));
    let num_dropped = Arc::new(AtomicUsize::new(0));

    let num_gcs = Arc::new(AtomicUsize::new(0));
    {
        fuzz_config.wasmtime.consume_fuel = true;
        let mut store = fuzz_config.to_store();
        set_fuel(&mut store, 1_000);

        let wasm = ops.to_wasm_binary();
        log_wasm(&wasm);
        let module = match compile_module(store.engine(), &wasm, false, &fuzz_config) {
            Some(m) => m,
            None => return 0,
        };

        let mut linker = Linker::new(store.engine());

        // To avoid timeouts, limit the number of explicit GCs we perform per
        // test case.
        const MAX_GCS: usize = 5;

        linker
            .define(
                "",
                "gc",
                // NB: use `Func::new` so that this can still compile on the old x86
                // backend, where `IntoFunc` isn't implemented for multi-value
                // returns.
                Func::new(
                    &mut store,
                    FuncType::new(
                        vec![],
                        vec![ValType::ExternRef, ValType::ExternRef, ValType::ExternRef],
                    ),
                    {
                        let num_dropped = num_dropped.clone();
                        let expected_drops = expected_drops.clone();
                        let num_gcs = num_gcs.clone();
                        move |mut caller: Caller<'_, StoreLimits>, _params, results| {
                            log::info!("table_ops: GC");
                            if num_gcs.fetch_add(1, SeqCst) < MAX_GCS {
                                caller.gc();
                            }

                            let a = ExternRef::new(CountDrops(num_dropped.clone()));
                            let b = ExternRef::new(CountDrops(num_dropped.clone()));
                            let c = ExternRef::new(CountDrops(num_dropped.clone()));

                            log::info!("table_ops: make_refs() -> ({:p}, {:p}, {:p})", a, b, c);

                            expected_drops.fetch_add(3, SeqCst);
                            results[0] = Some(a).into();
                            results[1] = Some(b).into();
                            results[2] = Some(c).into();
                            Ok(())
                        }
                    },
                ),
            )
            .unwrap();

        linker
            .func_wrap("", "take_refs", {
                let expected_drops = expected_drops.clone();
                move |a: Option<ExternRef>, b: Option<ExternRef>, c: Option<ExternRef>| {
                    log::info!(
                        "table_ops: take_refs({}, {}, {})",
                        a.as_ref().map_or_else(
                            || format!("{:p}", std::ptr::null::<()>()),
                            |r| format!("{:p}", *r)
                        ),
                        b.as_ref().map_or_else(
                            || format!("{:p}", std::ptr::null::<()>()),
                            |r| format!("{:p}", *r)
                        ),
                        c.as_ref().map_or_else(
                            || format!("{:p}", std::ptr::null::<()>()),
                            |r| format!("{:p}", *r)
                        ),
                    );

                    // Do the assertion on each ref's inner data, even though it
                    // all points to the same atomic, so that if we happen to
                    // run into a use-after-free bug with one of these refs we
                    // are more likely to trigger a segfault.
                    if let Some(a) = a {
                        let a = a.data().downcast_ref::<CountDrops>().unwrap();
                        assert!(a.0.load(SeqCst) <= expected_drops.load(SeqCst));
                    }
                    if let Some(b) = b {
                        let b = b.data().downcast_ref::<CountDrops>().unwrap();
                        assert!(b.0.load(SeqCst) <= expected_drops.load(SeqCst));
                    }
                    if let Some(c) = c {
                        let c = c.data().downcast_ref::<CountDrops>().unwrap();
                        assert!(c.0.load(SeqCst) <= expected_drops.load(SeqCst));
                    }
                }
            })
            .unwrap();

        linker
            .define(
                "",
                "make_refs",
                // NB: use `Func::new` so that this can still compile on the old
                // x86 backend, where `IntoFunc` isn't implemented for
                // multi-value returns.
                Func::new(
                    &mut store,
                    FuncType::new(
                        vec![],
                        vec![ValType::ExternRef, ValType::ExternRef, ValType::ExternRef],
                    ),
                    {
                        let num_dropped = num_dropped.clone();
                        let expected_drops = expected_drops.clone();
                        move |_caller, _params, results| {
                            log::info!("table_ops: make_refs");
                            expected_drops.fetch_add(3, SeqCst);
                            results[0] =
                                Some(ExternRef::new(CountDrops(num_dropped.clone()))).into();
                            results[1] =
                                Some(ExternRef::new(CountDrops(num_dropped.clone()))).into();
                            results[2] =
                                Some(ExternRef::new(CountDrops(num_dropped.clone()))).into();
                            Ok(())
                        }
                    },
                ),
            )
            .unwrap();

        let instance = linker.instantiate(&mut store, &module).unwrap();
        let run = instance.get_func(&mut store, "run").unwrap();

        let args: Vec<_> = (0..ops.num_params)
            .map(|_| Val::ExternRef(Some(ExternRef::new(CountDrops(num_dropped.clone())))))
            .collect();

        // The generated function should always return a trap. The only two
        // valid traps are table-out-of-bounds which happens through `table.get`
        // and `table.set` generated or an out-of-fuel trap. Otherwise any other
        // error is unexpected and should fail fuzzing.
        let trap = run
            .call(&mut store, &args, &mut [])
            .unwrap_err()
            .downcast::<Trap>()
            .unwrap();

        match trap.trap_code() {
            Some(TrapCode::TableOutOfBounds) => {}
            None if trap
                .to_string()
                .contains("all fuel consumed by WebAssembly") => {}
            _ => {
                panic!("unexpected trap: {}", trap);
            }
        }

        // Do a final GC after running the Wasm.
        store.gc();
    }

    assert_eq!(num_dropped.load(SeqCst), expected_drops.load(SeqCst));
    return num_gcs.load(SeqCst);

    struct CountDrops(Arc<AtomicUsize>);

    impl Drop for CountDrops {
        fn drop(&mut self) {
            self.0.fetch_add(1, SeqCst);
        }
    }
}

// Test that the `table_ops` fuzzer eventually runs the gc function in the host.
// We've historically had issues where this fuzzer accidentally wasn't fuzzing
// anything for a long time so this is an attempt to prevent that from happening
// again.
#[test]
fn table_ops_eventually_gcs() {
    use arbitrary::Unstructured;
    use rand::prelude::*;

    // Skip if we're under emulation because some fuzz configurations will do
    // large address space reservations that QEMU doesn't handle well.
    if std::env::var("WASMTIME_TEST_NO_HOG_MEMORY").is_ok() {
        return;
    }

    let mut rng = SmallRng::seed_from_u64(0);
    let mut buf = vec![0; 2048];
    let n = 100;
    for _ in 0..n {
        rng.fill_bytes(&mut buf);
        let u = Unstructured::new(&buf);

        if let Ok((config, test)) = Arbitrary::arbitrary_take_rest(u) {
            if table_ops(config, test) > 0 {
                return;
            }
        }
    }

    panic!("after {n} runs nothing ever gc'd, something is probably wrong");
}

#[derive(Default)]
struct SignalOnDrop {
    state: Arc<(Mutex<bool>, Condvar)>,
    thread: Option<std::thread::JoinHandle<()>>,
}

impl SignalOnDrop {
    fn spawn_timeout(&mut self, dur: Duration, closure: impl FnOnce() + Send + 'static) {
        let state = self.state.clone();
        let start = Instant::now();
        self.thread = Some(std::thread::spawn(move || {
            // Using our mutex/condvar we wait here for the first of `dur` to
            // pass or the `SignalOnDrop` instance to get dropped.
            let (lock, cvar) = &*state;
            let mut signaled = lock.lock().unwrap();
            while !*signaled {
                // Adjust our requested `dur` based on how much time has passed.
                let dur = match dur.checked_sub(start.elapsed()) {
                    Some(dur) => dur,
                    None => break,
                };
                let (lock, result) = cvar.wait_timeout(signaled, dur).unwrap();
                signaled = lock;
                // If we timed out for sure then there's no need to continue
                // since we'll just abort on the next `checked_sub` anyway.
                if result.timed_out() {
                    break;
                }
            }
            drop(signaled);

            closure();
        }));
    }
}

impl Drop for SignalOnDrop {
    fn drop(&mut self) {
        if let Some(thread) = self.thread.take() {
            let (lock, cvar) = &*self.state;
            // Signal our thread that we've been dropped and wake it up if it's
            // blocked.
            let mut g = lock.lock().unwrap();
            *g = true;
            cvar.notify_one();
            drop(g);

            // ... and then wait for the thread to exit to ensure we clean up
            // after ourselves.
            thread.join().unwrap();
        }
    }
}

fn set_fuel<T>(store: &mut Store<T>, fuel: u64) {
    // Determine the amount of fuel already within the store, if any, and
    // add/consume as appropriate to set the remaining amount to` fuel`.
    let remaining = store.consume_fuel(0).unwrap();
    if fuel > remaining {
        store.add_fuel(fuel - remaining).unwrap();
    } else {
        store.consume_fuel(remaining - fuel).unwrap();
    }
    // double-check that the store has the expected amount of fuel remaining
    assert_eq!(store.consume_fuel(0).unwrap(), fuel);
}

/// Generate and execute a `crate::generators::component_types::TestCase` using the specified `input` to create
/// arbitrary types and values.
pub fn dynamic_component_api_target(input: &mut arbitrary::Unstructured) -> arbitrary::Result<()> {
    use crate::generators::component_types;
    use anyhow::Result;
    use component_fuzz_util::{TestCase, EXPORT_FUNCTION, IMPORT_FUNCTION};
    use component_test_util::FuncExt;
    use wasmtime::component::{Component, Linker, Val};

    crate::init_fuzzing();

    let case = input.arbitrary::<TestCase>()?;

    let mut config = component_test_util::config();
    config.debug_adapter_modules(input.arbitrary()?);
    let engine = Engine::new(&config).unwrap();
    let mut store = Store::new(&engine, (Vec::new(), None));
    let wat = case.declarations().make_component();
    let wat = wat.as_bytes();
    log_wasm(wat);
    let component = Component::new(&engine, wat).unwrap();
    let mut linker = Linker::new(&engine);

    linker
        .root()
        .func_new(&component, IMPORT_FUNCTION, {
            move |mut cx: StoreContextMut<'_, (Vec<Val>, Option<Vec<Val>>)>,
                  params: &[Val],
                  results: &mut [Val]|
                  -> Result<()> {
                log::trace!("received params {params:?}");
                let (expected_args, expected_results) = cx.data_mut();
                assert_eq!(params.len(), expected_args.len());
                for (expected, actual) in expected_args.iter().zip(params) {
                    assert_eq!(expected, actual);
                }
                results.clone_from_slice(&expected_results.take().unwrap());
                log::trace!("returning results {results:?}");
                Ok(())
            }
        })
        .unwrap();

    let instance = linker.instantiate(&mut store, &component).unwrap();
    let func = instance.get_func(&mut store, EXPORT_FUNCTION).unwrap();
    let param_tys = func.params(&store);
    let result_tys = func.results(&store);

    while input.arbitrary()? {
        let params = param_tys
            .iter()
            .map(|ty| component_types::arbitrary_val(ty, input))
            .collect::<arbitrary::Result<Vec<_>>>()?;
        let results = result_tys
            .iter()
            .map(|ty| component_types::arbitrary_val(ty, input))
            .collect::<arbitrary::Result<Vec<_>>>()?;

        *store.data_mut() = (params.clone(), Some(results.clone()));

        log::trace!("passing params {params:?}");
        let mut actual = vec![Val::Bool(false); results.len()];
        func.call_and_post_return(&mut store, &params, &mut actual)
            .unwrap();
        log::trace!("received results {actual:?}");
        assert_eq!(actual, results);
    }

    Ok(())
}
