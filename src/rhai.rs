use parking_lot::Mutex;
use rhai::module_resolvers::{FileModuleResolver, ModuleResolversCollection};
use rhai::{Dynamic, Engine as RhaiEngine, EvalAltResult, FnPtr, Position, Scope};
use std::{path::PathBuf, sync::Arc};

use crate::commands::register_commands;
use crate::environment::Environment;
use crate::state::{SharedState, TestReport};

pub struct Engine<E: Environment> {
    engine: RhaiEngine,
    scope: Scope<'static>,
    shared_state: Arc<Mutex<SharedState<E>>>,
}

impl<E: Environment + Clone + 'static> Engine<E> {
    pub fn new(env: E, module_dirs: &[String]) -> Self {
        let mut engine = Engine {
            engine: RhaiEngine::new(),
            scope: Scope::new(),
            shared_state: Arc::new(Mutex::new(SharedState::new(env))),
        };

        engine.shared_state.lock().module_dirs = module_dirs.into();

        engine.engine.set_max_call_levels(256);
        engine.engine.set_max_expr_depths(256, 256);

        let mut resolvers = ModuleResolversCollection::new();
        for module_dir in module_dirs {
            let mut resolver = FileModuleResolver::new();
            resolver.set_base_path(module_dir);
            resolvers.push(resolver);
        }
        engine.engine.set_module_resolver(resolvers);

        register_commands(&mut engine.engine, engine.shared_state.clone());

        engine
    }

    pub fn run_file(&mut self, path: PathBuf) -> Result<(), Box<EvalAltResult>> {
        log::info!("Running script file {}", path.display());
        {
            let mut state = self.shared_state.lock();
            state.current_file = Some(path.display().to_string());
        }
        self.engine.run_file_with_scope(&mut self.scope, path)?;
        {
            let mut state = self.shared_state.lock();
            state.current_file = None;
        }
        Ok(())
    }

    pub fn run_directory(&mut self, path: PathBuf) -> Result<(), Box<EvalAltResult>> {
        for entry in std::fs::read_dir(path).map_err(|e| {
            let msg = format!("Failed to read directory: {}", e);
            Box::new(EvalAltResult::ErrorRuntime(msg.into(), Position::NONE))
        })? {
            if let Ok(entry) = entry {
                let path = entry.path();
                if path.is_file() && path.extension().unwrap_or_default() == "rhai" {
                    self.run_file(path)?;
                }
            }
        }
        Ok(())
    }

    pub fn run(&mut self, path: PathBuf) -> Result<(), Box<EvalAltResult>> {
        if path.is_file() {
            self.run_file(path)
        } else {
            self.run_directory(path)
        }
    }

    pub fn set_filter(&mut self, filter: String) {
        let mut state = self.shared_state.lock();
        state.filter_expression = Some(filter);
    }

    pub fn set_skip(&mut self, skip: String) {
        let mut state = self.shared_state.lock();
        state.skip_expression = Some(skip);
    }

    pub fn get_error_count(&self) -> usize {
        let state = self.shared_state.lock();
        let error_count = state
            .assertions
            .iter()
            .flat_map(|(_, assertions)| assertions.iter().filter(|a| !a.success))
            .count();
        error_count
    }

    pub fn get_report(&self) -> TestReport {
        let state = self.shared_state.lock();
        TestReport::from(&*state)
    }

    pub fn run_fn_ptr(
        &mut self,
        fn_ptr: FnPtr,
        source_file: &str,
    ) -> Result<Dynamic, Box<EvalAltResult>> {
        let ast = self.engine.compile(source_file)?;
        fn_ptr.call(&mut self.engine, &ast, ())
    }
}
