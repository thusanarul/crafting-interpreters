use thiserror::Error;

use crate::{
    expr::Expr,
    token::{Token, TokenType},
};

pub struct Parser {
    tokens: Vec<Token>,
    current: i32,
}

#[derive(Error, Debug, Clone)]
pub(crate) enum Error {
    #[error("Out of bounds for index {0} in tokens list")]
    OutOfBounds(i32),
    #[error("Empty literal in token {0:?}")]
    EmptyLiteral(Token),
    #[error("Unexpected token: {0:?} in line {1}")]
    UnexpectedToken(Token, i32),
    #[error("Mismatched token: Expected '{expected:?}' and found '{actual:?}' in line {line}.\n{message}")]
    MismatchedToken {
        line: i32,
        expected: TokenType,
        actual: TokenType,
        message: String,
    },
    #[error("Unable to find boundary (keyword or semicolon) when synchronizing parser state")]
    SyncBoundaryNotFound,
}

type PResult<T> = Result<T, Error>;

impl Parser {
    pub fn new(tokens: Vec<Token>) -> Self {
        Self { tokens, current: 0 }
    }

    pub fn parse(&mut self) -> PResult<Expr> {
        self.expression()
    }

    fn expression(&mut self) -> PResult<Expr> {
        self.equality()
    }

    fn equality(&mut self) -> PResult<Expr> {
        let mut _expr = self.comparison()?;

        while self.match_types(vec![TokenType::BangEqual, TokenType::EqualEqual]) {
            // Is there a way to avoid this?
            let operator = self.previous()?.to_owned();
            let right = self.comparison()?;

            _expr = Expr::Binary(_expr.into(), operator, right.into());
        }

        return Ok(_expr);
    }

    fn comparison(&mut self) -> PResult<Expr> {
        let mut _expr = self.term()?;

        while self.match_types(vec![
            TokenType::LessEqual,
            TokenType::Less,
            TokenType::Greater,
            TokenType::GreaterEqual,
        ]) {
            let operator = self.previous()?.to_owned();
            let right = self.term()?;

            _expr = Expr::Binary(_expr.into(), operator, right.into());
        }

        return Ok(_expr);
    }

    fn term(&mut self) -> PResult<Expr> {
        let mut _expr = self.factor()?;

        while self.match_types(vec![TokenType::Minus, TokenType::Plus]) {
            let operator = self.previous()?.to_owned();

            let right = self.factor()?;
            _expr = Expr::Binary(_expr.into(), operator, right.into());
        }

        return Ok(_expr);
    }

    fn factor(&mut self) -> PResult<Expr> {
        let mut _expr = self.unary()?;

        while self.match_types(vec![TokenType::Slash, TokenType::Star]) {
            let operator = self.previous()?.to_owned();

            let right = self.unary()?;
            _expr = Expr::Binary(_expr.into(), operator, right.into());
        }

        return Ok(_expr);
    }

    fn unary(&mut self) -> PResult<Expr> {
        if self.match_types(vec![TokenType::Bang, TokenType::Minus]) {}

        return self.primary();
    }

    fn primary(&mut self) -> PResult<Expr> {
        if self.match_types(vec![TokenType::False, TokenType::True, TokenType::Nil]) {
            let literal = self.previous()?;
            return Ok(Expr::Literal(literal.token_type().into()));
        }

        if self.match_types(vec![TokenType::Number, TokenType::String]) {
            let token = self.previous()?;
            let literal = token
                .literal()
                .ok_or(Error::EmptyLiteral(token.to_owned()))?;

            return Ok(Expr::Literal(literal));
        }

        if self.match_types(vec![TokenType::LeftParen]) {
            let expr = self.expression()?;
            self.consume(
                TokenType::RightParen,
                "Expect ')' after expression.".to_owned(),
            )?;
            return Ok(Expr::Grouping(expr.into()));
        }

        return Err(Error::UnexpectedToken(
            self.peek()?.to_owned(),
            self.current,
        ));
    }

    // NOTE: If token type is matched, the token is consumed with the call to advance()
    fn match_types(&mut self, types: Vec<TokenType>) -> bool {
        for token_type in types {
            if self.check(&token_type) {
                self.advance();
                return true;
            }
        }
        return false;
    }

    fn check(&self, token_type: &TokenType) -> bool {
        if self.is_at_end() || self.peek().is_err() {
            return false;
        }

        return self.peek().unwrap().token_type() == token_type;
    }

    fn advance(&mut self) -> &Token {
        if !self.is_at_end() {
            self.current = self.current + 1
        }
        return self.previous().unwrap();
    }

    fn is_at_end(&self) -> bool {
        if let Ok(value) = self.peek() {
            return value.token_type() == &TokenType::Eof;
        }

        //  If peek() returned OutOfBounds, we consider that we are at the end.s
        return true;
    }

    fn peek(&self) -> PResult<&Token> {
        self.tokens
            .get(self.current as usize)
            .ok_or(Error::OutOfBounds(self.current))
    }

    fn previous(&self) -> PResult<&Token> {
        self.tokens
            .get((self.current - 1) as usize)
            .ok_or(Error::OutOfBounds(self.current - 1))
    }

    fn consume(&mut self, token_type: TokenType, error_message: String) -> PResult<Token> {
        if self.check(&token_type) {
            return Ok(self.advance().clone());
        }

        let actual = self.peek()?.clone();

        return Err(Error::MismatchedToken {
            actual: actual.token_type().clone(),
            expected: token_type,
            line: actual.line().clone(),
            message: error_message,
        });
    }

    fn synchronize(&mut self) -> PResult<()> {
        self.advance();
        while !self.is_at_end() {
            if self.previous()?.token_type() == &TokenType::Semicolon {
                return Ok(());
            }

            match self.peek()?.token_type() {
                TokenType::Class
                | TokenType::Fun
                | TokenType::Var
                | TokenType::For
                | TokenType::If
                | TokenType::While
                | TokenType::Print
                | TokenType::Return => return Ok(()),
                _ => (),
            }

            self.advance();
        }

        Err(Error::SyncBoundaryNotFound)
    }
}
