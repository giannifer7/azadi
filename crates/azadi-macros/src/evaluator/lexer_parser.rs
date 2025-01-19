// azadi/crates/azadi-macros/src/evaluator/lexer_parser.rs

use crate::lexer::Lexer;
use crate::parser::Parser;
use crate::types::ASTNode;
pub fn lex_parse_content(source: &str, special_char: char, src: i32) -> Result<ASTNode, String> {
    use std::sync::mpsc::channel;
    let (tx, rx) = channel();
    {
        let mut lexer = Lexer::new(source, special_char, src, tx);
        lexer.run();
        if !lexer.errors.is_empty() {
            let errs = lexer
                .errors
                .iter()
                .map(|e| format!("{:?}", e))
                .collect::<Vec<_>>()
                .join("; ");
            return Err(format!("Lexer errors: {}", errs));
        }
    }
    let tokens: Vec<_> = rx.try_iter().collect();

    let mut parser = Parser::new();
    parser
        .parse(&tokens)
        .map_err(|e| format!("Parse error: {:?}", e))?;

    let ast = parser
        .build_ast()
        .map_err(|e| format!("AST build error: {:?}", e))?;

    Ok(ast)
}
