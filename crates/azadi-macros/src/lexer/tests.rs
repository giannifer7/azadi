// crates/azadi-macros/src/lexer/tests.rs

use crate::lexer::Lexer;
use crate::types::{Token, TokenKind};
use std::sync::mpsc::{channel, Receiver};
use std::time::Duration;

/// Collect tokens from the lexer, with a small timeout to prevent hangs.
fn collect_tokens_with_timeout(input: &str) -> Result<Vec<Token>, String> {
    // Create a channel.
    let (sender, receiver) = channel();

    // Convert input to a static str we can safely reference in a thread.
    let input_string = Box::leak(input.to_string().into_boxed_str());

    // Spawn a thread that runs the lexer.
    std::thread::spawn(move || {
        let mut lexer = Lexer::new(input_string, '%', 0, sender);
        lexer.run();
    });

    // Collect tokens until EOF or channel close.
    collect_from_receiver(receiver)
}

/// Helper to read tokens from the receiver with a timeout.
fn collect_from_receiver(receiver: Receiver<Token>) -> Result<Vec<Token>, String> {
    let mut tokens = Vec::new();
    let timeout = Duration::from_secs(2); // or whatever

    loop {
        match receiver.recv_timeout(timeout) {
            Ok(token) => {
                // If we see TokenKind::EOF, return the tokens (no EOF token included).
                if token.kind == TokenKind::EOF {
                    return Ok(tokens);
                }
                tokens.push(token);
            }
            Err(std::sync::mpsc::RecvTimeoutError::Disconnected) => {
                // Channel closed, return what we have.
                return Ok(tokens);
            }
            Err(_) => {
                return Err(format!(
                    "Lexer timed out. Collected {} tokens: {:?}",
                    tokens.len(),
                    tokens
                ));
            }
        }
    }
}

/// Helper to assert tokens match an expected sequence of (TokenKind, &str).
/// We compare both `kind` and the `length` of the text (since we can't store real text easily).
fn assert_tokens(input: &str, expected: &[(TokenKind, &str)]) {
    let result = collect_tokens_with_timeout(input).expect("Failed to collect tokens");
    let tokens = result;

    assert_eq!(
        tokens.len(),
        expected.len(),
        "Wrong number of tokens: expected {}, got {}. Tokens: {:?}",
        expected.len(),
        tokens.len(),
        tokens
    );

    for (i, (token, (exp_kind, exp_text))) in tokens.iter().zip(expected.iter()).enumerate() {
        // Check the kind.
        assert_eq!(
            token.kind, *exp_kind,
            "Token {} kind mismatch: expected {:?}, got {:?}",
            i, exp_kind, token.kind
        );
        // Check the textual length.
        let got_len = token.length;
        let exp_len = exp_text.len();
        assert_eq!(
            got_len, exp_len,
            "Token {} length mismatch: expected {}, got {} (expected text='{}')",
            i, exp_len, got_len, exp_text
        );
    }
}

//-------------------------------------------------------------------------
// Tests
//-------------------------------------------------------------------------

#[test]
fn test_error_cases() {
    // Incomplete block: "%{incomplete" should produce a BlockOpen token and then a Text token.
    assert_tokens(
        "%{incomplete",
        &[
            (TokenKind::BlockOpen, "%{"),
            (TokenKind::Text, "incomplete"),
        ],
    );

    // Incomplete macro: "%macro(incomplete" should produce a Macro token and then an Ident token.
    assert_tokens(
        "%macro(incomplete",
        &[
            (TokenKind::Macro, "%macro("),
            (TokenKind::Ident, "incomplete"),
        ],
    );

    // Unclosed comment: "%/* unfinished" should produce a CommentOpen token and then a Text token.
    assert_tokens(
        "%/* unfinished",
        &[
            (TokenKind::CommentOpen, "%/*"),
            (TokenKind::Text, " unfinished"),
        ],
    );
}

