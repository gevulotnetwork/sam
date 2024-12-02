use std::{collections::HashMap, fmt::Display};

use rhai::Dynamic;
use serde::{Deserialize, Serialize};

use crate::environment::Environment;

pub struct Assertion {
    pub name: String,
    pub success: bool,
    pub message: String,
    pub file: String,
    pub line: usize,
}

#[derive(PartialEq, Eq, Hash)]
pub struct TestId(Vec<String>);

impl TestId {
    pub fn new(test_stack: Vec<String>) -> Self {
        Self(test_stack)
    }
}

impl Display for TestId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0.join("."))
    }
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
    pub assertions: HashMap<TestId, Vec<Assertion>>,
    pub current_test_failed: bool,
    pub silent: bool,
    pub kv_store: HashMap<String, Dynamic>,
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
            silent: false,
            kv_store: HashMap::new(),
            env,
        }
    }

    pub fn get_current_test_id(&self) -> TestId {
        TestId(self.current_test_stack.clone())
    }

    pub fn push_assertion(&mut self, assertion: Assertion) {
        let test_id = self.get_current_test_id();
        self.assertions.entry(test_id).or_default().push(assertion);
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TestReport {
    pub name: String,
    pub success: bool,
    pub error_count: usize,
    pub test_count: usize,
    pub children: Vec<TestReport>,
}

impl From<&Assertion> for TestReport {
    fn from(assertion: &Assertion) -> Self {
        Self::new(assertion.message.clone(), assertion.success)
    }
}

impl TestReport {
    pub fn new(name: String, success: bool) -> Self {
        Self {
            name,
            success,
            error_count: if success { 0 } else { 1 },
            test_count: 1,
            children: vec![],
        }
    }

    pub fn insert(&mut self, path: &TestId, assertions: &Vec<Assertion>) {
        if let Some(head) = path.0.first() {
            let tail = path.0[1..].to_vec();
            if let Some(child) = self.children.iter_mut().find(|c| c.name == *head) {
                child.insert(&TestId(tail), assertions);
            } else {
                let mut report = TestReport::new(head.clone(), true);
                report.insert(&TestId(tail), assertions);
                self.children.push(report);
            }
        } else {
            for assertion in assertions {
                self.children.push(assertion.into());
            }
        }
        self.success = self.children.iter().all(|c| c.success);
        self.error_count = self.children.iter().map(|c| c.error_count).sum();
        self.test_count = self.children.iter().map(|c| c.test_count).sum();
    }
}

impl<E: Environment> From<&SharedState<E>> for TestReport {
    fn from(state: &SharedState<E>) -> Self {
        let mut report = TestReport::new(
            "root".to_string(),
            state.error_count == 0,
        );
        for (test_id, assertions) in &state.assertions {
            report.insert(test_id, assertions);
        }
        report
    }
}

mod tests {
    use crate::{config::Config, MockEnvironment};

    use super::*;

    #[tokio::test]
    async fn test_report_from_state_complex() {
        let mut state = SharedState::new(MockEnvironment {});
        state.current_test_stack.push("test".to_string());
        state.current_test_stack.push("nested".to_string());
        state.current_test_stack.push("grandchild_1".to_string());
        state.push_assertion(Assertion {
            name: "test".to_string(),
            success: true,
            message: "test".to_string(),
            file: "test".to_string(),
            line: 1,
        });
        state.current_test_stack.pop();
        state.current_test_stack.push("grandchild_2".to_string());
        state.push_assertion(Assertion {
            name: "test".to_string(),
            success: false,
            message: "test".to_string(),
            file: "test".to_string(),
            line: 1,
        });
        let report = TestReport::from(&state);
        println!("{:#?}", report);
    }
}
