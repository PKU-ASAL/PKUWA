//! Implementation of "inlining" a component into a flat list of initializers.
//!
//! After the first phase of compiling a component we're left with a single
//! root `Translation` for the original component along with a "static" list of
//! child components. Each `Translation` has a list of `LocalInitializer` items
//! inside of it which is a primitive representation of how the component
//! should be constructed with effectively one initializer per item in the
//! index space of a component. This "local initializer" list would be
//! relatively inefficient to process at runtime and more importantly doesn't
//! convey enough information to understand what trampolines need to be
//! compiled or what fused adapters need to be generated. This consequently is
//! the motivation for this file.
//!
//! The second phase of compilation, inlining here, will in a sense interpret
//! the initializers, at compile time, into a new list of `GlobalInitializer` entries
//! which are a sort of "global initializer". The generated `GlobalInitializer` is
//! much more specific than the `LocalInitializer` and additionally far fewer
//! `GlobalInitializer` structures are generated (in theory) than there are local
//! initializers.
//!
//! The "inlining" portion of the name of this module indicates how the
//! instantiation of a component is interpreted as calling a function. The
//! function's arguments are the imports provided to the instantiation of a
//! component, and further nested function calls happen on a stack when a
//! nested component is instantiated. The inlining then refers to how this
//! stack of instantiations is flattened to one list of `GlobalInitializer`
//! entries to represent the process of instantiating a component graph,
//! similar to how function inlining removes call instructions and creates one
//! giant function for a call graph. Here there are no inlining heuristics or
//! anything like that, we simply inline everything into the root component's
//! list of initializers.
//!
//! Another primary task this module performs is a form of dataflow analysis
//! to represent items in each index space with their definition rather than
//! references of relative indices. These definitions (all the `*Def` types in
//! this module) are not local to any one nested component and instead
//! represent state available at runtime tracked in the final `Component`
//! produced.
//!
//! With all this pieced together the general idea is relatively
//! straightforward. All of a component's initializers are processed in sequence
//! where instantiating a nested component pushes a "frame" onto a stack to
//! start executing and we resume at the old one when we're done. Items are
//! tracked where they come from and at the end after processing only the
//! side-effectful initializers are emitted to the `GlobalInitializer` list in the
//! final `Component`.

use crate::component::translate::adapt::{Adapter, AdapterOptions};
use crate::component::translate::*;
use crate::{EntityType, PrimaryMap};
use indexmap::IndexMap;

pub(super) fn run(
    types: &ComponentTypesBuilder,
    result: &Translation<'_>,
    nested_modules: &PrimaryMap<StaticModuleIndex, ModuleTranslation<'_>>,
    nested_components: &PrimaryMap<StaticComponentIndex, Translation<'_>>,
) -> Result<dfg::ComponentDfg> {
    let mut inliner = Inliner {
        types,
        nested_modules,
        nested_components,
        result: Default::default(),
        import_path_interner: Default::default(),
        runtime_instances: PrimaryMap::default(),
    };

    // The initial arguments to the root component are all host imports. This
    // means that they're all using the `ComponentItemDef::Host` variant. Here
    // an `ImportIndex` is allocated for each item and then the argument is
    // recorded.
    //
    // Note that this is represents the abstract state of a host import of an
    // item since we don't know the precise structure of the host import.
    let mut args = HashMap::with_capacity(result.exports.len());
    for init in result.initializers.iter() {
        let (name, ty) = match *init {
            LocalInitializer::Import(name, ty) => (name, ty),
            _ => continue,
        };
        let index = inliner.result.import_types.push((name.to_string(), ty));
        let path = ImportPath::root(index);
        args.insert(name, ComponentItemDef::from_import(path, ty)?);
    }

    // This will run the inliner to completion after being seeded with the
    // initial frame. When the inliner finishes it will return the exports of
    // the root frame which are then used for recording the exports of the
    // component.
    let index = RuntimeComponentInstanceIndex::from_u32(0);
    inliner.result.num_runtime_component_instances += 1;
    let mut frames = vec![InlinerFrame::new(
        index,
        result,
        ComponentClosure::default(),
        args,
    )];
    let exports = inliner.run(&mut frames)?;
    assert!(frames.is_empty());

    let mut export_map = Default::default();
    for (name, def) in exports {
        inliner.record_export(name, def, &mut export_map)?;
    }
    inliner.result.exports = export_map;

    Ok(inliner.result)
}

