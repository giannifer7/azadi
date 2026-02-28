lexer grammar AzadiLexer;

options {
    language=Cpp;
}

// Define modes corresponding to Rust states
// Block is the default mode

// -----------------------------------------------------------------------------
// DEFAULT MODE (BLOCK)
// -----------------------------------------------------------------------------

// Variable: %(ident)
VAR: '%(' [a-zA-Z_][a-zA-Z0-9_]* ')' ;

// Block Open: %{ or %ident{
BLOCK_OPEN: '%' ('{' | [a-zA-Z_][a-zA-Z0-9_]* '{') { pushMode(DEFAULT_MODE); } ;

// Block Close: %} or %ident}
BLOCK_CLOSE: '%' ('}' | [a-zA-Z_][a-zA-Z0-9_]* '}') { 
    if (modeStack.size() > 0) popMode(); 
} ;

// Macro Start: %ident(
MACRO_START: '%' [a-zA-Z_][a-zA-Z0-9_]* '(' { pushMode(MACRO_MODE); } ;

// Comments
LINE_COMMENT_SLASH: '%//' ~[\r\n]* -> skip ;
LINE_COMMENT_DASH: '%--' ~[\r\n]* -> skip ;
LINE_COMMENT_HASH: '%#' ~[\r\n]* -> skip ;

COMMENT_OPEN: '%/*' { pushMode(COMMENT_MODE); } -> channel(HIDDEN) ;

// Special Character Escaped
SPECIAL: '%%' ;

// Plain Text in Block
// Match anything that is NOT the start of one of the above.
// ANTLR matches rules in order.
// We need to be careful not to consume '%' if it starts a valid sequence.
// But if it's just '%' followed by something else, it's text.
TEXT: ( ~[%] | '%' ~[({}/#\-] )+ ;


// -----------------------------------------------------------------------------
// MACRO MODE
// -----------------------------------------------------------------------------
mode MACRO_MODE;

MACRO_CLOSE_PAREN: ')' { popMode(); } ;
COMMA: ',' ;
EQUAL: '=' ;
SPACE: [ \t\r\n]+ ;

// Macro mode also supports nested blocks/macros/comments initiated by %
MACRO_VAR: '%(' [a-zA-Z_][a-zA-Z0-9_]* ')' ;

MACRO_BLOCK_OPEN: '%' ('{' | [a-zA-Z_][a-zA-Z0-9_]* '{') { pushMode(DEFAULT_MODE); } ;

MACRO_BLOCK_CLOSE: '%' ('}' | [a-zA-Z_][a-zA-Z0-9_]* '}') { 
    if (modeStack.size() > 0) popMode(); 
} ;

MACRO_NESTED_START: '%' [a-zA-Z_][a-zA-Z0-9_]* '(' { pushMode(MACRO_MODE); } ;

MACRO_LINE_COMMENT_SLASH: '%//' ~[\r\n]* -> skip ;
MACRO_LINE_COMMENT_DASH: '%--' ~[\r\n]* -> skip ;
MACRO_LINE_COMMENT_HASH: '%#' ~[\r\n]* -> skip ;

MACRO_COMMENT_OPEN: '%/*' { pushMode(COMMENT_MODE); } -> channel(HIDDEN) ;

MACRO_SPECIAL: '%%' ;

IDENT: [a-zA-Z_][a-zA-Z0-9_]* ;

// Text in Macro: consume until whitespace, ), ,, =, or %
MACRO_TEXT: ~[ \t\r\n),=%]+ ;


// -----------------------------------------------------------------------------
// COMMENT MODE
// -----------------------------------------------------------------------------
mode COMMENT_MODE;

NESTED_COMMENT: '%/*' { pushMode(COMMENT_MODE); } -> channel(HIDDEN) ;
COMMENT_CLOSE: '%*/' { popMode(); } -> channel(HIDDEN) ;
COMMENT_TEXT: . -> channel(HIDDEN) ;
