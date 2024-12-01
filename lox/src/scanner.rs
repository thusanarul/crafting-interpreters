use core::str;
use std::fmt::Display;

pub struct Scanner {
    source: String,
    tokens: Vec<Token>,
    start: usize,
    current: usize,
    line: i32,
}

impl Scanner {
    pub fn new(source: &[u8]) -> Self {
        Scanner {
            source: String::from_utf8(source.to_owned()).expect("Invalid UTF-8 string"),
            tokens: Vec::new(),
            start: 0,
            current: 0,
            line: 1,
        }
    }

    pub fn scan_tokens(&mut self) -> Vec<Token> {
        while !self.is_at_end() {
            // We are at the beginning of the next lexeme.
            self.start = self.current;
            if let Some(token) = self.scan_token() {
                self.tokens.push(token);
            }
        }

        self.tokens
            .push(Token::new(TokenType::Eof, String::new(), None, 0));

        todo!()
    }

    fn scan_token(&mut self) -> Option<Token> {
        let c = self.advance();
        match c {
            '(' => self.get_token(TokenType::LeftParen, None),
            ')' => self.get_token(TokenType::RightParen, None),
            '{' => self.get_token(TokenType::LeftBrace, None),
            '}' => self.get_token(TokenType::RightBrace, None),
            ',' => self.get_token(TokenType::Comma, None),
            '.' => self.get_token(TokenType::Dot, None),
            '-' => self.get_token(TokenType::Minus, None),
            '+' => self.get_token(TokenType::Plus, None),
            ';' => self.get_token(TokenType::Semicolon, None),
            '*' => self.get_token(TokenType::Star, None),
            _ => None,
        }
    }

    fn get_token(&self, token_type: TokenType, literal: Option<Literal>) -> Option<Token> {
        let lexeme = self.source[self.start..self.current].to_owned();
        return Some(Token::new(token_type, lexeme, literal, self.line));
    }

    fn advance(&mut self) -> char {
        let curr_index = self.current;
        let source = self
            .source
            .chars()
            .nth(curr_index)
            .expect("Could not get char from string");
        self.current = self.current + 1;
        return source;
    }

    fn is_at_end(&self) -> bool {
        return self.current >= self.source.len();
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
    literal: Option<Literal>,
    line: i32,
}

impl Token {
    pub fn new(token_type: TokenType, lexeme: String, literal: Option<Literal>, line: i32) -> Self {
        Self {
            token_type,
            lexeme,
            literal,
            line,
        }
    }
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