struct Inliner<'a> {
    /// Global type information for the entire component.
    types: &'a ComponentTypesBuilder,

    /// The list of static modules that were found during initial translation of
    /// the component.
    ///
    /// This is used during the instantiation of these modules to ahead-of-time
    /// order the arguments precisely according to what the module is defined as
    /// needing which avoids the need to do string lookups or permute arguments
    /// at runtime.
    nested_modules: &'a PrimaryMap<StaticModuleIndex, ModuleTranslation<'a>>,

    /// The list of static components that were found during initial translation of
    /// the component.
    ///
    /// This is used when instantiating nested components to push a new
    /// `InlinerFrame` with the `Translation`s here.
    nested_components: &'a PrimaryMap<StaticComponentIndex, Translation<'a>>,

    /// The final `Component` that is being constructed and returned from this
    /// inliner.
    result: dfg::ComponentDfg,

    // Maps used to "intern" various runtime items to only save them once at
    // runtime instead of multiple times.
    import_path_interner: HashMap<ImportPath<'a>, RuntimeImportIndex>,

    /// Origin information about where each runtime instance came from
    runtime_instances: PrimaryMap<dfg::InstanceId, InstanceModule>,
}

/// A "stack frame" as part of the inlining process, or the progress through
/// instantiating a component.
///
/// All instantiations of a component will create an `InlinerFrame` and are
/// incrementally processed via the `initializers` list here. Note that the
/// inliner frames are stored on the heap to avoid recursion based on user
/// input.
struct InlinerFrame<'a> {
    instance: RuntimeComponentInstanceIndex,

    /// The remaining initializers to process when instantiating this component.
    initializers: std::slice::Iter<'a, LocalInitializer<'a>>,

    /// The component being instantiated.
    translation: &'a Translation<'a>,

    /// The "closure arguments" to this component, or otherwise the maps indexed
    /// by `ModuleUpvarIndex` and `ComponentUpvarIndex`. This is created when
    /// a component is created and stored as part of a component's state during
    /// inlining.
    closure: ComponentClosure<'a>,

    /// The arguments to the creation of this component.
    ///
    /// At the root level these are all imports from the host and between
    /// components this otherwise tracks how all the arguments are defined.
    args: HashMap<&'a str, ComponentItemDef<'a>>,

    // core wasm index spaces
    funcs: PrimaryMap<FuncIndex, dfg::CoreDef>,
    memories: PrimaryMap<MemoryIndex, dfg::CoreExport<EntityIndex>>,
    tables: PrimaryMap<TableIndex, dfg::CoreExport<EntityIndex>>,
    globals: PrimaryMap<GlobalIndex, dfg::CoreExport<EntityIndex>>,
    modules: PrimaryMap<ModuleIndex, ModuleDef<'a>>,

    // component model index spaces
    component_funcs: PrimaryMap<ComponentFuncIndex, ComponentFuncDef<'a>>,
    module_instances: PrimaryMap<ModuleInstanceIndex, ModuleInstanceDef<'a>>,
    component_instances: PrimaryMap<ComponentInstanceIndex, ComponentInstanceDef<'a>>,
    components: PrimaryMap<ComponentIndex, ComponentDef<'a>>,
}

/// "Closure state" for a component which is resolved from the `ClosedOverVars`
/// state that was calculated during translation.
//
// FIXME: this is cloned quite a lot and given the internal maps if this is a
// perf issue we may want to `Rc` these fields. Note that this is only a perf
// hit at compile-time though which we in general don't pay too too much
// attention to.
#[derive(Default, Clone)]
struct ComponentClosure<'a> {
    modules: PrimaryMap<ModuleUpvarIndex, ModuleDef<'a>>,
    components: PrimaryMap<ComponentUpvarIndex, ComponentDef<'a>>,
}

/// Representation of a "path" into an import.
///
/// Imports from the host at this time are one of three things:
///
/// * Functions
/// * Core wasm modules
/// * "Instances" of these three items
///
/// The "base" values are functions and core wasm modules, but the abstraction
/// of an instance allows embedding functions/modules deeply within other
/// instances. This "path" represents optionally walking through a host instance
/// to get to the final desired item. At runtime instances are just maps of
/// values and so this is used to ensure that we primarily only deal with
/// individual functions and modules instead of synthetic instances.
#[derive(Clone, PartialEq, Hash, Eq)]
struct ImportPath<'a> {
    index: ImportIndex,
    path: Vec<&'a str>,
}