#[test]
fn test_nested_comment() {
    // This input contains a nested comment:
    // Outer comment: starts with "%/*" and ends with "%*/"
    // Inside the outer comment, a nested comment is opened with "%/*" and closed with "%*/".
    let input = "%/* outer comment %/* inner %*/ outer %*/";
    assert_tokens(
        input,
        &[
            // From the block state.
            (TokenKind::CommentOpen, "%/*"),
            // Outer comment text up to the nested comment open.
            (TokenKind::Text, " outer comment "),
            // Nested comment open.
            (TokenKind::CommentOpen, "%/*"),
            // Nested comment text.
            (TokenKind::Text, " inner "),
            // Nested comment close.
            (TokenKind::CommentClose, "%*/"),
            // Outer comment text after the nested comment.
            (TokenKind::Text, " outer "),
            // Outer comment close.
            (TokenKind::CommentClose, "%*/"),
        ],
    );
}

#[test]
fn test_unfinished_special() {
    // Input that does not match any recognized pattern after the special char,
    // so it should be treated as plain text.
    assert_tokens("%something", &[(TokenKind::Text, "%something")]);
}

#[test]
fn test_simple_completion() {
    // Just ensure we don't crash on a single character
    let result = collect_tokens_with_timeout("a");
    assert!(result.is_ok());
}

#[test]
fn test_basic_tokens() {
    // "Hello %name(world)"
    // We expect:
    //   (Text, "Hello "), (Macro, "%name("), (Ident, "world"), (CloseParen, ")")
    assert_tokens(
        "Hello %name(world)",
        &[
            (TokenKind::Text, "Hello "),
            (TokenKind::Macro, "%name("),
            (TokenKind::Ident, "world"),
            (TokenKind::CloseParen, ")"),
        ],
    );
}

#[test]
fn test_comments() {
    // line comment
    assert_tokens(
        "text %// line comment\nmore text",
        &[
            (TokenKind::Text, "text "),
            (TokenKind::LineComment, "%// line comment\n"),
            (TokenKind::Text, "more text"),
        ],
    );

    // block comment
    assert_tokens(
        "before %/* multi\nline %*/ after",
        &[
            (TokenKind::Text, "before "),
            (TokenKind::CommentOpen, "%/*"),
            (TokenKind::Text, " multi\nline "),
            (TokenKind::CommentClose, "%*/"),
            (TokenKind::Text, " after"),
        ],
    );
}

#[test]
fn test_nested_blocks() {
    // nested blocks => %{outer %{inner%}%}
    assert_tokens(
        "%{outer %{inner%}%}",
        &[
            (TokenKind::BlockOpen, "%{"),
            (TokenKind::Text, "outer "),
            (TokenKind::BlockOpen, "%{"),
            (TokenKind::Text, "inner"),
            (TokenKind::BlockClose, "%}"),
            (TokenKind::BlockClose, "%}"),
        ],
    );
}

#[test]
fn test_macro_with_args() {
    // e.g. "%func(a, b, c)"
    assert_tokens(
        "%func(a, b, c)",
        &[
            (TokenKind::Macro, "%func("),
            (TokenKind::Ident, "a"),
            (TokenKind::Comma, ","),
            (TokenKind::Space, " "),
            (TokenKind::Ident, "b"),
            (TokenKind::Comma, ","),
            (TokenKind::Space, " "),
            (TokenKind::Ident, "c"),
            (TokenKind::CloseParen, ")"),
        ],
    );
}

#[test]
fn test_unicode() {
    // if your lexer doesn't do real unicode id parsing, it might treat "名前" as text
    assert_tokens(
        "Hello 世界 %macro(名前)",
        &[
            (TokenKind::Text, "Hello 世界 "),
            (TokenKind::Macro, "%macro("),
            // "名前" might become (Text, "名前") or (Ident, "名前") depending on your code
            (TokenKind::Text, "名前"),
            (TokenKind::CloseParen, ")"),
        ],
    );
}

#[test]
fn test_special_sequences() {
    // double '%'
    assert_tokens(
        "%%double",
        &[(TokenKind::Special, "%%"), (TokenKind::Text, "double")],
    );
}

