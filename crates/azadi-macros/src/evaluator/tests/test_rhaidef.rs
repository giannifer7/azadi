// crates/azadi-macros/src/evaluator/tests/test_rhaidef.rs

use crate::evaluator::{eval_string, EvalConfig, Evaluator};

fn evaluator() -> Evaluator {
    Evaluator::new(EvalConfig::default())
}

#[test]
fn test_basic_arithmetic() {
    // Body must be wrapped in %{ %} so azadi doesn't parse parentheses as arg lists
    let mut ev = evaluator();
    let src = r#"%rhaidef(double, x, %{(parse_int(x) * 2).to_string()%})
%double(21)"#;
    let result = eval_string(src, None, &mut ev).expect("eval failed");
    assert_eq!(result.trim(), "42");
}

#[test]
fn test_helper_function_in_body() {
    let mut ev = evaluator();
    let src = r#"%rhaidef(factorial, n, %{
fn fact(k) { if k <= 1 { 1 } else { k * fact(k - 1) } }
fact(parse_int(n)).to_string()
%})
%factorial(5)"#;
    let result = eval_string(src, None, &mut ev).expect("eval failed");
    assert_eq!(result.trim(), "120");
}

#[test]
fn test_variable_capture_from_scope() {
    // Outer azadi scope variables are injected into Rhai scope
    let mut ev = evaluator();
    let src = r#"%set(greeting, Hello)
%rhaidef(greet, name, %{
let g = greeting;
let n = name;
g + ", " + n + "!"
%})
%greet(Rhai)"#;
    let result = eval_string(src, None, &mut ev).expect("eval failed");
    assert_eq!(result.trim(), "Hello, Rhai!");
}

#[test]
fn test_hex_formatting() {
    let mut ev = evaluator();
    let src = r#"%rhaidef(as_hex, n, %{to_hex(parse_int(n))%})
%as_hex(255)"#;
    let result = eval_string(src, None, &mut ev).expect("eval failed");
    assert_eq!(result.trim(), "0xFF");
}

#[test]
fn test_error_propagation() {
    let mut ev = evaluator();
    // @@@ is not valid Rhai syntax
    let src = r#"%rhaidef(broken, x, %{@@@%})
%broken(foo)"#;
    let result = eval_string(src, None, &mut ev);
    assert!(result.is_err(), "expected error from bad Rhai code");
}
