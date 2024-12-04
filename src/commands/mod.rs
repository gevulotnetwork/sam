use std::sync::Arc;

use parking_lot::Mutex;
use rhai::{Array, Dynamic, Engine, EvalAltResult, FnPtr, NativeCallContext, Position};

use crate::environment::Environment;
use crate::state::SharedState;

mod assertions;
mod structure_helpers;
mod system;
mod kv;
mod encoding;
mod fs;
mod http;
mod math;
mod spawn;

pub fn register_commands<E: Environment + Clone + 'static>(
    engine: &mut Engine,
    state: Arc<Mutex<SharedState<E>>>,
) {
    register_structure_helpers(engine, state.clone());
    register_assertions(engine, state.clone());
    register_system(engine, state.clone());
    register_kv(engine, state.clone());
    register_encoding(engine);
    register_fs(engine, state.clone());
    register_http(engine);
    register_math(engine);
    register_spawn(engine, state.clone());
}

fn register_structure_helpers<E: Environment + Clone + 'static>(
    engine: &mut Engine,
    state: Arc<Mutex<SharedState<E>>>,
) {
    let state_clone = state.clone();
    engine.register_fn(
        "describe",
        move |context: NativeCallContext, msg: &str, cb: FnPtr| -> Result<(), Box<EvalAltResult>> {
            structure_helpers::describe::<E>(state_clone.clone(), context, msg, cb, "Testing")
        },
    );

    // alias describe as task
    let state_clone = state.clone();
    engine.register_fn(
        "task",
        move |context: NativeCallContext, msg: &str, cb: FnPtr| -> Result<(), Box<EvalAltResult>> {
            structure_helpers::describe::<E>(state_clone.clone(), context, msg, cb, "Task:")
        },
    );

    let state_clone = state.clone();
    engine.register_fn(
        "it",
        move |context: NativeCallContext, msg: &str, cb: FnPtr| -> Result<(), Box<EvalAltResult>> {
            structure_helpers::it::<E>(state_clone.clone(), context, msg, cb, "It")
        },
    );

    // alias it as step
    let state_clone = state.clone();
    engine.register_fn(
        "step",
        move |context: NativeCallContext, msg: &str, cb: FnPtr| -> Result<(), Box<EvalAltResult>> {
            structure_helpers::it::<E>(state_clone.clone(), context, msg, cb, "Step:")
        },
    );
}

fn register_assertions<E: Environment + Clone + 'static>(
    engine: &mut Engine,
    state: Arc<Mutex<SharedState<E>>>,
) {
    let state_clone = state.clone();
    engine.register_fn(
        "require",
        move |context: NativeCallContext,
              success: bool,
              msg: &str|
              -> Result<(), Box<EvalAltResult>> {
            assertions::require::<E>(state_clone.clone(), context, success, msg)
        },
    );

    let state_clone = state.clone();
    engine.register_fn(
        "assert",
        move |context: NativeCallContext,
              success: bool,
              msg: &str|
              -> Result<(), Box<EvalAltResult>> {
            assertions::assert::<E>(state_clone.clone(), context, success, msg)
        },
    );

    engine.register_fn("diff", move |expected: &str, actual: &str| -> String {
        assertions::diff(expected, actual)
    });
}

