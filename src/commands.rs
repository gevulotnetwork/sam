use std::{io::Write, marker::PhantomData, process::Command, sync::Arc};

use parking_lot::Mutex;
use rhai::{EvalAltResult, FnPtr, NativeCallContext, Position};

use crate::{rhai::SharedState, Environment};


pub struct Commands<E: Environment> {
    _phantom: PhantomData<E>,
}

impl<E: Environment> Commands<E> {
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
        

        println!(" \x1b[32mTEST\x1b[0m{}Testing \x1b[3m{}\x1b[0m ...", "  ".repeat(indention_level-1), msg);
        let start = std::time::Instant::now();
        match cb.call_within_context::<()>(&context, ()) {
            Ok(_) => {
                let mut state = state.lock();
                let duration = start.elapsed();
                if state.error_count == 0 && state.test_count > 0 {
                    println!(" \x1b[32mTEST\x1b[0m{}Testing \x1b[3m{}\x1b[0m \x1b[32msucceeded\x1b[0m! ✅ ({} tests passed) ({})", 
                        "  ".repeat(indention_level-1), msg, state.test_count, humantime::format_duration(duration))
                } else if state.test_count == 0 {
                    println!(" \x1b[32mTEST\x1b[0m{}Testing \x1b[3m{}\x1b[0m \x1b[33mskipped\x1b[0m! ⏭️ (no tests) ({})", 
                        "  ".repeat(indention_level-1), msg, humantime::format_duration(duration))
                }           
                else {
                    println!(" \x1b[32mTEST\x1b[0m{}Testing \x1b[3m{}\x1b[0m \x1b[31mfailed\x1b[0m! 😭 ({} tests failed out of {}) ({})", 
                        "  ".repeat(indention_level-1), msg, state.error_count, state.test_count, humantime::format_duration(duration))                    
                }
                if let Some((parent_tests, parent_errors)) = state.nested_test_counts.pop() {
                    state.test_count += parent_tests;
                    state.error_count += parent_errors;
                }
            },
            Err(e) => {
                let duration = start.elapsed();
                println!(" \x1b[32mTEST\x1b[0m{}Testing \x1b[3m{}\x1b[0m \x1b[31mfailed\x1b[0m! 😭: {} ({})", 
                    "  ".repeat(indention_level-1), msg, e, humantime::format_duration(duration));
                let mut state = state.lock();
                state.nested_test_counts.pop();  // Clean up the stack on error
            },
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
                println!(" \x1b[32mTEST\x1b[0m{}Skipping \x1b[3m{}\x1b[0m ⏭️", "  ".repeat(state.indention_level), msg);
                state.current_test_stack.pop();
                return Ok(());
            }
            state.test_count += 1;
            state.indention_level
        };
        print!(" \x1b[32mTEST\x1b[0m{}It {}... ", "  ".repeat(indention_level), msg);
        std::io::stdout().flush().unwrap();
        
        let start = std::time::Instant::now();
        let result = cb.call_within_context::<()>(&context, ());
        let duration = start.elapsed();

        match result {
            Ok(_) => print!("✅ ({})", humantime::format_duration(duration)),
            Err(e) => {
                let error = e.to_string().replace("\n", " ").replace("  ", " ");
                print!("😭: {} ({})", error, humantime::format_duration(duration));
                let mut state = state.lock();
                state.error_count += 1;
                
            },
        };
        {
            let mut state = state.lock();
            state.current_test_stack.pop();
        }
        println!();
        Ok(())
    }

    pub fn require(
        _state: Arc<Mutex<SharedState<E>>>,
        _context: NativeCallContext,
        success: bool,
        msg: &str,
    ) -> Result<(), Box<EvalAltResult>> {
        if success {
            Ok(())
        } else {
            Err(Box::new(
                msg.into(),
            ))
        }
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
            return Err(Box::new(EvalAltResult::ErrorRuntime(msg.into(), Position::NONE)));
        }
        let resp = String::from_utf8(output.stdout).map_err(|e| {
            let msg = format!("Failed to convert output to string: {}", e);
            Box::new(EvalAltResult::ErrorRuntime(msg.into(), Position::NONE))
        })?;
        Ok(resp)
    }

    pub fn log(context: NativeCallContext, state: Arc<Mutex<SharedState<E>>>, msg: &str) -> Result<(), Box<EvalAltResult>> {
        println!();
        let file = state.lock().current_file.as_ref().map(|e| e.clone()).unwrap_or("unknown".to_string());
        let file = file.split('/').last().unwrap_or("unknown").to_string();
        log::info!("{}:{}: {}", file, context.position().line().unwrap_or(0), msg);
        Ok(())
    }

    pub fn set_env(key: &str, value: &str) -> Result<(), Box<EvalAltResult>> {
        std::env::set_var(key, value);
        Ok(())
    }

    pub fn wait_until(context: NativeCallContext, condition: FnPtr, timeout: i64) -> Result<(), Box<EvalAltResult>> {
        let start = std::time::Instant::now();
        while !condition.call_within_context::<bool>(&context, ()).unwrap() {
            if start.elapsed().as_millis() > timeout as u128 {
                let msg = format!("Timeout waiting for condition");
                return Err(Box::new(EvalAltResult::ErrorRuntime(msg.into(), Position::NONE)));
            }
            std::thread::sleep(std::time::Duration::from_millis(100));
        }
        Ok(())
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

    pub async fn start_component(state: Arc<Mutex<SharedState<E>>>, component: &str) -> Result<(), Box<EvalAltResult>> {
        state.lock().env.start_component(component).await.map_err(|e| {
            let msg = format!("Failed to start component: {}", e);
            Box::new(EvalAltResult::ErrorRuntime(msg.into(), Position::NONE))
        })
    }

    pub async fn stop_component(state: Arc<Mutex<SharedState<E>>>, component: &str) -> Result<(), Box<EvalAltResult>> {
        state.lock().env.stop_component(component).await.map_err(|e| {
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
                },
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
                    log::debug!("Filter expression matches test path: {}", !re.is_match(&test_path));
                    return !re.is_match(&test_path);
                },
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
