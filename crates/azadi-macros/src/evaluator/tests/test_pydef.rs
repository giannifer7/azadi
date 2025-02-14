// crates/azadi-macros/src/evaluator/tests/test_pydef.rs

use super::test_utils::create_test_evaluator;
use crate::evaluator::python::{PythonConfig, PythonEvaluator, SecurityLevel};
use crate::evaluator::{eval_string, EvalConfig, Evaluator};
use std::collections::HashMap;

fn create_python_config() -> PythonConfig {
    PythonConfig {
        enabled: true,
        venv_path: None,
        python_path: None,
        security_level: SecurityLevel::None,
    }
}

#[test]
fn test_basic_pydef() {
    let mut config = EvalConfig::default();
    config.python = create_python_config();
    config.pydef = true;

    let mut evaluator = Evaluator::new(config);

    let source = r#"
%pydef(hello, name, %{
name_str = "%(name)"
print(f"Hello, {name_str}!")
%})
%hello(World)
"#;

    let result = eval_string(source, None, &mut evaluator);
    if let Err(ref e) = result {
        eprintln!("Error running pydef test: {:?}", e);
        eprintln!("Source code was:\n{}", source);
    }
    assert!(result.is_ok(), "Failed to evaluate pydef");
    let output = result.unwrap();
    assert_eq!(output.trim(), "Hello, World!");
}

#[test]
fn test_pydef_with_variables() {
    let mut config = EvalConfig::default();
    config.python = create_python_config();
    config.pydef = true;

    let mut evaluator = Evaluator::new(config);

    let source = r#"
%set(greeting, Hello)
%pydef(greet, name, %{
greeting_str = "%(greeting)"
name_str = "%(name)"
print(f"{greeting_str}, {name_str}!")
%})
%greet(Python)
"#;

    let result = eval_string(source, None, &mut evaluator);
    if let Err(ref e) = result {
        eprintln!("Error running pydef with variables test: {:?}", e);
        eprintln!("Source code was:\n{}", source);
    }
    assert!(result.is_ok(), "Failed to evaluate pydef with variables");
    let output = result.unwrap();
    assert_eq!(output.trim(), "Hello, Python!");
}

#[test]
fn test_security_validation() {
    let evaluator = create_test_evaluator();

    let dangerous_codes = vec![
        ("os_import", "import os\nos.system('ls')"),
        ("file_open", "open('test.txt', 'w')"),
        ("subprocess", "import subprocess\nsubprocess.run(['ls'])"),
        ("eval_code", "eval('2 + 2')"),
    ];

    for (name, code) in dangerous_codes {
        let result = evaluator.validate_code(code, SecurityLevel::Basic);
        assert!(result.is_err(), "Security check should fail for {}", name);

        let result = evaluator.evaluate(code, HashMap::new());
        assert!(result.is_err(), "Evaluation should fail for {}", name);
    }
}