/// Representation of all items which can be defined within a component.
///
/// This is the "value" of an item defined within a component and is used to
/// represent both imports and exports.
#[derive(Clone)]
enum ComponentItemDef<'a> {
    Component(ComponentDef<'a>),
    Instance(ComponentInstanceDef<'a>),
    Func(ComponentFuncDef<'a>),
    Module(ModuleDef<'a>),
    // TODO: https://github.com/bytecodealliance/wasmtime/issues/4494
    // The entity is a type; currently unsupported but represented here
    // so that type exports can be ignored for now.
    Type,
}

#[derive(Clone)]
enum ModuleDef<'a> {
    /// A core wasm module statically defined within the original component.
    ///
    /// The `StaticModuleIndex` indexes into the `static_modules` map in the
    /// `Inliner`.
    Static(StaticModuleIndex),

    /// A core wasm module that was imported from the host.
    Import(ImportPath<'a>, TypeModuleIndex),
}

// Note that unlike all other `*Def` types which are not allowed to have local
// indices this type does indeed have local indices. That is represented with
// the lack of a `Clone` here where once this is created it's never moved across
// components because module instances always stick within one component.
enum ModuleInstanceDef<'a> {
    /// A core wasm module instance was created through the instantiation of a
    /// module.
    ///
    /// The `RuntimeInstanceIndex` was the index allocated as this was the
    /// `n`th instantiation and the `ModuleIndex` points into an
    /// `InlinerFrame`'s local index space.
    Instantiated(dfg::InstanceId, ModuleIndex),

    /// A "synthetic" core wasm module which is just a bag of named indices.
    ///
    /// Note that this can really only be used for passing as an argument to
    /// another module's instantiation and is used to rename arguments locally.
    Synthetic(&'a HashMap<&'a str, EntityIndex>),
}

#[derive(Clone)]
enum ComponentFuncDef<'a> {
    /// A host-imported component function.
    Import(ImportPath<'a>),

    /// A core wasm function was lifted into a component function.
    Lifted {
        ty: TypeFuncIndex,
        func: dfg::CoreDef,
        options: AdapterOptions,
    },
}

#[derive(Clone)]
enum ComponentInstanceDef<'a> {
    /// A host-imported instance.
    ///
    /// This typically means that it's "just" a map of named values. It's not
    /// actually supported to take a `wasmtime::component::Instance` and pass it
    /// to another instance at this time.
    Import(ImportPath<'a>, TypeComponentInstanceIndex),

    /// A concrete map of values.
    ///
    /// This is used for both instantiated components as well as "synthetic"
    /// components. This variant can be used for both because both are
    /// represented by simply a bag of items within the entire component
    /// instantiation process.
    //
    // FIXME: same as the issue on `ComponentClosure` where this is cloned a lot
    // and may need `Rc`.
    Items(IndexMap<&'a str, ComponentItemDef<'a>>),
}

#[derive(Clone)]
struct ComponentDef<'a> {
    index: StaticComponentIndex,
    closure: ComponentClosure<'a>,
}

impl<'a> Inliner<'a> {
    fn run(
        &mut self,
        frames: &mut Vec<InlinerFrame<'a>>,
    ) -> Result<IndexMap<&'a str, ComponentItemDef<'a>>> {
        // This loop represents the execution of the instantiation of a
        // component. This is an iterative process which is finished once all
        // initializers are processed. Currently this is modeled as an infinite
        // loop which drives the top-most iterator of the `frames` stack
        // provided as an argument to this function.
        loop {
            let frame = frames.last_mut().unwrap();
            match frame.initializers.next() {
                // Process the initializer and if it started the instantiation
                // of another component then we push that frame on the stack to
                // continue onwards.
                Some(init) => match self.initializer(frame, init)? {
                    Some(new_frame) => frames.push(new_frame),
                    None => {}
                },

                // If there are no more initializers for this frame then the
                // component it represents has finished instantiation. The
                // exports of the component are collected and then the entire
                // frame is discarded. The exports are then either pushed in the
                // parent frame, if any, as a new component instance or they're
                // returned from this function for the root set of exports.
                None => {
                    let exports = frame
                        .translation
                        .exports
                        .iter()
                        .map(|(name, item)| (*name, frame.item(*item)))
                        .collect();
                    frames.pop();
                    match frames.last_mut() {
                        Some(parent) => {
                            parent
                                .component_instances
                                .push(ComponentInstanceDef::Items(exports));
                        }
                        None => break Ok(exports),
                    }
                }
            }
        }
    }

    fn initializer(
        &mut self,
        frame: &mut InlinerFrame<'a>,
        initializer: &'a LocalInitializer,
    ) -> Result<Option<InlinerFrame<'a>>> {
        use LocalInitializer::*;

        match initializer {
            // When a component imports an item the actual definition of the
            // item is looked up here (not at runtime) via its name. The
            // arguments provided in our `InlinerFrame` describe how each
            // argument was defined, so we simply move it from there into the
            // correct index space.
            //
            // Note that for the root component this will add `*::Import` items
            // but for sub-components this will do resolution to connect what
            // was provided as an import at the instantiation-site to what was
            // needed during the component's instantiation.
            Import(name, _ty) => match &frame.args[name] {
                ComponentItemDef::Module(i) => {
                    frame.modules.push(i.clone());
                }
                ComponentItemDef::Component(i) => {
                    frame.components.push(i.clone());
                }
                ComponentItemDef::Instance(i) => {
                    frame.component_instances.push(i.clone());
                }
                ComponentItemDef::Func(i) => {
                    frame.component_funcs.push(i.clone());
                }
                ComponentItemDef::Type => {}
            },

            // Lowering a component function to a core wasm function is
            // generally what "triggers compilation". Here various metadata is
            // recorded and then the final component gets an initializer
            // recording the lowering.
            //
            // NB: at this time only lowered imported functions are supported.
            Lower(func, options) => {
                let canonical_abi = frame.translation.funcs[frame.funcs.next_key()];
                let lower_ty = frame.translation.component_funcs[*func];

                let options_lower = self.adapter_options(frame, options);
                let func = match &frame.component_funcs[*func] {
                    // If this component function was originally a host import
                    // then this is a lowered host function which needs a
                    // trampoline to enter WebAssembly. That's recorded here
                    // with all relevant information.
                    ComponentFuncDef::Import(path) => {
                        let import = self.runtime_import(path);
                        let options = self.canonical_options(options_lower);
                        let index = self.result.lowerings.push_uniq(dfg::LowerImport {
                            canonical_abi,
                            import,
                            options,
                        });
                        dfg::CoreDef::Lowered(index)
                    }

                    // This case handles when a lifted function is later
                    // lowered, and both the lowering and the lifting are
                    // happening within the same component instance.
                    //
                    // In this situation if the `canon.lower`'d function is
                    // called then it immediately sets `may_enter` to `false`.
                    // When calling the callee, however, that's `canon.lift`
                    // which immediately traps if `may_enter` is `false`. That
                    // means that this pairing of functions creates a function
                    // that always traps.
                    //
                    // When closely reading the spec though the precise trap
                    // that comes out can be somewhat variable. Technically the
                    // function yielded here is one that should validate the
                    // arguments by lifting them, and then trap. This means that
                    // the trap could be different depending on whether all
                    // arguments are valid for now. This was discussed in
                    // WebAssembly/component-model#51 somewhat and the
                    // conclusion was that we can probably get away with "always
                    // trap" here.
                    //
                    // The `CoreDef::AlwaysTrap` variant here is used to
                    // indicate that this function is valid but if something
                    // actually calls it then it just generates a trap
                    // immediately.
                    ComponentFuncDef::Lifted {
                        options: options_lift,
                        ..
                    } if options_lift.instance == options_lower.instance => {
                        let index = self.result.always_trap.push_uniq(canonical_abi);
                        dfg::CoreDef::AlwaysTrap(index)
                    }

                    // Lowering a lifted function where the destination
                    // component is different than the source component means
                    // that a "fused adapter" was just identified.
                    //
                    // Metadata about this fused adapter is recorded in the
                    // `Adapters` output of this compilation pass. Currently the
                    // implementation of fused adapters is to generate a core
                    // wasm module which is instantiated with relevant imports
                    // and the exports are used as the fused adapters. At this
                    // time we don't know when precisely the instance will be
                    // created but we do know that the result of this will be an
                    // export from a previously-created instance.
                    //
                    // To model this the result of this arm is a
                    // `CoreDef::Export`. The actual indices listed within the
                    // export are "fake indices" in the sense of they're not
                    // resolved yet. This resolution will happen at a later
                    // compilation phase. Any usages of the `CoreDef::Export`
                    // here will be detected and rewritten to an actual runtime
                    // instance created.
                    //
                    // The `instance` field of the `CoreExport` has a marker
                    // which indicates that it's a fused adapter. The `item` is
                    // a function where the function index corresponds to the
                    // `adapter_idx` which contains the metadata about this
                    // adapter being created. The metadata is used to learn
                    // about the dependencies and when the adapter module can
                    // be instantiated.
                    ComponentFuncDef::Lifted {
                        ty: lift_ty,
                        func,
                        options: options_lift,
                    } => {
                        let adapter_idx = self.result.adapters.push_uniq(Adapter {
                            lift_ty: *lift_ty,
                            lift_options: options_lift.clone(),
                            lower_ty,
                            lower_options: options_lower,
                            func: func.clone(),
                        });
                        dfg::CoreDef::Adapter(adapter_idx)
                    }
                };
                frame.funcs.push(func);
            }

            // Lifting a core wasm function is relatively easy for now in that
            // some metadata about the lifting is simply recorded. This'll get
            // plumbed through to exports or a fused adapter later on.
            Lift(ty, func, options) => {
                let options = self.adapter_options(frame, options);
                frame.component_funcs.push(ComponentFuncDef::Lifted {
                    ty: *ty,
                    func: frame.funcs[*func].clone(),
                    options,
                });
            }

            ModuleStatic(idx) => {
                frame.modules.push(ModuleDef::Static(*idx));
            }

            // Instantiation of a module is one of the meatier initializers that
            // we'll generate. The main magic here is that for a statically
            // known module we can order the imports as a list to exactly what
            // the static module needs to be instantiated. For imported modules,
            // however, the runtime string resolution must happen at runtime so
            // that is deferred here by organizing the arguments as a two-layer
            // `IndexMap` of what we're providing.
            //
            // In both cases though a new `RuntimeInstanceIndex` is allocated
            // and an initializer is recorded to indicate that it's being
            // instantiated.
            ModuleInstantiate(module, args) => {
                let instance_module;
                let init = match &frame.modules[*module] {
                    ModuleDef::Static(idx) => {
                        let mut defs = Vec::new();
                        for (module, name, _ty) in self.nested_modules[*idx].module.imports() {
                            let instance = args[module];
                            defs.push(
                                self.core_def_of_module_instance_export(frame, instance, name),
                            );
                        }
                        instance_module = InstanceModule::Static(*idx);
                        dfg::Instance::Static(*idx, defs.into())
                    }
                    ModuleDef::Import(path, ty) => {
                        let mut defs = IndexMap::new();
                        for ((module, name), _) in self.types[*ty].imports.iter() {
                            let instance = args[module.as_str()];
                            let def =
                                self.core_def_of_module_instance_export(frame, instance, name);
                            defs.entry(module.to_string())
                                .or_insert(IndexMap::new())
                                .insert(name.to_string(), def);
                        }
                        let index = self.runtime_import(path);
                        instance_module = InstanceModule::Import(*ty);
                        dfg::Instance::Import(index, defs)
                    }
                };

                let idx = self.result.instances.push(init);
                let idx2 = self.runtime_instances.push(instance_module);
                assert_eq!(idx, idx2);
                frame
                    .module_instances
                    .push(ModuleInstanceDef::Instantiated(idx, *module));
            }

            ModuleSynthetic(map) => {
                frame
                    .module_instances
                    .push(ModuleInstanceDef::Synthetic(map));
            }

            // This is one of the stages of the "magic" of implementing outer
            // aliases to components and modules. For more information on this
            // see the documentation on `LexicalScope`. This stage of the
            // implementation of outer aliases is where the `ClosedOverVars` is
            // transformed into a `ComponentClosure` state using the current
            // `InlinerFrame`'s state. This will capture the "runtime" state of
            // outer components and upvars and such naturally as part of the
            // inlining process.
            ComponentStatic(index, vars) => {
                frame.components.push(ComponentDef {
                    index: *index,
                    closure: ComponentClosure {
                        modules: vars
                            .modules
                            .iter()
                            .map(|(_, m)| frame.closed_over_module(m))
                            .collect(),
                        components: vars
                            .components
                            .iter()
                            .map(|(_, m)| frame.closed_over_component(m))
                            .collect(),
                    },
                });
            }

            // Like module instantiation is this is a "meaty" part, and don't be
            // fooled by the relative simplicity of this case. This is
            // implemented primarily by the `Inliner` structure and the design
            // of this entire module, so the "easy" step here is to simply
            // create a new inliner frame and return it to get pushed onto the
            // stack.
            ComponentInstantiate(component, args) => {
                let component: &ComponentDef<'a> = &frame.components[*component];
                let index = RuntimeComponentInstanceIndex::from_u32(
                    self.result.num_runtime_component_instances,
                );
                self.result.num_runtime_component_instances += 1;
                let frame = InlinerFrame::new(
                    index,
                    &self.nested_components[component.index],
                    component.closure.clone(),
                    args.iter()
                        .map(|(name, item)| (*name, frame.item(*item)))
                        .collect(),
                );
                return Ok(Some(frame));
            }

            ComponentSynthetic(map) => {
                let items = map
                    .iter()
                    .map(|(name, index)| (*name, frame.item(*index)))
                    .collect();
                frame
                    .component_instances
                    .push(ComponentInstanceDef::Items(items));
            }

            // Core wasm aliases, this and the cases below, are creating
            // `CoreExport` items primarily to insert into the index space so we
            // can create a unique identifier pointing to each core wasm export
            // with the instance and relevant index/name as necessary.
            AliasExportFunc(instance, name) => {
                frame
                    .funcs
                    .push(self.core_def_of_module_instance_export(frame, *instance, *name));
            }

            AliasExportTable(instance, name) => {
                frame.tables.push(
                    match self.core_def_of_module_instance_export(frame, *instance, *name) {
                        dfg::CoreDef::Export(e) => e,
                        _ => unreachable!(),
                    },
                );
            }

            AliasExportGlobal(instance, name) => {
                frame.globals.push(
                    match self.core_def_of_module_instance_export(frame, *instance, *name) {
                        dfg::CoreDef::Export(e) => e,
                        _ => unreachable!(),
                    },
                );
            }

            AliasExportMemory(instance, name) => {
                frame.memories.push(
                    match self.core_def_of_module_instance_export(frame, *instance, *name) {
                        dfg::CoreDef::Export(e) => e,
                        _ => unreachable!(),
                    },
                );
            }

            AliasComponentExport(instance, name) => {
                match &frame.component_instances[*instance] {
                    // Aliasing an export from an imported instance means that
                    // we're extending the `ImportPath` by one name, represented
                    // with the clone + push here. Afterwards an appropriate
                    // item is then pushed in the relevant index space.
                    ComponentInstanceDef::Import(path, ty) => {
                        let mut path = path.clone();
                        path.path.push(name);
                        match self.types[*ty].exports[*name] {
                            TypeDef::ComponentFunc(_) => {
                                frame.component_funcs.push(ComponentFuncDef::Import(path));
                            }
                            TypeDef::ComponentInstance(ty) => {
                                frame
                                    .component_instances
                                    .push(ComponentInstanceDef::Import(path, ty));
                            }
                            TypeDef::Module(ty) => {
                                frame.modules.push(ModuleDef::Import(path, ty));
                            }
                            TypeDef::Component(_) => {
                                unimplemented!("aliasing component export of component import")
                            }
                            TypeDef::Interface(_) => {
                                unimplemented!("aliasing type export of component import")
                            }

                            // not possible with valid components
                            TypeDef::CoreFunc(_) => unreachable!(),
                        }
                    }

                    // Given a component instance which was either created
                    // through instantiation of a component or through a
                    // synthetic renaming of items we just schlep around the
                    // definitions of various items here.
                    ComponentInstanceDef::Items(map) => match &map[*name] {
                        ComponentItemDef::Func(i) => {
                            frame.component_funcs.push(i.clone());
                        }
                        ComponentItemDef::Module(i) => {
                            frame.modules.push(i.clone());
                        }
                        ComponentItemDef::Component(i) => {
                            frame.components.push(i.clone());
                        }
                        ComponentItemDef::Instance(i) => {
                            let instance = i.clone();
                            frame.component_instances.push(instance);
                        }
                        ComponentItemDef::Type => {
                            // Ignore type aliases for now
                        }
                    },
                }
            }

            // For more information on these see `LexicalScope` but otherwise
            // this is just taking a closed over variable and inserting the
            // actual definition into the local index space since this
            // represents an outer alias to a module/component
            AliasModule(idx) => {
                frame.modules.push(frame.closed_over_module(idx));
            }
            AliasComponent(idx) => {
                frame.components.push(frame.closed_over_component(idx));
            }
        }

        Ok(None)
    }

    /// "Commits" a path of an import to an actual index which is something that
    /// will be calculated at runtime.
    ///
    /// Note that the cost of calculating an item for a `RuntimeImportIndex` at
    /// runtime is amortized with an `InstancePre` which represents "all the
    /// runtime imports are lined up" and after that no more name resolution is
    /// necessary.
    fn runtime_import(&mut self, path: &ImportPath<'a>) -> RuntimeImportIndex {
        *self
            .import_path_interner
            .entry(path.clone())
            .or_insert_with(|| {
                self.result.imports.push((
                    path.index,
                    path.path.iter().map(|s| s.to_string()).collect(),
                ))
            })
    }

    /// Returns the `CoreDef`, the canonical definition for a core wasm item,
    /// for the export `name` of `instance` within `frame`.
    fn core_def_of_module_instance_export(
        &self,
        frame: &InlinerFrame<'a>,
        instance: ModuleInstanceIndex,
        name: &'a str,
    ) -> dfg::CoreDef {
        match &frame.module_instances[instance] {
            // Instantiations of a statically known module means that we can
            // refer to the exported item by a precise index, skipping name
            // lookups at runtime.
            //
            // Instantiations of an imported module, however, must do name
            // lookups at runtime since we don't know the structure ahead of
            // time here.
            ModuleInstanceDef::Instantiated(instance, module) => {
                let item = match frame.modules[*module] {
                    ModuleDef::Static(idx) => {
                        let entity = self.nested_modules[idx].module.exports[name];
                        ExportItem::Index(entity)
                    }
                    ModuleDef::Import(..) => ExportItem::Name(name.to_string()),
                };
                dfg::CoreExport {
                    instance: *instance,
                    item,
                }
                .into()
            }

            // This is a synthetic instance so the canonical definition of the
            // original item is returned.
            ModuleInstanceDef::Synthetic(instance) => match instance[name] {
                EntityIndex::Function(i) => frame.funcs[i].clone(),
                EntityIndex::Table(i) => frame.tables[i].clone().into(),
                EntityIndex::Global(i) => frame.globals[i].clone().into(),
                EntityIndex::Memory(i) => frame.memories[i].clone().into(),
            },
        }
    }

    /// Translates a `LocalCanonicalOptions` which indexes into the `frame`
    /// specified into a runtime representation.
    fn adapter_options(
        &mut self,
        frame: &InlinerFrame<'a>,
        options: &LocalCanonicalOptions,
    ) -> AdapterOptions {
        let memory = options.memory.map(|i| {
            frame.memories[i].clone().map_index(|i| match i {
                EntityIndex::Memory(i) => i,
                _ => unreachable!(),
            })
        });
        let memory64 = match &memory {
            Some(memory) => match &self.runtime_instances[memory.instance] {
                InstanceModule::Static(idx) => match &memory.item {
                    ExportItem::Index(i) => {
                        let plan = &self.nested_modules[*idx].module.memory_plans[*i];
                        plan.memory.memory64
                    }
                    ExportItem::Name(_) => unreachable!(),
                },
                InstanceModule::Import(ty) => match &memory.item {
                    ExportItem::Name(name) => match self.types[*ty].exports[name] {
                        EntityType::Memory(m) => m.memory64,
                        _ => unreachable!(),
                    },
                    ExportItem::Index(_) => unreachable!(),
                },
            },
            None => false,
        };
        let realloc = options.realloc.map(|i| frame.funcs[i].clone());
        let post_return = options.post_return.map(|i| frame.funcs[i].clone());
        AdapterOptions {
            instance: frame.instance,
            string_encoding: options.string_encoding,
            memory,
            memory64,
            realloc,
            post_return,
        }
    }

    /// Translatees an `AdapterOptions` into a `CanonicalOptions` where
    /// memories/functions are inserted into the global initializer list for
    /// use at runtime. This is only used for lowered host functions and lifted
    /// functions exported to the host.
    fn canonical_options(&mut self, options: AdapterOptions) -> dfg::CanonicalOptions {
        let memory = options
            .memory
            .map(|export| self.result.memories.push_uniq(export));
        let realloc = options
            .realloc
            .map(|def| self.result.reallocs.push_uniq(def));
        let post_return = options
            .post_return
            .map(|def| self.result.post_returns.push_uniq(def));
        dfg::CanonicalOptions {
            instance: options.instance,
            string_encoding: options.string_encoding,
            memory,
            realloc,
            post_return,
        }
    }

    fn record_export(
        &mut self,
        name: &str,
        def: ComponentItemDef<'a>,
        map: &mut IndexMap<String, dfg::Export>,
    ) -> Result<()> {
        let export = match def {
            // Exported modules are currently saved in a `PrimaryMap`, at
            // runtime, so an index (`RuntimeModuleIndex`) is assigned here and
            // then an initializer is recorded about where the module comes
            // from.
            ComponentItemDef::Module(module) => match module {
                ModuleDef::Static(idx) => dfg::Export::ModuleStatic(idx),
                ModuleDef::Import(path, _) => dfg::Export::ModuleImport(self.runtime_import(&path)),
            },

            ComponentItemDef::Func(func) => match func {
                // If this is a lifted function from something lowered in this
                // component then the configured options are plumbed through
                // here.
                ComponentFuncDef::Lifted { ty, func, options } => {
                    let options = self.canonical_options(options);
                    dfg::Export::LiftedFunction { ty, func, options }
                }

                // Currently reexported functions from an import are not
                // supported. Being able to actually call these functions is
                // somewhat tricky and needs something like temporary scratch
                // space that isn't implemented.
                ComponentFuncDef::Import(_) => {
                    bail!("component export `{name}` is a reexport of an imported function which is not implemented")
                }
            },

            ComponentItemDef::Instance(instance) => {
                let mut result = IndexMap::new();
                match instance {
                    // If this instance is one that was originally imported by
                    // the component itself then the imports are translated here
                    // by converting to a `ComponentItemDef` and then
                    // recursively recording the export as a reexport.
                    //
                    // Note that for now this would only work with
                    // module-exporting instances.
                    ComponentInstanceDef::Import(path, ty) => {
                        for (name, ty) in self.types[ty].exports.iter() {
                            let mut path = path.clone();
                            path.path.push(name);
                            let def = ComponentItemDef::from_import(path, *ty)?;
                            self.record_export(name, def, &mut result)?;
                        }
                    }

                    // An exported instance which is itself a bag of items is
                    // translated recursively here to our `result` map which is
                    // the bag of items we're exporting.
                    ComponentInstanceDef::Items(map) => {
                        for (name, def) in map {
                            self.record_export(name, def, &mut result)?;
                        }
                    }
                }
                dfg::Export::Instance(result)
            }

            // FIXME(#4283) should make an official decision on whether this is
            // the final treatment of this or not.
            ComponentItemDef::Component(_) => {
                bail!("exporting a component from the root component is not supported")
            }

            ComponentItemDef::Type => {
                // Ignore type exports for now
                return Ok(());
            }
        };

        map.insert(name.to_string(), export);
        Ok(())
    }
}

