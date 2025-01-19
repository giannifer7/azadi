// azadi/crates/azadi-macros/src/evaluator/tests/test_def.rs
//
// Tests for `%def` macro.

use crate::evaluator::Evaluator;
use crate::types::{ASTNode, NodeKind, Token, TokenKind};

#[test]
fn test_basic_def() {
    let mut ev = Evaluator::new();

    // We emulate calling "%def(myMacro, param, bodyStuff)"
    // => ASTNode kind=Macro, parts => [Text("myMacro"), Text("param"), Text("bodyStuff")]
    let def_call = ASTNode {
        kind: NodeKind::Macro,
        src: 0,
        token: Token {
            kind: TokenKind::Macro,
            src: 0,
            pos: 0,
            length: 3,
        },
        end_pos: 3,
        parts: vec![
            ASTNode {
                kind: NodeKind::Ident,
                src: 0,
                token: Token {
                    kind: TokenKind::Ident,
                    src: 0,
                    pos: 10,
                    length: 7, // "myMacro"
                },
                end_pos: 17,
                parts: vec![],
                name: None,
            },
            ASTNode {
                kind: NodeKind::Ident,
                src: 0,
                token: Token {
                    kind: TokenKind::Ident,
                    src: 0,
                    pos: 20,
                    length: 5, // "param"
                },
                end_pos: 25,
                parts: vec![],
                name: None,
            },
            ASTNode {
                kind: NodeKind::Ident,
                src: 0,
                token: Token {
                    kind: TokenKind::Ident,
                    src: 0,
                    pos: 30,
                    length: 9, // "bodyStuff"
                },
                end_pos: 39,
                parts: vec![],
                name: None,
            },
        ],
        name: None,
    };

    let res = ev.evaluate(&def_call);
    assert!(res.is_ok());
    // check that "myMacro" was defined
    let mm = ev.get_macro("myMacro");
    assert!(mm.is_some());
    let mac = mm.unwrap();
    assert_eq!(mac.params, vec!["param"]);
    assert_eq!(mac.body.kind, NodeKind::Ident);
}
