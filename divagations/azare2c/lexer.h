#ifndef LEXER_H
#define LEXER_H

#include <string>
#include <vector>

enum State {
    BLOCK,
    MACRO,
    COMMENT
};

struct Token {
    int kind;
    std::string text;
};

enum TokenKind {
    TOK_EOF = 0,
    TOK_TEXT,
    TOK_BLOCK_OPEN,
    TOK_BLOCK_CLOSE,
    TOK_MACRO,
    TOK_VAR,
    TOK_COMMENT_OPEN,
    TOK_COMMENT_CLOSE,
    TOK_LINE_COMMENT,
    TOK_SPECIAL,
    TOK_IDENT,
    TOK_CLOSE_PAREN,
    TOK_COMMA,
    TOK_EQUAL,
    TOK_SPACE,
    TOK_ERROR
};

struct Lexer {
    const char *cursor;
    const char *marker;
    const char *limit;
    const char *token_start;
    std::vector<State> state_stack;

    Lexer(const char *input, size_t len);
    Token next_token();
};

#endif // LEXER_H
