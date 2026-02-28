/*
 * Azadi Lexer - re2c implementation
 * Generated based on Rust implementation.
 * Special character assumed to be '%'.
 */

#include "lexer.h"
#include <iostream>

Lexer::Lexer(const char *input, size_t len) {
    cursor = input;
    limit = input + len;
    marker = input;
    token_start = input;
    state_stack.push_back(BLOCK);
}

Token Lexer::next_token() {
    if (state_stack.empty()) return {TOK_EOF, ""};
    
    State current_state = state_stack.back();
    token_start = cursor;

    // re2c variables
    const char *YYCURSOR = cursor;
    const char *YYMARKER = marker;
    const char *YYLIMIT = limit;
    
    // We use a computed goto or switch based on state
    switch (current_state) {
        case BLOCK: goto yy_BLOCK;
        case MACRO: goto yy_MACRO;
        case COMMENT: goto yy_COMMENT;
    }

yy_BLOCK:
    /*!re2c
    re2c:define:YYCTYPE = "unsigned char";
    re2c:define:YYCURSOR = cursor;
    re2c:define:YYMARKER = marker;
    re2c:define:YYLIMIT = limit;
    re2c:yyfill:enable = 0;

    // Macros
    ident = [a-zA-Z_][a-zA-Z0-9_]*;
    
    * { 
        return {TOK_TEXT, std::string(token_start, cursor)};
    }
    
    "\x00" { return {TOK_EOF, ""}; }

    // Special char sequences
    "%(" ident ")" { return {TOK_VAR, std::string(token_start, cursor)}; }
    "%{" { 
        state_stack.push_back(BLOCK);
        return {TOK_BLOCK_OPEN, "%{"}; 
    }
    "%}" {
        if (state_stack.size() > 1) state_stack.pop_back();
        return {TOK_BLOCK_CLOSE, "%}"};
    }
    "%//" [^\n\x00]* "\n"? { return {TOK_LINE_COMMENT, std::string(token_start, cursor)}; }
    "%--" [^\n\x00]* "\n"? { return {TOK_LINE_COMMENT, std::string(token_start, cursor)}; }
    "%#" [^\n\x00]* "\n"? { return {TOK_LINE_COMMENT, std::string(token_start, cursor)}; }
    "%/*" {
        state_stack.push_back(COMMENT);
        return {TOK_COMMENT_OPEN, "%/*"};
    }
    "%%" { return {TOK_SPECIAL, "%%"}; }
    
    // % + Identifier + ...
    "%" ident "{" {
        state_stack.push_back(BLOCK);
        return {TOK_BLOCK_OPEN, std::string(token_start, cursor)};
    }
    "%" ident "}" {
        if (state_stack.size() > 1) state_stack.pop_back();
        return {TOK_BLOCK_CLOSE, std::string(token_start, cursor)};
    }
    "%" ident "(" {
        state_stack.push_back(MACRO);
        return {TOK_MACRO, std::string(token_start, cursor)};
    }
    
    // Fallback for %
    "%" { return {TOK_TEXT, "%"}; }
    
    // Optimization: Match non-special text in bulk
    [^%\x00]+ { return {TOK_TEXT, std::string(token_start, cursor)}; }

    */

yy_MACRO:
    /*!re2c
    re2c:define:YYCTYPE = "unsigned char";
    re2c:define:YYCURSOR = cursor;
    re2c:define:YYMARKER = marker;
    re2c:define:YYLIMIT = limit;
    re2c:yyfill:enable = 0;

    ")" {
        state_stack.pop_back();
        return {TOK_CLOSE_PAREN, ")"};
    }
    "," { return {TOK_COMMA, ","}; }
    "=" { return {TOK_EQUAL, "="}; }
    [ \t\r\n]+ { return {TOK_SPACE, std::string(token_start, cursor)}; }
    
    "%(" ident ")" { return {TOK_VAR, std::string(token_start, cursor)}; }
    "%{" { 
        state_stack.push_back(BLOCK);
        return {TOK_BLOCK_OPEN, "%{"}; 
    }
    "%}" {
        if (state_stack.size() > 1) state_stack.pop_back();
        return {TOK_BLOCK_CLOSE, "%}"};
    }
    "%//" [^\n\x00]* "\n"? { return {TOK_LINE_COMMENT, std::string(token_start, cursor)}; }
    "%--" [^\n\x00]* "\n"? { return {TOK_LINE_COMMENT, std::string(token_start, cursor)}; }
    "%#" [^\n\x00]* "\n"? { return {TOK_LINE_COMMENT, std::string(token_start, cursor)}; }
    "%/*" {
        state_stack.push_back(COMMENT);
        return {TOK_COMMENT_OPEN, "%/*"};
    }
    "%%" { return {TOK_SPECIAL, "%%"}; }
    
    "%" ident "{" {
        state_stack.push_back(BLOCK);
        return {TOK_BLOCK_OPEN, std::string(token_start, cursor)};
    }
    "%" ident "}" {
        if (state_stack.size() > 1) state_stack.pop_back();
        return {TOK_BLOCK_CLOSE, std::string(token_start, cursor)};
    }
    "%" ident "(" {
        state_stack.push_back(MACRO);
        return {TOK_MACRO, std::string(token_start, cursor)};
    }

    ident { return {TOK_IDENT, std::string(token_start, cursor)}; }
    
    // Fallback for % in macro mode
    "%" { return {TOK_TEXT, "%"}; }

    [^ \t\r\n),=%\x00]+ { return {TOK_TEXT, std::string(token_start, cursor)}; }
    
    "\x00" { return {TOK_EOF, ""}; }
    
    * { return {TOK_TEXT, std::string(token_start, cursor)}; }
    */

yy_COMMENT:
    /*!re2c
    re2c:define:YYCTYPE = "unsigned char";
    re2c:define:YYCURSOR = cursor;
    re2c:define:YYMARKER = marker;
    re2c:define:YYLIMIT = limit;
    re2c:yyfill:enable = 0;

    "%/*" {
        state_stack.push_back(COMMENT);
        return {TOK_COMMENT_OPEN, "%/*"};
    }
    "%*/" {
        state_stack.pop_back();
        return {TOK_COMMENT_CLOSE, "%*/"};
    }
    
    [^%\x00]+ { return {TOK_TEXT, std::string(token_start, cursor)}; }
    
    "%" { return {TOK_TEXT, "%"}; }
    
    "\x00" { return {TOK_EOF, ""}; }
    */
}
