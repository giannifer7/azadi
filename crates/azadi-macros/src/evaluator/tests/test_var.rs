// <[@file crates/azadi-macros/src/evaluator/tests/test_var.rs]>=
// azadi/crates/azadi-macros/src/evaluator/tests/test_var.rs

use crate::evaluator::Evaluator;
use crate::types::{ASTNode, NodeKind, Token, TokenKind};

#[test]
fn test_simple_var() {
    let mut ev = Evaluator::new();
    ev.set_variable("foo", "bar");
    let var_node = ASTNode {
        kind: NodeKind::Var,
        src: 0,
        token: Token {
            kind: TokenKind::Var,
            src: 0,
            pos: 0,
            length: 3, // "foo"
        },
        end_pos: 3,
        parts: vec![],
        name: None,
    };
    let out = ev.evaluate(&var_node).unwrap();
    assert_eq!(out, "bar");
}