fn register_system<E: Environment + Clone + 'static>(
    engine: &mut Engine,
    state: Arc<Mutex<SharedState<E>>>,
) {
    let state_clone = state.clone();
    engine.register_fn(
        "log",
        move |context: NativeCallContext, msg: &str| -> Result<(), Box<EvalAltResult>> {
            system::log::<E>(context, state_clone.clone(), msg)
        },
    );

    engine.register_fn(
        "exec",
        move |command: &str| -> Result<String, Box<EvalAltResult>> { system::exec(command) },
    );

    let state_clone = state.clone();
    engine.register_fn(
        "start_component",
        move |component: &str| -> Result<(), Box<EvalAltResult>> {
            tokio::task::block_in_place(|| {
                tokio::runtime::Handle::current().block_on(system::start_component::<E>(
                    state_clone.clone(),
                    component,
                ))
            })
        },
    );

    let state_clone = state.clone();
    engine.register_fn(
        "stop_component",
        move |component: &str| -> Result<(), Box<EvalAltResult>> {
            tokio::task::block_in_place(|| {
                tokio::runtime::Handle::current().block_on(system::stop_component::<E>(
                    state_clone.clone(),
                    component,
                ))
            })
        },
    );

    engine.register_fn(
        "set_env",
        |key: &str, value: &str| -> Result<(), Box<EvalAltResult>> { system::set_env(key, value) },
    );

    engine.register_fn(
        "get_env",
        |key: &str| -> Result<String, Box<EvalAltResult>> { system::get_env(key) },
    );

    engine.register_fn(
        "sleep",
        |duration: &str| -> Result<(), Box<EvalAltResult>> { system::sleep_str(duration) },
    );

    engine.register_fn(
        "wait_until",
        |context: NativeCallContext,
         condition: FnPtr,
         timeout: i64|
         -> Result<(), Box<EvalAltResult>> {
            system::wait_until(context, condition, timeout)
        },
    );

    engine.register_fn(
        "wait_until",
        |context: NativeCallContext,
         condition: FnPtr,
         timeout: &str|
         -> Result<(), Box<EvalAltResult>> {
            let duration = humantime::parse_duration(timeout).map_err(|e| {
                let msg = format!("Invalid duration: {}", e);
                Box::new(EvalAltResult::ErrorRuntime(msg.into(), Position::NONE))
            })?;
            system::wait_until(context, condition, duration.as_millis() as i64)
        },
    );
}

fn register_kv<E: Environment + Clone + 'static>(
    engine: &mut Engine,
    state: Arc<Mutex<SharedState<E>>>,
) {
    let state_clone = state.clone();
    engine.register_fn(
        "get",
        move |key: &str| -> Result<Dynamic, Box<EvalAltResult>> {
            kv::get::<E>(state_clone.clone(), key)
        },
    );

    let state_clone = state.clone();
    engine.register_fn(
        "set",
        move |key: &str, value: Dynamic| -> Result<(), Box<EvalAltResult>> {
            kv::set::<E>(state_clone.clone(), key, value)
        },
    );
}

fn register_encoding(engine: &mut Engine) {
    engine.register_fn(
        "parse_json",
        |json: &str| -> Result<Dynamic, Box<EvalAltResult>> { encoding::parse_json(json) },
    );

    engine.register_fn(
        "parse_yaml",
        |yaml: &str| -> Result<Dynamic, Box<EvalAltResult>> { encoding::parse_yaml(yaml) },
    );

    engine.register_fn(
        "parse_toml",
        |toml: &str| -> Result<Dynamic, Box<EvalAltResult>> { encoding::parse_toml(toml) },
    );

    engine.register_fn(
        "to_json",
        |value: Dynamic| -> Result<String, Box<EvalAltResult>> { encoding::to_json(&value) },
    );

    engine.register_fn(
        "to_json_pretty",
        |value: Dynamic| -> Result<String, Box<EvalAltResult>> {
            encoding::to_json_pretty(&value)
        },
    );

    engine.register_fn(
        "to_yaml",
        |value: Dynamic| -> Result<String, Box<EvalAltResult>> { encoding::to_yaml(&value) },
    );

    engine.register_fn(
        "to_toml",
        |value: Dynamic| -> Result<String, Box<EvalAltResult>> { encoding::to_toml(&value) },
    );
}