impl<'a> InlinerFrame<'a> {
    fn new(
        instance: RuntimeComponentInstanceIndex,
        translation: &'a Translation<'a>,
        closure: ComponentClosure<'a>,
        args: HashMap<&'a str, ComponentItemDef<'a>>,
    ) -> Self {
        // FIXME: should iterate over the initializers of `translation` and
        // calculate the size of each index space to use `with_capacity` for
        // all the maps below. Given that doing such would be wordy and compile
        // time is otherwise not super crucial it's not done at this time.
        InlinerFrame {
            instance,
            translation,
            closure,
            args,
            initializers: translation.initializers.iter(),

            funcs: Default::default(),
            memories: Default::default(),
            tables: Default::default(),
            globals: Default::default(),

            component_instances: Default::default(),
            component_funcs: Default::default(),
            module_instances: Default::default(),
            components: Default::default(),
            modules: Default::default(),
        }
    }

    fn item(&self, index: ComponentItem) -> ComponentItemDef<'a> {
        match index {
            ComponentItem::Func(i) => ComponentItemDef::Func(self.component_funcs[i].clone()),
            ComponentItem::Component(i) => ComponentItemDef::Component(self.components[i].clone()),
            ComponentItem::ComponentInstance(i) => {
                ComponentItemDef::Instance(self.component_instances[i].clone())
            }
            ComponentItem::Module(i) => ComponentItemDef::Module(self.modules[i].clone()),
            ComponentItem::Type(_) => ComponentItemDef::Type,
        }
    }

    fn closed_over_module(&self, index: &ClosedOverModule) -> ModuleDef<'a> {
        match *index {
            ClosedOverModule::Local(i) => self.modules[i].clone(),
            ClosedOverModule::Upvar(i) => self.closure.modules[i].clone(),
        }
    }

    fn closed_over_component(&self, index: &ClosedOverComponent) -> ComponentDef<'a> {
        match *index {
            ClosedOverComponent::Local(i) => self.components[i].clone(),
            ClosedOverComponent::Upvar(i) => self.closure.components[i].clone(),
        }
    }
}

impl<'a> ImportPath<'a> {
    fn root(index: ImportIndex) -> ImportPath<'a> {
        ImportPath {
            index,
            path: Vec::new(),
        }
    }
}

impl<'a> ComponentItemDef<'a> {
    fn from_import(path: ImportPath<'a>, ty: TypeDef) -> Result<ComponentItemDef<'a>> {
        let item = match ty {
            TypeDef::Module(ty) => ComponentItemDef::Module(ModuleDef::Import(path, ty)),
            TypeDef::ComponentInstance(ty) => {
                ComponentItemDef::Instance(ComponentInstanceDef::Import(path, ty))
            }
            TypeDef::ComponentFunc(_ty) => ComponentItemDef::Func(ComponentFuncDef::Import(path)),
            // FIXME(#4283) should commit one way or another to how this
            // should be treated.
            TypeDef::Component(_ty) => bail!("root-level component imports are not supported"),
            TypeDef::Interface(_ty) => unimplemented!("import of a type"),
            TypeDef::CoreFunc(_ty) => unreachable!(),
        };
        Ok(item)
    }
}

enum InstanceModule {
    Static(StaticModuleIndex),
    Import(TypeModuleIndex),
}
