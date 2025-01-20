// crates/azadi-macros/src/evaluator/tests/test_ast_builder.rs
use crate::parser::Parser;
use crate::types::{ASTNode, NodeKind, ParseNode, Token, TokenKind};

fn make_token(kind: TokenKind, pos: usize, length: usize) -> Token {
    Token {
        src: 0,
        kind,
        pos,
        length,
    }
}

fn make_node(
    kind: NodeKind,
    token: Token,
    end_pos: usize,
    parts: Vec<usize>,
    parser: &mut Parser,
) -> usize {
    let node = ParseNode {
        kind,
        src: 0,
        token,
        end_pos,
        parts,
    };
    parser.add_node(node)
}

#[test]
fn test_analyze_empty_param() {
    let mut parser = Parser::new();
    let token = make_token(TokenKind::Text, 0, 0);
    let node_index = make_node(NodeKind::Param, token, 0, vec![], &mut parser);
    let result = parser.get_node(node_index).unwrap().analyze_param(&parser);
    assert!(result.is_some());
    let result_node = result.unwrap();
    assert_eq!(result_node.kind, NodeKind::Param);
    assert!(result_node.name.is_none());
    assert!(result_node.parts.is_empty());
}
