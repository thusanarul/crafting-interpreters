use std::fmt::Display;

pub struct Scanner {
    source: Vec<u8>,
}

impl Scanner {
    pub fn new(source: &[u8]) -> Self {
        Scanner {
            source: source.to_vec(),
        }
    }

    pub fn scan_tokens(&self) -> Vec<Token> {
        todo!()
    }
}

#[derive(Debug)]
enum Literal {
    Number(f64),
    String(String),
    // Probably other stuff?
}

pub struct Token {
    token_type: TokenType,
    lexeme: String,
    literal: Literal,
    line: i32,
}

impl Display for Token {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{:?} {:?} {:?}",
            self.token_type, self.lexeme, self.literal
        )
    }
}

#[derive(Debug)]
enum TokenType {
    // Single-character tokens.
    LeftParen,
    RightParen,
    LeftBrace,
    RightBrace,
    Comma,
    Dot,
    Minus,
    Plus,
    Semicolon,
    Slash,
    Star,

    // One or two character tokens.
    Bang,
    BangEqual,
    Equal,
    EqualEqual,
    Greater,
    GreaterEqual,
    Less,
    LessEqual,

    // Literals.
    Identifier,
    String,
    Number,

    // Keywords.
    And,
    Class,
    Else,
    False,
    Fun,
    For,
    If,
    Nil,
    Or,
    Print,
    Return,
    Super,
    This,
    True,
    Var,
    While,

    Eof,
}
