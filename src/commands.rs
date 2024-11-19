use std::{io::Write, marker::PhantomData, process::Command, sync::Arc};

use parking_lot::Mutex;
use rhai::{Dynamic, Engine, EvalAltResult, FnPtr, NativeCallContext, Position};
use similar_asserts::SimpleDiff;

use crate::environment::Environment;
use crate::state::{Assertion, SharedState};

pub struct Commands<E: Environment> {
    _phantom: PhantomData<E>,
}

impl<E: Environment> Commands<E> {
    fn print_indented(msg: &str, indention_level: usize) {
        let prefix = format!(" \x1b[32mTEST\x1b[0m{}", "  ".repeat(indention_level));
        if msg.contains('\n') {
            for line in msg.lines() {
                print!("{}{}\n", prefix, line);
            }
        } else {
            print!("{}{}", prefix, msg);
        }
    }

    pub fn describe(
        state: Arc<Mutex<SharedState<E>>>,
        context: NativeCallContext,
        msg: &str,
        cb: FnPtr,
    ) -> Result<(), Box<EvalAltResult>> {
        let indention_level = {
            let mut state = state.lock();
            let (test_count, error_count) = (state.test_count, state.error_count);
            state.nested_test_counts.push((test_count, error_count));
            state.test_count = 0;
            state.error_count = 0;
            state.indention_level += 1;
            state.current_test_stack.push(msg.to_string());
            state.indention_level
        };

        Self::print_indented(
            &format!("Testing \x1b[3m{}\x1b[0m ...\n", msg),
            indention_level - 1,
        );

        let start = std::time::Instant::now();
        match cb.call_within_context::<()>(&context, ()) {
            Ok(_) => {
                let mut state = state.lock();
                let duration = start.elapsed();
                if state.error_count == 0 && state.test_count > 0 {
                    Self::print_indented(
                        &format!("Testing \x1b[3m{}\x1b[0m \x1b[32msucceeded\x1b[0m! ✅ ({} tests passed) ({})\n", msg, state.test_count, humantime::format_duration(duration)),
                        indention_level - 1
                    );
                } else if state.test_count == 0 {
                    Self::print_indented(
                        &format!(
                            "Testing \x1b[3m{}\x1b[0m \x1b[33mskipped\x1b[0m! ⏭️ (no tests) ({})\n",
                            msg,
                            humantime::format_duration(duration)
                        ),
                        indention_level - 1,
                    );
                } else {
                    Self::print_indented(
                        &format!("Testing \x1b[3m{}\x1b[0m \x1b[31mfailed\x1b[0m! 😭 ({} tests failed out of {}) ({})\n", msg, state.error_count, state.test_count, humantime::format_duration(duration)),
                        indention_level - 1
                    );
                }
                if let Some((parent_tests, parent_errors)) = state.nested_test_counts.pop() {
                    state.test_count += parent_tests;
                    state.error_count += parent_errors;
                }
            }
            Err(e) => {
                let duration = start.elapsed();
                Self::print_indented(
                    &format!(
                        "Testing \x1b[3m{}\x1b[0m \x1b[31mfailed\x1b[0m! 😭: {} ({})\n",
                        msg,
                        e,
                        humantime::format_duration(duration)
                    ),
                    indention_level - 1,
                );
                let mut state = state.lock();
                state.nested_test_counts.pop(); // Clean up the stack on error
            }
        };
        {
            let mut state = state.lock();
            state.indention_level -= 1;
            state.current_test_stack.pop();
        }
        Ok(())
    }

