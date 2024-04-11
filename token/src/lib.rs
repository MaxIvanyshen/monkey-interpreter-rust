use std::fmt;

#[derive(Debug)]
pub struct Token {
    pub token_type: TokenType,
    pub literal: String,
}

impl Token {
    pub fn new(token_type: TokenType, literal: String) -> Token {
        Token {
            token_type,
            literal,
        }
    }
}

pub fn lookup_ident(ident: &str) -> TokenType {
    match ident {
        "fn" => TokenType::FUNCTION,
        "let" => TokenType::LET,
        "true" => TokenType::TRUE,
        "false" => TokenType::FALSE,
        "return" => TokenType::RETURN,
        "if" => TokenType::IF,
        "else" => TokenType::ELSE,
        _ => TokenType::IDENT,
    }
}

#[derive(Debug, Hash, PartialEq, Eq, Clone, Copy)]
pub enum TokenType {
    ILLEGAL,
    EOF,

    // Identifiers + literals
    IDENT,
    INT,
    STRING,

    // Operators
    ASSIGN,
    PLUS,
    MINUS,
    ASTERISK,
    SLASH,
    BANG,
    MODULO,

    LT,
    RT,

    EQ,
    NOT_EQ,

    // Delimiters
    COMMA,
    SEMICOLON,

    LPAREN,
    RPAREN,
    LBRACE,
    RBRACE,

    DOUBLE_QUOTE,

    // Keywords
    FUNCTION,
    LET,
    TRUE,
    FALSE,
    RETURN,

    IF,
    ELSE,
}

impl fmt::Display for TokenType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}
