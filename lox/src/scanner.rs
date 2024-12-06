use std::fmt::Display;

use thiserror::Error;

#[derive(Error, Debug, Clone, Copy)]
pub(crate) enum Error {
    #[error("invalid char: {0}")]
    UnexceptedChar(char),
}

#[derive(Debug, Clone)]
pub struct Errors(Vec<Error>);

impl Errors {
    fn new() -> Self {
        Self { 0: Vec::new() }
    }

    fn push(&mut self, val: Error) {
        self.0.push(val);
    }

    fn is_empty(&self) -> bool {
        self.0.is_empty()
    }
}

pub struct Scanner {
    source: String,
    tokens: Vec<Token>,
    start: usize,
    current: usize,
    line: i32,
    errors: Errors,
}

impl Scanner {
    pub fn new(source: &[u8]) -> Self {
        Scanner {
            source: String::from_utf8(source.to_owned()).expect("Invalid UTF-8 string"),
            tokens: Vec::new(),
            start: 0,
            current: 0,
            line: 1,
            errors: Errors::new(),
        }
    }

    pub fn scan_tokens(&mut self) -> Result<Vec<Token>, Errors> {
        while !self.is_at_end() {
            // We are at the beginning of the next lexeme.
            self.start = self.current;

            match self.scan_token() {
                Ok(token) => {
                    self.tokens.push(token);
                }
                Err(err) => {
                    self.errors.push(err.clone());
                }
            }
        }

        self.tokens
            .push(Token::new(TokenType::Eof, String::new(), None, 0));

        if !self.errors.is_empty() {
            return Err(self.errors.clone());
        }

        Ok(self.tokens.clone())
    }

    fn scan_token(&mut self) -> Result<Token, Error> {
        let token = match self.advance() {
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
            '!' => {
                if self.match_char('=') {
                    self.get_token(TokenType::BangEqual, None)
                } else {
                    self.get_token(TokenType::Bang, None)
                }
            }
            '=' => {
                if self.match_char('=') {
                    self.get_token(TokenType::EqualEqual, None)
                } else {
                    self.get_token(TokenType::Equal, None)
                }
            }
            '<' => {
                if self.match_char('=') {
                    self.get_token(TokenType::LessEqual, None)
                } else {
                    self.get_token(TokenType::Less, None)
                }
            }
            '>' => {
                if self.match_char('=') {
                    self.get_token(TokenType::GreaterEqual, None)
                } else {
                    self.get_token(TokenType::Greater, None)
                }
            }
            unexpected => return Err(Error::UnexceptedChar(unexpected)),
        };

        Ok(token)
    }

    fn get_token(&self, token_type: TokenType, literal: Option<Literal>) -> Token {
        let lexeme = self.source[self.start..self.current].to_owned();
        return Token::new(token_type, lexeme, literal, self.line);
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

    fn match_char(&mut self, expected: char) -> bool {
        if self.is_at_end() {
            return false;
        }

        let curr_index = self.current;
        let source = self
            .source
            .chars()
            .nth(curr_index)
            .expect("Could not get char from string");

        if source != expected {
            return false;
        }

        self.current = self.current + 1;
        return true;
    }
}

#[derive(Debug, Clone)]
enum Literal {
    Number(f64),
    String(String),
    // Probably other stuff?
}

#[derive(Debug, Clone)]
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

#[derive(Debug, Clone, Copy)]
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