    pub fn it(
        state: Arc<Mutex<SharedState<E>>>,
        context: NativeCallContext,
        msg: &str,
        cb: FnPtr,
    ) -> Result<(), Box<EvalAltResult>> {
        let indention_level = {
            let mut state = state.lock();
            state.current_test_stack.push(msg.to_string());
            if Commands::should_skip(&state) {
                Self::print_indented(
                    &format!("Skipping \x1b[3m{}\x1b[0m ⏭️\n", msg),
                    state.indention_level,
                );
                state.current_test_stack.pop();
                return Ok(());
            }
            state.test_count += 1;
            state.indention_level
        };
        Self::print_indented(&format!("It \x1b[3m{}\x1b[0m...", msg), indention_level);
        std::io::stdout().flush().unwrap();

        let start = std::time::Instant::now();
        let result = cb.call_within_context::<()>(&context, ());
        let duration = start.elapsed();
        let mut state = state.lock();

        match result {
            Ok(_) => {
                if !state.current_test_failed {
                    println!("✅ ({})", humantime::format_duration(duration));
                } else {
                    println!("😭 ({})", humantime::format_duration(duration));
                    state.error_count += 1;
                    for assertion in state
                        .assertions
                        .get(&state.get_current_test_id())
                        .unwrap_or(&vec![])
                        .iter()
                        .filter(|a| !a.success)
                    {
                        Self::print_indented(
                            &format!(
                                "\x1b[3m{}\x1b[0m \x1b[31m(failed)\x1b[0m",
                                assertion.message
                            ),
                            state.indention_level + 1,
                        );
                    }
                }
            }
            Err(e) => {
                let error = e.to_string().replace("\n", " ").replace("  ", " ");
                println!("😭: {} ({})", error, humantime::format_duration(duration));
                for assertion in state
                    .assertions
                    .get(&state.get_current_test_id())
                    .unwrap_or(&vec![])
                    .iter()
                    .filter(|a| !a.success)
                {
                    Self::print_indented(
                        &format!(
                            " - \x1b[3m{}\x1b[0m \x1b[31m(failed)\x1b[0m",
                            assertion.message
                        ),
                        state.indention_level,
                    );
                }
                state.error_count += 1;
            }
        };
        state.current_test_stack.pop();
        state.current_test_failed = false;
        Ok(())
    }

    pub fn require(
        state: Arc<Mutex<SharedState<E>>>,
        context: NativeCallContext,
        success: bool,
        msg: &str,
    ) -> Result<(), Box<EvalAltResult>> {
        Commands::assert(state, context, success, msg)?;
        if success {
            Ok(())
        } else {
            Err(Box::new(msg.into()))
        }
    }

