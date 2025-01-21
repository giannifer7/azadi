use crate::macro_api::process_string_defaults;

#[test]
fn test_builtin_capitalize() {
    // Test the %capitalize macro with a direct string input
    let result = process_string_defaults(r#"%capitalize(hello)"#).unwrap();

    // Verify that the first letter is capitalized
    assert_eq!(String::from_utf8(result).unwrap().trim(), "Hello");
}

#[test]
fn test_builtin_decapitalize() {
    // Test the %decapitalize macro with a direct string input
    let result = process_string_defaults(r#"%decapitalize(HELLO)"#).unwrap();

    // Verify that the first letter is lowercased
    assert_eq!(String::from_utf8(result).unwrap().trim(), "hELLO");
}
