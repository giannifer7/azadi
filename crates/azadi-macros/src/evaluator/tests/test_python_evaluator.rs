// crates/azadi-macros/src/evaluator/tests/test_python_evaluator.rs

use super::test_utils::create_test_evaluator;
use crate::evaluator::python::{PythonEvaluator, SecurityLevel};
use std::collections::HashMap;

#[test]
fn test_numeric_calculations() {
    let evaluator = create_test_evaluator();
    let code = r#"
import math
radius = 5
area = math.pi * radius ** 2
print(f"Area of circle with radius {radius}: {area:.2f}")
"#;
    let result = evaluator.evaluate(code, HashMap::new());
    assert!(result.is_ok());
    assert!(result
        .unwrap()
        .contains("Area of circle with radius 5: 78.54"));
}

#[test]
fn test_list_manipulation() {
    let evaluator = create_test_evaluator();
    let code = r#"
numbers = [1, 2, 3, 4, 5]
squared = [x**2 for x in numbers]
print(f"Squared numbers: {squared}")
print(f"Sum of squares: {sum(squared)}")
"#;
    let result = evaluator.evaluate(code, HashMap::new());
    assert!(result.is_ok());
    let output = result.unwrap();
    assert!(output.contains("Squared numbers: [1, 4, 9, 16, 25]"));
    assert!(output.contains("Sum of squares: 55"));
}

#[test]
fn test_with_variables() {
    let evaluator = create_test_evaluator();
    let mut variables = HashMap::new();
    variables.insert("s_min_value".to_string(), "10".to_string());
    variables.insert("s_max_value".to_string(), "20".to_string());

    let code = r#"
min_value = int(s_min_value)
max_value = int(s_max_value)
numbers = list(range(min_value, max_value + 1))
average = sum(numbers) / len(numbers)
print(f"Numbers from {min_value} to {max_value}")
print(f"Average: {average:.1f}")
"#;

    let result = evaluator.evaluate(code, variables);
    assert!(result.is_ok());
    let output = result.unwrap();
    assert!(output.contains("Numbers from 10 to 20"));
    assert!(output.contains("Average: 15.0"));
}

#[test]
fn test_string_processing() {
    let evaluator = create_test_evaluator();
    let mut variables = HashMap::new();
    variables.insert("text".to_string(), "Hello, wonderful World!".to_string());

    let code = r#"
words = text.split()
word_lengths = [len(word) for word in words]
print(f"Word lengths: {word_lengths}")
print(f"Longest word: {max(words, key=len)}")
print(f"Total characters: {sum(word_lengths)}")
"#;

    let result = evaluator.evaluate(code, variables);
    assert!(result.is_ok());
    let output = result.unwrap();
    assert!(output.contains("Word lengths: [6, 9, 6]"));
    assert!(output.contains("Longest word: wonderful"));
}

#[test]
fn test_dict_manipulation() {
    let evaluator = create_test_evaluator();
    let code = r#"
scores = {'Alice': 85, 'Bob': 92, 'Charlie': 78}
print(f"Highest score: {max(scores.values())}")
print(f"Best student: {max(scores, key=scores.get)}")
print(f"Average score: {sum(scores.values()) / len(scores):.1f}")
"#;

    let result = evaluator.evaluate(code, HashMap::new());
    assert!(result.is_ok());
    let output = result.unwrap();
    assert!(output.contains("Highest score: 92"));
    assert!(output.contains("Best student: Bob"));
    assert!(output.contains("Average score: 85.0"));
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