    pub fn assert(
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

    pub fn assert_eq(
        state: Arc<Mutex<SharedState<E>>>,
        context: NativeCallContext,
        expected: Dynamic,
        actual: Dynamic,
        msg: &str,
    ) -> Result<(), Box<EvalAltResult>> {
        let success = expected.to_string() == actual.to_string();
        let mut state = state.lock();
        let assertion_name = state.current_test_stack.join(".") + "/" + msg;
        let mut message = msg.to_string();
        if !success {
            let expected_str = expected.to_string();
            let actual_str = actual.to_string();
            let diff = SimpleDiff::from_str(&expected_str, &actual_str, "EXPECTED", "ACTUAL");
            message = format!("{}:\n{}", msg, diff);
        }
        let assertion = Assertion {
            name: assertion_name,
            success,
            message,
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
            let msg = format!("Command failed with exit code {}: {}", output.status, error);
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

    pub fn log(
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

    pub fn wait_until(
        context: NativeCallContext,
        condition: FnPtr,
        timeout: i64,
    ) -> Result<(), Box<EvalAltResult>> {
        let start = std::time::Instant::now();
        while let Ok(result) = condition.call_within_context::<bool>(&context, ()) {
            if !result {
                if start.elapsed().as_millis() > timeout as u128 {
                    let msg = "Timeout waiting for condition".to_string();
                    return Err(Box::new(EvalAltResult::ErrorRuntime(
                        msg.into(),
                        Position::NONE,
                    )));
                }
                std::thread::sleep(std::time::Duration::from_millis(100));
            } else {
                return Ok(());
            }
        }

        Err(Box::new(EvalAltResult::ErrorRuntime(
            "Failed to evaluate condition".into(),
            Position::NONE,
        )))
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

    pub async fn start_component(
        state: Arc<Mutex<SharedState<E>>>,
        component: &str,
    ) -> Result<(), Box<EvalAltResult>> {
        state
            .lock()
            .env
            .start_component(component)
            .await
            .map_err(|e| {
                let msg = format!("Failed to start component: {}", e);
                Box::new(EvalAltResult::ErrorRuntime(msg.into(), Position::NONE))
            })
    }

    pub async fn stop_component(
        state: Arc<Mutex<SharedState<E>>>,
        component: &str,
    ) -> Result<(), Box<EvalAltResult>> {
        state
            .lock()
            .env
            .stop_component(component)
            .await
            .map_err(|e| {
                let msg = format!("Failed to stop component: {}", e);
                Box::new(EvalAltResult::ErrorRuntime(msg.into(), Position::NONE))
            })
    }

    pub fn should_skip(state: &SharedState<E>) -> bool {
        log::debug!("Checking if we should skip");
        let test_path = state.current_test_stack.join(".");
        log::debug!("Test path: {}", test_path);
        // If there's a skip expression and it matches, we should skip
        if let Some(skip) = &state.skip_expression {
            log::debug!("Skip expression: {}", skip);
            match regex::Regex::new(skip) {
                Ok(re) => {
                    if re.is_match(&test_path) {
                        log::debug!("Skip expression matches test path");
                        return true;
                    }
                }
                Err(_) => {
                    log::debug!("Invalid skip expression: {}", skip);
                    if test_path.contains(skip) {
                        log::debug!("Skip expression matches test path anyway");
                        return true;
                    }
                }
            }
        }

        // If there's a filter expression, skip if it doesn't match
        if let Some(filter) = &state.filter_expression {
            log::debug!("Filter expression: {}", filter);
            match regex::Regex::new(filter) {
                Ok(re) => {
                    log::debug!(
                        "Filter expression matches test path: {}",
                        !re.is_match(&test_path)
                    );
                    !re.is_match(&test_path)
                }
                Err(_) => {
                    log::debug!("Invalid filter expression: {}", filter);
                    if test_path.contains(filter) {
                        log::debug!("Filter expression matches test path anyway");
                        return true;
                    }
                    false
                }
            }
        } else {
            // No filter or skip expressions, don't skip
            log::debug!("No filter or skip expressions, don't skip");
            false
        }
    }
}

pub fn register_commands<E: Environment + 'static>(
    engine: &mut Engine,
    state: Arc<Mutex<SharedState<E>>>,
) {
    let state_clone = state.clone();
    engine.register_fn(
        "describe",
        move |context: NativeCallContext, msg: &str, cb: FnPtr| -> Result<(), Box<EvalAltResult>> {
            Commands::<E>::describe(state_clone.clone(), context, msg, cb)
        },
    );

    let state_clone = state.clone();
    engine.register_fn(
        "it",
        move |context: NativeCallContext, msg: &str, cb: FnPtr| -> Result<(), Box<EvalAltResult>> {
            Commands::<E>::it(state_clone.clone(), context, msg, cb)
        },
    );

    let state_clone = state.clone();
    engine.register_fn(
        "require",
        move |context: NativeCallContext,
              success: bool,
              msg: &str|
              -> Result<(), Box<EvalAltResult>> {
            Commands::<E>::require(state_clone.clone(), context, success, msg)
        },
    );

    let state_clone = state.clone();
    engine.register_fn(
        "assert",
        move |context: NativeCallContext,
              success: bool,
              msg: &str|
              -> Result<(), Box<EvalAltResult>> {
            Commands::<E>::assert(state_clone.clone(), context, success, msg)
        },
    );

    engine.register_fn("diff", move |expected: &str, actual: &str| -> String {
        Commands::<E>::diff(expected, actual)
    });

    let state_clone = state.clone();
    engine.register_fn(
        "log",
        move |context: NativeCallContext, msg: &str| -> Result<(), Box<EvalAltResult>> {
            Commands::<E>::log(context, state_clone.clone(), msg)
        },
    );

    engine.register_fn(
        "exec",
        move |command: &str| -> Result<String, Box<EvalAltResult>> { Commands::<E>::exec(command) },
    );

    let state_clone = state.clone();
    engine.register_fn(
        "start_component",
        move |component: &str| -> Result<(), Box<EvalAltResult>> {
            tokio::task::block_in_place(|| {
                tokio::runtime::Handle::current().block_on(Commands::<E>::start_component(
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
                tokio::runtime::Handle::current().block_on(Commands::<E>::stop_component(
                    state_clone.clone(),
                    component,
                ))
            })
        },
    );

    engine.register_fn(
        "set_env",
        |key: &str, value: &str| -> Result<(), Box<EvalAltResult>> {
            Commands::<E>::set_env(key, value)
        },
    );

    engine.register_fn(
        "sleep",
        |duration: &str| -> Result<(), Box<EvalAltResult>> { Commands::<E>::sleep_str(duration) },
    );

    engine.register_fn(
        "wait_until",
        |context: NativeCallContext,
         condition: FnPtr,
         timeout: i64|
         -> Result<(), Box<EvalAltResult>> {
            Commands::<E>::wait_until(context, condition, timeout)
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
            Commands::<E>::wait_until(context, condition, duration.as_millis() as i64)
        },
    );
}
