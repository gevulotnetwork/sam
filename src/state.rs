use std::collections::HashMap;

use crate::environment::Environment;

pub struct Assertion {
    pub name: String,
    pub success: bool,
    pub message: String,
    pub file: String,
    pub line: usize,
}

#[derive(PartialEq, Eq, Hash)]
pub struct TestId(String);

pub struct SharedState<E: Environment> {
    pub indention_level: usize,
    pub test_count: usize,
    pub error_count: usize,
    pub nested_test_counts: Vec<(usize, usize)>, // (test_count, error_count) stack for nested describes
    pub filter_expression: Option<String>,
    pub skip_expression: Option<String>,
    pub current_test_stack: Vec<String>,
    pub current_file: Option<String>,
    pub assertions: HashMap<TestId, Vec<Assertion>>,
    pub current_test_failed: bool,
    pub env: E,
}

impl<E: Environment> SharedState<E> {
    pub fn new(env: E) -> Self {
        Self {
            indention_level: 1,
            test_count: 0,
            error_count: 0,
            nested_test_counts: vec![],
            filter_expression: None,
            skip_expression: None,
            current_test_stack: vec![],
            current_file: None,
            assertions: HashMap::new(),
            current_test_failed: false,
            env,
        }
    }

    pub fn get_current_test_id(&self) -> TestId {
        TestId(self.current_test_stack.join("."))
    }

    pub fn push_assertion(&mut self, assertion: Assertion) {
        let test_id = self.get_current_test_id();
        self.assertions.entry(test_id).or_default().push(assertion);
    }
}
