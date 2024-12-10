use std::fmt::Display;

use phf::phf_map;
use thiserror::Error;

#[derive(Error, Debug, Clone)]
pub(crate) enum Error {
    #[error("invalid char: {0}")]
    UnexceptedChar(char),
    #[error("unterminated string at line: {0}")]
    UnterminatedString(i32),
    #[error("unable to parse to float: {0}")]
    ParseError(#[from] std::num::ParseFloatError),
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

static KEYWORDS: phf::Map<&'static str, TokenType> = phf_map! {
    "and" => TokenType::And,
    "class" => TokenType::Class,
    "else" => TokenType::Else,
    "false" => TokenType::False,
    "for" => TokenType::For,
    "fun" => TokenType::Fun,
    "if" => TokenType::If,
    "nil" => TokenType::Nil,
    "or" => TokenType::Or,
    "print" => TokenType::Print,
    "return" => TokenType::Return,
    "super" => TokenType::Super,
    "this" => TokenType::This,
    "true" => TokenType::True,
    "var" => TokenType::Var,
    "while" => TokenType::While
};

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

            if let Err(err) = self.scan_token() {
                self.errors.push(err.clone());
            }
        }

        self.tokens
            .push(Token::new(TokenType::Eof, String::new(), None, 0));

        if !self.errors.is_empty() {
            return Err(self.errors.clone());
        }

        Ok(self.tokens.clone())
    }

    fn scan_token(&mut self) -> Result<(), Error> {
        match self.advance() {
            '(' => self.get_and_add_token(TokenType::LeftParen),
            ')' => self.get_and_add_token(TokenType::RightParen),
            '{' => self.get_and_add_token(TokenType::LeftBrace),
            '}' => self.get_and_add_token(TokenType::RightBrace),
            ',' => self.get_and_add_token(TokenType::Comma),
            '.' => self.get_and_add_token(TokenType::Dot),
            '-' => self.get_and_add_token(TokenType::Minus),
            '+' => self.get_and_add_token(TokenType::Plus),
            ';' => self.get_and_add_token(TokenType::Semicolon),
            '*' => self.get_and_add_token(TokenType::Star),
            '!' => {
                if self.match_char('=') {
                    self.get_and_add_token(TokenType::BangEqual)
                } else {
                    self.get_and_add_token(TokenType::Bang)
                }
            }
            '=' => {
                if self.match_char('=') {
                    self.get_and_add_token(TokenType::EqualEqual)
                } else {
                    self.get_and_add_token(TokenType::Equal)
                }
            }
            '<' => {
                if self.match_char('=') {
                    self.get_and_add_token(TokenType::LessEqual)
                } else {
                    self.get_and_add_token(TokenType::Less)
                }
            }
            '>' => {
                if self.match_char('=') {
                    self.get_and_add_token(TokenType::GreaterEqual)
                } else {
                    self.get_and_add_token(TokenType::Greater)
                }
            }
            '/' => {
                if self.match_char('/') {
                    // A comment goes until the end of the line.
                    while self.peek() != '\n' && !self.is_at_end() {
                        self.advance();
                    }
                } else {
                    self.get_and_add_token(TokenType::Slash)
                }
            }
            ' ' | '\r' | '\t' => {}
            '\n' => {
                self.line = self.line + 1;
            }
            '"' => {
                let token = self.string()?;
                self.add_token(token);
            }
            unknown => {
                if self.is_digit(unknown) {
                    let token = self.number()?;
                    self.add_token(token);
                    return Ok(());
                } else if self.is_alpha(unknown) {
                    let token = self.identifier();
                    self.add_token(token);
                    return Ok(());
                }
                return Err(Error::UnexceptedChar(unknown));
            }
        };

        Ok(())
    }

    fn identifier(&mut self) -> Token {
        while self.is_alphanumeric(self.peek()) {
            self.advance();
        }

        let value = self.source[self.start..self.current].to_owned();
        let token = KEYWORDS.get(&value);

        if let Some(token_type) = token {
            return self.get_token(token_type.to_owned(), None);
        }

        return self.get_token(TokenType::Identifier, None);
    }

    fn is_alphanumeric(&self, c: char) -> bool {
        return self.is_alpha(c) || self.is_digit(c);
    }

    fn is_alpha(&self, c: char) -> bool {
        return c.is_ascii_alphabetic() || c == '_';
    }

    fn number(&mut self) -> Result<Token, Error> {
        while self.is_digit(self.peek()) {
            self.advance();
        }

        if self.peek() == '.' && self.is_digit(self.peek_next()) {
            self.advance();

            while self.is_digit(self.peek()) {
                self.advance();
            }
        }

        let value = self.source[self.start..self.current]
            .parse::<f64>()
            .map_err(|err| Error::ParseError(err))?;

        Ok(self.get_token(TokenType::Number, Some(Literal::Number(value))))
    }

    fn peek_next(&self) -> char {
        if self.current + 1 >= self.source.len() {
            return '\0';
        }

        return self
            .source
            .chars()
            .nth(self.current + 1)
            .expect("Could not get char from string");
    }

    fn is_digit(&self, c: char) -> bool {
        return c >= '0' && c <= '9';
    }

    fn string(&mut self) -> Result<Token, Error> {
        // Consume chars until we hit the '"' that ends the string.
        while self.peek() != '"' && !self.is_at_end() {
            if self.peek() != '\n' {
                self.line = self.line + 1;
            }
            self.advance();
        }

        if self.is_at_end() {
            return Err(Error::UnterminatedString(self.line.clone()));
        }

        self.advance();

        // NOTE: If Lox supported escape sequences like \n, we'd unescape those here.
        let value = self.source[self.start..self.current].to_owned();
        Ok(self.get_token(TokenType::String, Some(Literal::String(value))))
    }

    fn get_token(&self, token_type: TokenType, literal: Option<Literal>) -> Token {
        let lexeme = self.source[self.start..self.current].to_owned();
        return Token::new(token_type, lexeme, literal, self.line);
    }

    fn get_and_add_token(&mut self, token_type: TokenType) {
        let token = self.get_token(token_type, None);
        self.add_token(token);
    }

    fn add_token(&mut self, token: Token) {
        self.tokens.push(token);
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

    fn peek(&self) -> char {
        if self.is_at_end() {
            return '\0';
        }

        return self
            .source
            .chars()
            .nth(self.current)
            .expect("COuld not get char from string");
    }
}

#[derive(Debug, Clone)]
pub enum Literal {
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
pub enum TokenType {
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
