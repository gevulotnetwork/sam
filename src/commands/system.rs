use std::{process::Command, sync::Arc};

use parking_lot::Mutex;
use rhai::{EvalAltResult, FnPtr, NativeCallContext, Position};

use crate::{state::SharedState, Environment};

pub fn exec(command: &str) -> Result<String, Box<EvalAltResult>> {
    let output = Command::new("sh")
        .arg("-c")
        .arg(command)
        .envs(std::env::vars())
        .output()
        .map_err(|e| {
            let msg = format!("Failed to execute command: {}", e);
            Box::new(EvalAltResult::ErrorRuntime(msg.into(), Position::NONE))
        })?;
    if !output.status.success() {
        let error = String::from_utf8_lossy(&output.stderr);
        let msg = format!("Command failed with {}: {}", output.status, error);
        return Err(Box::new(EvalAltResult::ErrorRuntime(
            msg.into(),
            Position::NONE,
        )));
    }
    let resp = String::from_utf8(output.stdout).map_err(|e| {
        let msg = format!("Failed to convert output to string: {}", e);
        Box::new(EvalAltResult::ErrorRuntime(msg.into(), Position::NONE))
    })?;
    Ok(resp)
}

pub fn log<E: Environment>(
    context: NativeCallContext,
    state: Arc<Mutex<SharedState<E>>>,
    msg: &str,
) -> Result<(), Box<EvalAltResult>> {
    println!();
    let file = state
        .lock()
        .current_file
        .clone()
        .unwrap_or("unknown".to_string());
    let file = file.split('/').last().unwrap_or("unknown").to_string();
    log::info!(
        "{}:{}: {}",
        file,
        context.position().line().unwrap_or(0),
        msg
    );
    Ok(())
}

pub fn set_env(key: &str, value: &str) -> Result<(), Box<EvalAltResult>> {
    std::env::set_var(key, value);
    Ok(())
}

pub fn get_env(key: &str) -> Result<String, Box<EvalAltResult>> {
    std::env::var(key).map_err(|e| {
        let msg = format!("Failed to get environment variable: {}", e);
        Box::new(EvalAltResult::ErrorRuntime(msg.into(), Position::NONE))
    })
}

pub fn wait_until(
    context: NativeCallContext,
    condition: FnPtr,
    timeout: i64,
) -> Result<(), Box<EvalAltResult>> {
    let start = std::time::Instant::now();
    loop {
        match condition.call_within_context::<bool>(&context, ()) {
            // Condition evaluated to true -> stop waiting and return
            Ok(true) => return Ok(()),
            // Condition evaluated to false -> check timeout and wait if possible
            Ok(false) => {
                if start.elapsed().as_millis() > timeout as u128 {
                    let msg = "Timeout waiting for condition".to_string();
                    return Err(Box::new(EvalAltResult::ErrorRuntime(
                        msg.into(),
                        Position::NONE,
                    )));
                }
                std::thread::sleep(std::time::Duration::from_millis(100));
            }
            // Some error occurred inside condition call -> return it immediately
            Err(err) => {
                return Err(err);
            }
        }
    }
}

pub fn sleep_str(duration: &str) -> Result<(), Box<EvalAltResult>> {
    log::debug!("Sleeping for {}", duration);
    let duration = humantime::parse_duration(duration).map_err(|e| {
        let msg = format!("Invalid duration: {}", e);
        Box::new(EvalAltResult::ErrorRuntime(msg.into(), Position::NONE))
    })?;
    std::thread::sleep(duration);
    Ok(())
}

pub async fn start_component<E: Environment + Clone>(
    state: Arc<Mutex<SharedState<E>>>,
    component: &str,
) -> Result<(), Box<EvalAltResult>> {
    state.lock().env.start_component(component).await.map_err(|e| {
        let msg = format!("Failed to start component: {}", e);
        Box::new(EvalAltResult::ErrorRuntime(msg.into(), Position::NONE))
    })
}

pub async fn stop_component<E: Environment + Clone>(
    state: Arc<Mutex<SharedState<E>>>,
    component: &str,
) -> Result<(), Box<EvalAltResult>> {
    state.lock().env.stop_component(component).await.map_err(|e| {
        let msg = format!("Failed to stop component: {}", e);
        Box::new(EvalAltResult::ErrorRuntime(msg.into(), Position::NONE))
    })
}

