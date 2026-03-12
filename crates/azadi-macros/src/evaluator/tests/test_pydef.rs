// crates/azadi-macros/src/evaluator/tests/test_pydef.rs

#[cfg(feature = "python")]
mod pydef_tests {
    use crate::evaluator::{eval_string, EvalConfig, Evaluator};

    fn evaluator() -> Evaluator {
        Evaluator::new(EvalConfig::default())
    }

    // README example 1: basic arithmetic
    #[test]
    fn test_double() {
        let mut ev = evaluator();
        let src = r#"%pydef(double, x, %{str(int(x) * 2)%})
%double(21)"#;
        let result = eval_string(src, None, &mut ev).expect("eval failed");
        assert_eq!(result.trim(), "42");
    }

    // README example 2: multi-param offset
    #[test]
    fn test_offset() {
        let mut ev = evaluator();
        let src = r#"%pydef(offset, base, size, %{
str(int(base) + int(size))
%})
%offset(256, 64)"#;
        let result = eval_string(src, None, &mut ev).expect("eval failed");
        assert_eq!(result.trim(), "320");
    }

    // README example 3: string concatenation
    #[test]
    fn test_greet() {
        let mut ev = evaluator();
        let src = r#"%pydef(greet, name, %{
"Hello, " + name + "!"
%})
%greet(world)"#;
        let result = eval_string(src, None, &mut ev).expect("eval failed");
        assert_eq!(result.trim(), "Hello, world!");
    }

    // Only declared params are available — azadi scope is not injected
    #[test]
    fn test_only_declared_params_visible() {
        let mut ev = evaluator();
        let src = r#"%set(secret, hidden)
%pydef(echo, x, %{x%})
%echo(visible)"#;
        let result = eval_string(src, None, &mut ev).expect("eval failed");
        assert_eq!(result.trim(), "visible");
    }

    // Error from bad Python propagates
    #[test]
    fn test_error_propagation() {
        let mut ev = evaluator();
        let src = r#"%pydef(broken, x, %{1 / 0%})
%broken(foo)"#;
        let result = eval_string(src, None, &mut ev);
        assert!(result.is_err(), "expected error from division by zero");
    }
}