fn register_fs<E: Environment + Clone + 'static>(
    engine: &mut Engine,
    state: Arc<Mutex<SharedState<E>>>,
) {
    let state_clone = state.clone();
    engine.register_fn(
        "temp_dir",
        move |prefix: &str| -> Result<String, Box<EvalAltResult>> {
            fs::temp_dir(state_clone.clone(), prefix)
        },
    );

    engine.register_fn(
        "write_file",
        |path: &str, content: &str| -> Result<(), Box<EvalAltResult>> {
            fs::write_file(path, content)
        },
    );

    engine.register_fn(
        "read_file",
        |path: &str| -> Result<String, Box<EvalAltResult>> { fs::read_file(path) },
    );

    engine.register_fn("mkdir", |path: &str| -> Result<(), Box<EvalAltResult>> {
        fs::mkdir(path)
    });

    engine.register_fn("remove", |path: &str| -> Result<(), Box<EvalAltResult>> {
        fs::remove(path)
    });

    engine.register_fn("ls", |path: &str| -> Result<Array, Box<EvalAltResult>> {
        fs::ls(path)
    });

    engine.register_fn("file_exists", |path: &str| -> bool {
        fs::file_exists(path)
    });

    engine.register_fn("stat", |path: &str| -> Result<Dynamic, Box<EvalAltResult>> {
        fs::stat(path)
    });

    engine.register_fn(
        "copy",
        |src: &str, dst: &str| -> Result<(), Box<EvalAltResult>> {
            fs::copy(src, dst)
        },
    );

    engine.register_fn(
        "rename",
        |src: &str, dst: &str| -> Result<(), Box<EvalAltResult>> {
            fs::rename(src, dst)
        },
    );

    engine.register_fn("is_dir", |path: &str| -> bool {
        fs::is_dir(path)
    });

    engine.register_fn("is_file", |path: &str| -> bool {
        fs::is_file(path)
    });

    engine.register_fn(
        "absolute_path",
        |path: &str| -> Result<String, Box<EvalAltResult>> {
            fs::absolute_path(path)
        },
    );
}

fn register_http(engine: &mut Engine) {
    engine.register_fn(
        "http_get",
        |options: Dynamic| -> Result<String, Box<EvalAltResult>> {
            tokio::task::block_in_place(|| {
                tokio::runtime::Handle::current().block_on(http::http_get(options))
            })
        },
    );

    engine.register_fn(
        "http_post",
        |options: Dynamic| -> Result<String, Box<EvalAltResult>> {
            tokio::task::block_in_place(|| {
                tokio::runtime::Handle::current().block_on(http::http_post(options))
            })
        },
    );

    engine.register_fn(
        "http_head",
        |options: Dynamic| -> Result<(), Box<EvalAltResult>> {
            tokio::task::block_in_place(|| {
                tokio::runtime::Handle::current().block_on(http::http_head(options))
            })
        },
    );
}

fn register_math(engine: &mut Engine) {
    engine.register_fn("random_string", |length: i64| -> String {
        math::random_string(length as usize)
    });

    engine.register_fn("random_int", |min: i64, max: i64| -> i64 {
        math::random_int(min, max)
    });
}

fn register_spawn<E: Environment + Clone + 'static>(
    engine: &mut Engine,
    state: Arc<Mutex<SharedState<E>>>,
) {
    let state_clone = state.clone();
    engine.register_fn(
        "spawn_task",
        move |context: NativeCallContext, cb: FnPtr| -> Result<i64, Box<EvalAltResult>> {
            spawn::spawn_task(state_clone.clone(), context, cb)
        },
    );

    let state_clone = state.clone();
    engine.register_fn(
        "wait_for_tasks",
        move |ids: Array| -> Result<(), Box<EvalAltResult>> {
            spawn::wait_for_tasks(
                state_clone.clone(),
                ids.iter()
                    .map(|id| id.as_int().unwrap())
                    .collect::<Vec<i64>>()
                    .as_slice(),
            )
        },
    );

    let state_clone = state.clone();
    engine.register_fn(
        "wait_for_task",
        move |id: i64| -> Result<(), Box<EvalAltResult>> {
            spawn::wait_for_task(state_clone.clone(), id)
        },
    );
}
