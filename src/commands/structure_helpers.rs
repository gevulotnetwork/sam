use std::{io::Write, sync::Arc};

use parking_lot::Mutex;
use rhai::{EvalAltResult, FnPtr, NativeCallContext};

use crate::{state::SharedState, Environment};

pub fn print_indented(msg: &str, indention_level: usize, silent: bool) {
    if silent {
        return;
    }
    let prefix = format!(" \x1b[32mTEST\x1b[0m{}", "  ".repeat(indention_level));
    if msg.contains('\n') {
        for line in msg.lines() {
            println!("{}{}", prefix, line);
        }
    } else {
        print!("{}{}", prefix, msg);
    }
}

pub fn describe<E: Environment>(
    state: Arc<Mutex<SharedState<E>>>,
    context: NativeCallContext,
    msg: &str,
    cb: FnPtr,
    print_prefix: &str,
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

    print_indented(
        &format!("{} \x1b[3m{}\x1b[0m ...\n", print_prefix, msg),
        indention_level - 1,
        state.lock().silent,
    );

    let start = std::time::Instant::now();
    match cb.call_within_context::<()>(&context, ()) {
        Ok(_) => {
            let mut state = state.lock();
            let duration = start.elapsed();
            if state.error_count == 0 && state.test_count > 0 {
                print_indented(
                    &format!("{} \x1b[3m{}\x1b[0m \x1b[32msucceeded\x1b[0m! ✅ ({} tests passed) ({})\n", print_prefix, msg, state.test_count, humantime::format_duration(duration)),
                    indention_level - 1,
                    state.silent,
                );
            } else if state.test_count == 0 {
                print_indented(
                    &format!(
                        "{} \x1b[3m{}\x1b[0m \x1b[33mskipped\x1b[0m! ⏭️ (no tests) ({})\n",
                        print_prefix,
                        msg,
                        humantime::format_duration(duration)
                    ),
                    indention_level - 1,
                    state.silent,
                );
            } else {
                print_indented(
                    &format!(
                        "{} \x1b[3m{}\x1b[0m \x1b[31mfailed\x1b[0m! 😭 ({} tests failed out of {}) ({})\n",
                        print_prefix,
                        msg,
                        state.error_count,
                        state.test_count,
                        humantime::format_duration(duration)
                    ),
                    indention_level - 1,
                    state.silent,
                );
            }
            if let Some((parent_tests, parent_errors)) = state.nested_test_counts.pop() {
                state.test_count += parent_tests;
                state.error_count += parent_errors;
            }
        }
        Err(e) => {
            let duration = start.elapsed();
            print_indented(
                &format!(
                    "{} \x1b[3m{}\x1b[0m \x1b[31mfailed\x1b[0m! 😭: {} ({})\n",
                    print_prefix,
                    msg,
                    e,
                    humantime::format_duration(duration)
                ),
                indention_level - 1,
                state.lock().silent,
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

pub fn it<E: Environment>(
    state: Arc<Mutex<SharedState<E>>>,
    context: NativeCallContext,
    msg: &str,
    cb: FnPtr,
    print_prefix: &str,
) -> Result<(), Box<EvalAltResult>> {
    let indention_level = {
        let mut state = state.lock();
        state.current_test_stack.push(msg.to_string());
        if should_skip(&state) {
            print_indented(
                &format!("{} \x1b[3m{}\x1b[0m ⏭️\n", print_prefix, msg),
                state.indention_level,
                state.silent,
            );
            state.current_test_stack.pop();
            return Ok(());
        }
        state.test_count += 1;
        state.indention_level
    };
    print_indented(
        &format!("{} \x1b[3m{}\x1b[0m...", print_prefix, msg),
        indention_level,
        state.lock().silent,
    );
    std::io::stdout().flush().unwrap();

    let start = std::time::Instant::now();
    let result = cb.call_within_context::<()>(&context, ());
    let duration = start.elapsed();
    let mut state = state.lock();

    match result {
        Ok(_) => {
            if !state.current_test_failed && !state.silent {
                println!("✅ ({})", humantime::format_duration(duration));
            } else if !state.silent {
                println!("😭 ({})", humantime::format_duration(duration));
                state.error_count += 1;
                for assertion in state
                    .assertions
                    .get(&state.get_current_test_id())
                    .unwrap_or(&vec![])
                    .iter()
                    .filter(|a| !a.success)
                {
                    print_indented(
                        &format!(
                            "\x1b[3m{}\x1b[0m \x1b[31m(failed)\x1b[0m\n",
                            assertion.message
                        ),
                        state.indention_level + 1,
                        state.silent,
                    );
                }
            }
        }
        Err(e) => {
            let error = e.to_string().replace("\n", " ").replace("  ", " ");
            if !state.silent {
                println!("😭: {} ({})", error, humantime::format_duration(duration));
            }
            for assertion in state
                .assertions
                .get(&state.get_current_test_id())
                .unwrap_or(&vec![])
                .iter()
                .filter(|a| !a.success)
            {
                print_indented(
                    &format!(
                        " - \x1b[3m{}\x1b[0m \x1b[31m(failed)\x1b[0m",
                        assertion.message
                    ),
                    state.indention_level,
                    state.silent,
                );
            }
            state.error_count += 1;
        }
    };
    state.current_test_stack.pop();
    state.current_test_failed = false;
    Ok(())
}

pub fn should_skip<E: Environment>(state: &SharedState<E>) -> bool {
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