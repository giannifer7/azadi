// crates/azadi-macros/src/evaluator/tests/test_python_evaluator_pyo3.rs

#[cfg(feature = "pyo3")]
mod pyo3_evaluator_tests {
    use crate::evaluator::python::PythonEvaluator;
    use crate::evaluator::{PyO3Evaluator, PythonConfig, SecurityLevel};
    use std::collections::HashMap;

    fn create_pyo3_evaluator() -> PyO3Evaluator {
        let config = PythonConfig {
            enabled: true,
            venv_path: None,
            python_path: None,
            security_level: SecurityLevel::Basic,
        };
        PyO3Evaluator::new(config).expect("Failed to create PyO3Evaluator")
    }

    #[test]
    fn test_basic_python_execution() {
        let evaluator = create_pyo3_evaluator();
        let code = r#"
x = 42
print(f"The answer is {x}")
        "#;
        let result = evaluator.evaluate(code, HashMap::new());
        assert!(result.is_ok());
        assert_eq!(result.unwrap().trim(), "The answer is 42");
    }

    #[test]
    fn test_variable_passing() {
        let evaluator = create_pyo3_evaluator();
        let mut variables = HashMap::new();
        variables.insert("name".to_string(), "PyO3".to_string());
        variables.insert("version".to_string(), "1.0".to_string());

        let code = r#"
print(f"Hello {name} version {version}!")
        "#;
        let result = evaluator.evaluate(code, variables);
        assert!(result.is_ok());
        assert_eq!(result.unwrap().trim(), "Hello PyO3 version 1.0!");
    }

    #[test]
    fn test_python_computation() {
        let evaluator = create_pyo3_evaluator();
        let code = r#"def factorial(n):
    if n <= 1:
        return 1
    return n * factorial(n - 1)


result = factorial(5)
print(f"Factorial of 5 is {result}")
        "#;
        let result = evaluator.evaluate(code, HashMap::new());
        println!("result = {:?}", result); // Temporary debug
        assert!(result.is_ok());
        assert_eq!(result.unwrap().trim(), "Factorial of 5 is 120");
    }

    #[test]
    fn test_python_error_handling() {
        let evaluator = create_pyo3_evaluator();
        let code = r#"
# This will raise a NameError
print(undefined_variable)
        "#;
        let result = evaluator.evaluate(code, HashMap::new());
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("NameError"));
    }

    #[test]
    fn test_shared_context() {
        let evaluator = create_pyo3_evaluator();

        // First execution sets a value
        let code1 = r#"
shared_context.value = 42
print("Value set")
        "#;
        let result1 = evaluator.evaluate(code1, HashMap::new());
        assert!(result1.is_ok());

        // Second execution reads the value
        let code2 = r#"
print(f"Retrieved value: {shared_context.value}")
        "#;
        let result2 = evaluator.evaluate(code2, HashMap::new());
        assert!(result2.is_ok());
        assert!(result2.unwrap().contains("Retrieved value: 42"));
    }

    #[test]
    fn test_numeric_operations() {
        let evaluator = create_pyo3_evaluator();
        let mut variables = HashMap::new();
        variables.insert("x".to_string(), "10".to_string());
        variables.insert("y".to_string(), "5".to_string());

        let code = r#"
result = int(x) + int(y)
print(f"Sum is {result}")
        "#;
        let result = evaluator.evaluate(code, variables);
        assert!(result.is_ok());
        assert_eq!(result.unwrap().trim(), "Sum is 15");
    }

    #[test]
    fn test_list_manipulation() {
        let evaluator = create_pyo3_evaluator();
        let code = r#"
numbers = [1, 2, 3, 4, 5]
doubled = [x * 2 for x in numbers]
print(f"Doubled numbers: {doubled}")
        "#;
        let result = evaluator.evaluate(code, HashMap::new());
        assert!(result.is_ok());
        assert_eq!(result.unwrap().trim(), "Doubled numbers: [2, 4, 6, 8, 10]");
    }

    #[test]
    fn test_exception_propagation() {
        let evaluator = create_pyo3_evaluator();
        let code = r#"
def divide(a, b):
    return a / b

try:
    result = divide(1, 0)
except ZeroDivisionError as e:
    print(f"Caught error: {e}")
        "#;
        let result = evaluator.evaluate(code, HashMap::new());
        assert!(result.is_ok());
        assert!(result.unwrap().contains("Caught error: division by zero"));
    }

    #[test]
    fn test_multiple_evaluations() {
        let evaluator = create_pyo3_evaluator();

        // First evaluation
        let result1 = evaluator.evaluate("print('First')", HashMap::new());
        assert!(result1.is_ok());
        assert_eq!(result1.unwrap().trim(), "First");

        // Second evaluation
        let result2 = evaluator.evaluate("print('Second')", HashMap::new());
        assert!(result2.is_ok());
        assert_eq!(result2.unwrap().trim(), "Second");
    }
}
