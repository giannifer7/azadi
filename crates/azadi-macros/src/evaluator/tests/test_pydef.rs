// crates/azadi-macros/src/evaluator/tests/test_pydef.rs

use crate::evaluator::python::PythonConfig;
use crate::evaluator::{eval_string, EvalConfig, Evaluator};

fn create_python_config() -> PythonConfig {
    PythonConfig {
        enabled: true,
        venv_path: None,
        python_path: None,
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
