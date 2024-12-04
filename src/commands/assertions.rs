use std::sync::Arc;

use parking_lot::Mutex;
use rhai::{EvalAltResult, NativeCallContext};
use similar_asserts::SimpleDiff;

use crate::{state::{Assertion, SharedState}, Environment};

pub fn require<E: Environment>(
    state: Arc<Mutex<SharedState<E>>>,
    context: NativeCallContext,
    success: bool,
    msg: &str,
) -> Result<(), Box<EvalAltResult>> {
    assert(state, context, success, msg)?;
    if success {
        Ok(())
    } else {
        Err(Box::new(msg.into()))
    }
}

pub fn assert<E: Environment>(
    state: Arc<Mutex<SharedState<E>>>,
    context: NativeCallContext,
    success: bool,
    msg: &str,
) -> Result<(), Box<EvalAltResult>> {
    let mut state = state.lock();
    let assertion_name = state.current_test_stack.join(".") + "/" + msg;
    let assertion = Assertion {
        name: assertion_name,
        success,
        message: msg.to_string(),
        file: state.current_file.clone().unwrap_or("unknown".to_string()),
        line: context.position().line().unwrap_or(0),
    };
    state.push_assertion(assertion);
    if !success {
        state.current_test_failed = true;
    }
    Ok(())
}

pub fn diff(expected: &str, actual: &str) -> String {
    SimpleDiff::from_str(expected, actual, "EXPECTED", "ACTUAL").to_string()
}