#[test]
fn test_comment_styles() {
    // multiple line comment styles
    assert_tokens(
        "%# hash comment\n%// double slash\n%-- dash comment",
        &[
            (TokenKind::LineComment, "%# hash comment\n"),
            (TokenKind::LineComment, "%// double slash\n"),
            (TokenKind::LineComment, "%-- dash comment"),
        ],
    );
}

#[test]
fn test_lexer_completion() {
    // empty
    assert_tokens("", &[]);

    // single char
    assert_tokens("a", &[(TokenKind::Text, "a")]);

    // "text%"
    assert_tokens(
        "text%",
        &[(TokenKind::Text, "text"), (TokenKind::Text, "%")],
    );

    // "text %"
    assert_tokens(
        "text %",
        &[(TokenKind::Text, "text "), (TokenKind::Text, "%")],
    );
}

#[test]
fn test_lexer_buffer_boundaries() {
    // %token( rest
    assert_tokens(
        "%token( rest",
        &[
            (TokenKind::Macro, "%token("),
            (TokenKind::Space, " "),
            (TokenKind::Ident, "rest"),
        ],
    );

    // start %token(
    assert_tokens(
        "start %token(",
        &[(TokenKind::Text, "start "), (TokenKind::Macro, "%token(")],
    );

    // " % "
    assert_tokens(
        " % ",
        &[
            (TokenKind::Text, " "),
            (TokenKind::Text, "%"),
            (TokenKind::Text, " "),
        ],
    );
}

#[test]
fn test_leading_trailing_spaces() {
    // Should see (Text, "   Hello   ")
    assert_tokens("   Hello   ", &[(TokenKind::Text, "   Hello   ")]);
}

#[test]
fn test_macro_without_arguments() {
    // e.g. "%macro()"
    assert_tokens(
        "%macro()",
        &[(TokenKind::Macro, "%macro("), (TokenKind::CloseParen, ")")],
    );
}

#[test]
fn test_comment_immediately_following_block() {
    assert_tokens(
        "%{ hi %}%//comment\nleftover",
        &[
            (TokenKind::BlockOpen, "%{"),
            (TokenKind::Text, " hi "),
            (TokenKind::BlockClose, "%}"),
            (TokenKind::LineComment, "%//comment\n"),
            (TokenKind::Text, "leftover"),
        ],
    );
}

#[test]
fn test_multiple_unmatched_percents() {
    // "text % some % more"
    assert_tokens(
        "text % some % more",
        &[
            (TokenKind::Text, "text "),
            (TokenKind::Text, "%"),
            (TokenKind::Text, " some "),
            (TokenKind::Text, "%"),
            (TokenKind::Text, " more"),
        ],
    );
}

#[test]
fn test_unicode_identifier_in_macro() {
    // e.g. "%macro(привет)"
    assert_tokens(
        "%macro(привет)",
        &[
            (TokenKind::Macro, "%macro("),
            // "привет" might be (Text, "привет") or (Ident, "привет")
            (TokenKind::Text, "привет"),
            (TokenKind::CloseParen, ")"),
        ],
    );
}

#[test]
fn test_trailing_whitespace_before_comment() {
    // e.g. "%{ hi %}  %//comment\nleftover"
    assert_tokens(
        "%{ hi %}  %//comment\nleftover",
        &[
            (TokenKind::BlockOpen, "%{"),
            (TokenKind::Text, " hi "),
            (TokenKind::BlockClose, "%}"),
            (TokenKind::Text, "  "),
            (TokenKind::LineComment, "%//comment\n"),
            (TokenKind::Text, "leftover"),
        ],
    );
}

#[test]
fn test_named_block() {
    // e.g. "%blockName{ inside content %blockName}"
    assert_tokens(
        "%blockName{ inside content %blockName}",
        &[
            (TokenKind::BlockOpen, "%blockName{"),
            (TokenKind::Text, " inside content "),
            (TokenKind::BlockClose, "%blockName}"),
        ],
    );
}

#[test]
fn test_simple_var() {
    // e.g. "%(foo)"
    assert_tokens("%(foo)", &[(TokenKind::Var, "%(foo)")]);
}

