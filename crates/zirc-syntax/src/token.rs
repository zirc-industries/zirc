//! Token definitions for the Zirc lexer.

/// Kinds of tokens produced by the lexer.
#[derive(Debug, Clone, PartialEq)]
pub enum TokenKind {
    Ident(String),
    Number(i64),
    String(String),
    // keywords
    Fun,
    End,
    If,
    Else,
    While,
    Break,
    Continue,
    Return,
    Let,
    True,
    False,
    // punctuation
    Comma,
    Colon,
    LParen,
    RParen,
    LBracket,
    RBracket,
    // operators
    Equal,      // =
    Plus,       // +
    Minus,      // -
    Star,       // *
    Slash,      // /
    EqEq,       // ==
    NotEq,      // !=
    Less,       // <
    LessEq,     // <=
    Greater,    // >
    GreaterEq,  // >=
    AndAnd,     // &&
    OrOr,       // ||
    Bang,       // !
    Eof,
}

/// A token with its source position (line, col).
#[derive(Debug, Clone, PartialEq)]
pub struct Token {
    pub kind: TokenKind,
    pub line: usize,
    pub col: usize,
}

