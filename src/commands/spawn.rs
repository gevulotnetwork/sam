use std::sync::Arc;

use parking_lot::Mutex;
use rhai::{Array, Dynamic, EvalAltResult, FnPtr, NativeCallContext, Position};
use tokio::task::JoinHandle;

use crate::{state::SharedState, Environment};

pub fn spawn_task<E: Environment + Clone + 'static>(
    state: Arc<Mutex<SharedState<E>>>,
    _context: NativeCallContext,
    cb: FnPtr,
) -> Result<i64, Box<EvalAltResult>> {
    let (file, mut env, module_dirs) = {
        let state = state.lock();
        (
            state.current_file.clone().unwrap_or_default(),
            state.env.clone(),
            state.module_dirs.clone(),
        )
    };
    env.stop_on_drop(false);
    log::debug!("Spawning task in file: {}", file);
    let mut engine = crate::Engine::new(env, &module_dirs);
    log::debug!("fresh engine created");
    let out: JoinHandle<Result<Dynamic, Box<EvalAltResult>>> = tokio::task::spawn(async move {
        log::debug!("running task in file: {}", file);
        engine.run_fn_ptr(cb, &file)
    });
    log::debug!("task spawned");
    let id = {
        log::debug!("inserting task into state");
        let mut state = state.lock();
        let id = state.spawn_handles.len() as i64;
        state.spawn_handles.insert(id, out);
        log::debug!("task inserted into state");
        id
    };
    log::debug!("task id: {}", id);
    Ok(id)
}

pub fn wait_for_task<E: Environment + Clone + 'static>(
    state: Arc<Mutex<SharedState<E>>>,
    id: i64,
) -> Result<Dynamic, Box<EvalAltResult>> {
    let mut state = state.lock();
    let handle =
        state
            .spawn_handles
            .remove(&id)
            .ok_or(Box::new(EvalAltResult::ErrorRuntime(
                "No such task".into(),
                Position::NONE,
            )))?;
    let result = tokio::task::block_in_place(|| tokio::runtime::Handle::current().block_on(handle))
        .map_err(|e| {
            let msg = format!("Task failed: {}", e);
            state.spawn_handles.remove(&id);
            Box::new(EvalAltResult::ErrorRuntime(msg.into(), Position::NONE))
        })??;
    state.spawn_handles.remove(&id);
    Ok(result)
}

pub fn wait_for_tasks<E: Environment + Clone + 'static>(
    state: Arc<Mutex<SharedState<E>>>,
    ids: &[i64],
) -> Result<Array, Box<EvalAltResult>> {
    let mut results = Array::new();
    for id in ids {
        results.push(wait_for_task(state.clone(), *id)?);
    }
    Ok(results)
}