#[test]
fn test_var_in_block() {
    // e.g. "%{ hello %(abc) world %}"
    assert_tokens(
        "%{ hello %(abc) world %}",
        &[
            (TokenKind::BlockOpen, "%{"),
            (TokenKind::Text, " hello "),
            (TokenKind::Var, "%(abc)"),
            (TokenKind::Text, " world "),
            (TokenKind::BlockClose, "%}"),
        ],
    );
}

#[test]
fn test_var_in_macro() {
    // e.g. "%func( %(myVar), 123 )"
    assert_tokens(
        "%func( %(myVar), 123 )",
        &[
            (TokenKind::Macro, "%func("),
            (TokenKind::Space, " "),
            (TokenKind::Var, "%(myVar)"),
            (TokenKind::Comma, ","),
            (TokenKind::Space, " "),
            (TokenKind::Text, "123"),
            (TokenKind::Space, " "),
            (TokenKind::CloseParen, ")"),
        ],
    );
}

#[test]
fn test_multiple_vars_in_text() {
    // "Here %(x) and %(y) then done"
    assert_tokens(
        "Here %(x) and %(y) then done",
        &[
            (TokenKind::Text, "Here "),
            (TokenKind::Var, "%(x)"),
            (TokenKind::Text, " and "),
            (TokenKind::Var, "%(y)"),
            (TokenKind::Text, " then done"),
        ],
    );
}

#[test]
fn test_incomplete_var() {
    // e.g. "%( %(abc something %( )"
    assert_tokens(
        "%( %(abc something %( )",
        &[
            (TokenKind::Text, "%("),
            (TokenKind::Text, " "),
            (TokenKind::Text, "%(abc"),
            (TokenKind::Text, " something "),
            (TokenKind::Text, "%("),
            (TokenKind::Text, " )"),
        ],
    );
}

#[test]
fn test_real_world_macro_with_block_and_vars() {
    // just an example of a real input
    let input = r#"%def(shortTopCase,  case,  ch, impl, %blk{
// <[Macro_case]>=
case %(ch): {%(impl)}
// $$
%blk})"#;

    assert_tokens(
        input,
        &[
            (TokenKind::Macro, "%def("),
            (TokenKind::Ident, "shortTopCase"),
            (TokenKind::Comma, ","),
            (TokenKind::Space, "  "),
            (TokenKind::Ident, "case"),
            (TokenKind::Comma, ","),
            (TokenKind::Space, "  "),
            (TokenKind::Ident, "ch"),
            (TokenKind::Comma, ","),
            (TokenKind::Space, " "),
            (TokenKind::Ident, "impl"),
            (TokenKind::Comma, ","),
            (TokenKind::Space, " "),
            (TokenKind::BlockOpen, "%blk{"),
            (TokenKind::Text, "\n// <[Macro_case]>=\ncase "),
            (TokenKind::Var, "%(ch)"),
            (TokenKind::Text, ": {"),
            (TokenKind::Var, "%(impl)"),
            (TokenKind::Text, "}\n// $$\n"),
            (TokenKind::BlockClose, "%blk}"),
            (TokenKind::CloseParen, ")"),
        ],
    );
}

#[test]
fn test_escaped_pubfunc_not_macro() {
    // "%%pubfunc(%(name), Allocator* allo, %%{"
    assert_tokens(
        "%%pubfunc(%(name), Allocator* allo, %%{",
        &[
            (TokenKind::Special, "%%"),
            (TokenKind::Text, "pubfunc("),
            (TokenKind::Var, "%(name)"),
            (TokenKind::Text, ", Allocator* allo, "),
            (TokenKind::Special, "%%"),
            (TokenKind::Text, "{"),
        ],
    );
}

#[test]
fn test_no_error() {
    // Just ensure the lexer doesn't produce errors or crash
    let input = "Hello %macro(arg)";
    let tokens_res = collect_tokens_with_timeout(input);

    assert!(tokens_res.is_ok());
    let tokens = tokens_res.unwrap();
    assert!(!tokens.is_empty());
}
