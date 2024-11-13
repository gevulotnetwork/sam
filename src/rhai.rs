use parking_lot::Mutex;
use rhai::module_resolvers::{FileModuleResolver, ModuleResolversCollection};
use rhai::{
    Engine as RhaiEngine, EvalAltResult, FnPtr, NativeCallContext, Position, Scope
};
use std::{path::PathBuf, sync::Arc};

use crate::environment::Environment;
use crate::commands::Commands;
pub struct Engine<E: Environment> {
    engine: RhaiEngine,
    scope: Scope<'static>,
    shared_state: Arc<Mutex<SharedState<E>>>,
}

pub struct SharedState<E: Environment> {
    pub indention_level: usize,
    pub test_count: usize,
    pub error_count: usize,
    pub nested_test_counts: Vec<(usize, usize)>, // (test_count, error_count) stack for nested describes
    pub filter_expression: Option<String>,
    pub skip_expression: Option<String>,
    pub current_test_stack: Vec<String>,
    pub current_file: Option<String>,
    pub env: E,

}

impl<E: Environment + 'static> Engine<E> {
    pub fn new(env: E, module_dirs: &[String]) -> Self {
        let mut engine = Engine {
            engine: RhaiEngine::new(),
            scope: Scope::new(),
            shared_state: Arc::new(Mutex::new(SharedState {
                indention_level: 1,
                test_count: 0,
                error_count: 0,
                nested_test_counts: Vec::new(),
                filter_expression: None,
                skip_expression: None,
                current_test_stack: Vec::new(),
                current_file: None,
                env,
            })),
        };

        engine.engine.set_max_call_levels(256);
        engine.engine.set_max_expr_depths(256, 256);

        let mut resolvers = ModuleResolversCollection::new();
        for module_dir in module_dirs {
            let mut resolver = FileModuleResolver::new();
            resolver.set_base_path(module_dir);
            resolvers.push(resolver);
        }
        engine.engine.set_module_resolver(resolvers);

        let state = engine.shared_state.clone();
        engine.engine.register_fn(
            "describe",
            move |context: NativeCallContext, msg: &str, cb: FnPtr| -> Result<(), Box<EvalAltResult>> {
                Commands::describe(state.clone(), context, msg, cb)
            },
        );

        let state = engine.shared_state.clone();
        engine.engine.register_fn(
            "it",
            move|context: NativeCallContext, msg: &str, cb: FnPtr| -> Result<(), Box<EvalAltResult>> {
                Commands::it(state.clone(), context, msg, cb)
            },
        );

        let state = engine.shared_state.clone();
        engine.engine.register_fn(
            "require",
            move|context: NativeCallContext, success: bool, msg: &str| -> Result<(), Box<EvalAltResult>> {
                Commands::require(state.clone(), context, success, msg)
            },
        );

        let state = engine.shared_state.clone();
        engine.engine.register_fn(
            "log",
            move|context: NativeCallContext, msg: &str| -> Result<(), Box<EvalAltResult>> {
                Commands::<E>::log(context, state.clone(), msg)
            },
        );

        engine.engine.register_fn(
            "exec",
            move|command: &str| -> Result<String, Box<EvalAltResult>> {
                Commands::<E>::exec(command)
            },
        );

        let state = engine.shared_state.clone();
        engine.engine.register_fn(
            "start_component",
            move|component: &str| -> Result<(), Box<EvalAltResult>> {
                tokio::task::block_in_place(|| {
                    tokio::runtime::Handle::current().block_on(Commands::<E>::start_component(state.clone(), component))
                })
            },
        );

        let state = engine.shared_state.clone();
        engine.engine.register_fn(
            "stop_component",
            move|component: &str| -> Result<(), Box<EvalAltResult>> {
                tokio::task::block_in_place(|| {
                    tokio::runtime::Handle::current().block_on(Commands::<E>::stop_component(state.clone(), component))
                })
            },
        );

        engine.engine.register_fn(
            "set_env",
            |key: &str, value: &str| -> Result<(), Box<EvalAltResult>> {
                Commands::<E>::set_env(key, value)
            },
        );

        engine.engine.register_fn(
            "sleep", // overload!
            |duration: &str| -> Result<(), Box<EvalAltResult>> {
                log::debug!("calling sleep_str with {}", duration);
                Commands::<E>::sleep_str(duration)
            },
        );

        engine.engine.register_fn(
            "wait_until",
            |context: NativeCallContext, condition: FnPtr, timeout: i64| -> Result<(), Box<EvalAltResult>> {
                Commands::<E>::wait_until(context, condition, timeout)
            },
        );

        engine.engine.register_fn(
            "wait_until",
            |context: NativeCallContext, condition: FnPtr, timeout: &str| -> Result<(), Box<EvalAltResult>> {
                let duration = humantime::parse_duration(timeout).map_err(|e| {
                    let msg = format!("Invalid duration: {}", e);
                    Box::new(EvalAltResult::ErrorRuntime(msg.into(), Position::NONE))
                })?;
                Commands::<E>::wait_until(context, condition, duration.as_millis() as i64)
            },
        );

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
}
