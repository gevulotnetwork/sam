use std::sync::Arc;

use parking_lot::Mutex;
use rhai::{Dynamic, EvalAltResult, Position};

use crate::{state::SharedState, Environment};

pub fn set<E: Environment>(
    state: Arc<Mutex<SharedState<E>>>,
    key: &str,
    value: Dynamic,
) -> Result<(), Box<EvalAltResult>> {
    let mut state = state.lock();
    state.kv_store.insert(key.to_string(), value);
    Ok(())
}

pub fn get<E: Environment>(
    state: Arc<Mutex<SharedState<E>>>,
    key: &str,
) -> Result<Dynamic, Box<EvalAltResult>> {
    state
        .lock()
        .kv_store
        .get(key)
        .cloned()
        .ok_or(Box::new(EvalAltResult::ErrorRuntime(
            format!("Key not found: {}", key).into(),
            Position::NONE,
        )))
}