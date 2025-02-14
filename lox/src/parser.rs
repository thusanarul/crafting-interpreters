use thiserror::Error;

use crate::{
    expr::{Expr, Stmt},
    token::{self, Token, TokenType},
};

#[derive(Debug)]
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
    #[error("Invalid assignment target")]
    InvalidAssignmentTarget(Token),
}

type PResult<T> = Result<T, Error>;

// Recursive descent parser
impl Parser {
    pub fn new(tokens: Vec<Token>) -> Self {
        Self { tokens, current: 0 }
    }

    // grammar: -> declaration* EOF
    pub fn parse(&mut self) -> PResult<Vec<Stmt>> {
        let mut statements: Vec<Stmt> = vec![];

        while !self.is_at_end() {
            match self.declaration() {
                Ok(stmt) => statements.push(stmt),
                Err(_) => {
                    self.synchronize()?;
                }
            }
        }

        Ok(statements)
    }

    // grammar: -> varDecl | statement
    fn declaration(&mut self) -> PResult<Stmt> {
        if self.match_type(&TokenType::Var) {
            return self.var_declaration();
        }

        return self.statement();
    }

    // grammar: -> "var" IDENTIFIER ( "=" expression )? ";"
    fn var_declaration(&mut self) -> PResult<Stmt> {
        let name = self.consume(TokenType::Identifier, "Expect variable name.")?;

        let mut initializer = None;

        if self.match_type(&TokenType::Equal) {
            initializer = Some(self.expression()?)
        }

        self.consume(
            TokenType::Semicolon,
            "Expect ';' after variable declaration.",
        )?;

        return Ok(Stmt::Var(name, initializer));
    }

    // grammar: -> exprStmt | forStmt | ifStmt | printStmt | whileStmt | block;
    fn statement(&mut self) -> PResult<Stmt> {
        if self.match_type(&TokenType::If) {
            return self.if_statement();
        }

        if self.match_type(&TokenType::For) {
            return self.for_statement();
        }

        if self.match_type(&TokenType::Print) {
            return self.print_statement();
        }

        if self.match_type(&TokenType::While) {
            return self.while_statement();
        }

        if self.match_type(&TokenType::LeftBrace) {
            return Ok(Stmt::Block(self.block()?));
        }

        self.express_statement()
    }

    // grammar: -> "for" "(" ( varDecl | exprStmt | ";" )
    //              expression? ";"
    //              expression? ")" statement
    // NOTE: Desugars into while_statment
    fn for_statement(&mut self) -> PResult<Stmt> {
        self.consume(TokenType::LeftParen, "Expect '(' after 'for'.")?;

        let mut initializer = None;
        if self.match_type(&TokenType::Semicolon) {
            initializer = None;
        } else if self.match_type(&TokenType::Var) {
            initializer = Some(self.var_declaration()?);
        } else {
            initializer = Some(self.express_statement()?);
        }

        let mut condition = None;
        if !self.check(&TokenType::Semicolon) {
            condition = Some(self.expression()?)
        }

        self.consume(TokenType::Semicolon, "Expect ';' after loop condition.")?;

        let mut increment = None;
        if self.check(&TokenType::RightParen) {
            increment = Some(self.expression()?);
        }

        self.consume(TokenType::Semicolon, "Expect ')' after for clauses.")?;

        let mut body = self.statement()?;

        if let Some(increment) = increment {
            body = Stmt::Block(vec![body, Stmt::Expression(increment)]);
        }

        body = Stmt::While {
            condition: condition.unwrap_or(Expr::Literal(token::Literal::True)),
            body: Box::new(body),
        };

        if let Some(initializer) = initializer {
            body = Stmt::Block(vec![initializer, body]);
        }

        Ok(body)
    }

    // grammar: -> "while" "(" expression ")" statement
    fn while_statement(&mut self) -> PResult<Stmt> {
        self.consume(TokenType::LeftParen, "Expect '(' after 'while'.")?;
        let condition = self.expression()?;
        self.consume(TokenType::LeftParen, "Expect ')' after condition.")?;
        let body = self.statement()?;

        Ok(Stmt::While {
            condition,
            body: Box::new(body),
        })
    }

    // grammar: -> "if" "(" expression ")" statement ( "else" statement )?
    fn if_statement(&mut self) -> PResult<Stmt> {
        self.consume(TokenType::LeftParen, "Expect '(' after 'if'.")?;
        let condition = self.expression()?;
        self.consume(TokenType::RightParen, "Expect ')' after if condition.")?;

        let then_branch = self.statement()?;
        let else_branch = if self.match_type(&TokenType::Else) {
            Some(self.statement()?)
        } else {
            None
        };

        Ok(Stmt::If {
            condition,
            then_branch: Box::new(then_branch),
            else_branch: else_branch.map(|e| Box::new(e)),
        })
    }

    // grammar: -> "{" declaration "}"
    fn block(&mut self) -> PResult<Vec<Stmt>> {
        let mut statements = vec![];

        while !self.check(&TokenType::RightBrace) && !self.is_at_end() {
            statements.push(self.declaration()?);
        }

        self.consume(TokenType::RightBrace, "Expect '}' after block.")?;
        Ok(statements)
    }

    // grammar: -> "print" expression ";"
    fn print_statement(&mut self) -> PResult<Stmt> {
        let value = self.expression()?;
        self.consume(TokenType::Semicolon, "Expect ';' after value.")?;
        Ok(Stmt::Print(value))
    }

    // grammar: -> expression ";"
    fn express_statement(&mut self) -> PResult<Stmt> {
        let value = self.expression()?;
        self.consume(TokenType::Semicolon, "Expect ';' after value.")?;
        Ok(Stmt::Expression(value))
    }

    // grammar: -> assignment
    fn expression(&mut self) -> PResult<Expr> {
        self.assignment()
    }

    // grammar: -> IDENTIFIER "=" assignment | equality | logic_or
    fn assignment(&mut self) -> PResult<Expr> {
        let expr = self.logic_or()?;

        if self.match_type(&TokenType::Equal) {
            let equals = self.previous().map(|e| e.clone())?;
            let value = self.assignment()?;

            if let Expr::Variable(variable) = expr {
                return Ok(Expr::Assign(variable, Box::new(value)));
            }

            return Err(Error::InvalidAssignmentTarget(equals));
        }

        return Ok(expr);
    }

    // grammar: -> logic_and ( "or" logic_and )*
    fn logic_or(&mut self) -> PResult<Expr> {
        let mut expr = self.logic_and()?;

        while self.match_type(&TokenType::Or) {
            let operator = self.previous()?.clone();
            let right = self.logic_and()?;
            expr = Expr::Logical {
                left: Box::new(expr),
                operator,
                right: Box::new(right),
            }
        }

        Ok(expr)
    }

    // grammar: -> equality ( "and" equality )*
    fn logic_and(&mut self) -> PResult<Expr> {
        let mut expr = self.equality()?;

        while self.match_type(&TokenType::Or) {
            let operator = self.previous()?.clone();
            let right = self.equality()?;
            expr = Expr::Logical {
                left: Box::new(expr),
                operator,
                right: Box::new(right),
            }
        }

        Ok(expr)
    }

    // TODO(thusanarul): how does comma and ternary fit into the grammar now?
    // grammar: -> ternary ( ( "," ) ternary )*
    fn comma(&mut self) -> PResult<Expr> {
        let mut expr = self.ternary()?;

        while self.match_type(&TokenType::Comma) {
            let comma_operator = self.previous()?.to_owned();
            let right = self.ternary()?;
            expr = Expr::Binary(expr.into(), comma_operator, right.into())
        }

        return Ok(expr);
    }

    // grammar: -> equality ( ( "?" ) equality ( ":" ) equality )*
    fn ternary(&mut self) -> PResult<Expr> {
        let mut expr = self.equality()?;

        while self.match_types(vec![TokenType::QuestionMark]) {
            let inner_true = self.equality()?;

            self.consume(TokenType::Colon, "Expect ':' after expression")?;

            let inner_false = self.equality()?;

            expr = Expr::Condition(expr.into(), inner_true.into(), inner_false.into())
        }

        return Ok(expr);
    }

    // grammar: -> comparison ( ( "!=" | "==") comparison )* ;
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

    // grammar: -> term ( ( ">" | ">=" | "<" | "<=") term )* ;
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

    // grammar: -> factor ( ( "-" | "+") factor )* ;
    fn term(&mut self) -> PResult<Expr> {
        let mut _expr = self.factor()?;

        while self.match_types(vec![TokenType::Minus, TokenType::Plus]) {
            let operator = self.previous()?.to_owned();

            let right = self.factor()?;
            _expr = Expr::Binary(_expr.into(), operator, right.into());
        }

        return Ok(_expr);
    }

    // grammar: -> unary ( ( "/" | "*") unary )* ;
    fn factor(&mut self) -> PResult<Expr> {
        let mut _expr = self.unary()?;

        while self.match_types(vec![TokenType::Slash, TokenType::Star]) {
            let operator = self.previous()?.to_owned();

            let right = self.unary()?;
            _expr = Expr::Binary(_expr.into(), operator, right.into());
        }

        return Ok(_expr);
    }

    // grammar: -> ("!" | "-") unary | primary ;
    fn unary(&mut self) -> PResult<Expr> {
        if self.match_types(vec![TokenType::Bang, TokenType::Minus]) {}

        return self.primary();
    }

    // grammar: -> NUMBER | STRING | "true" | "false" | "nil" | "(" expression ")" | IDENTIFIER
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

        if self.match_type(&TokenType::Identifier) {
            return Ok(Expr::Variable(self.previous()?.to_owned()));
        }

        if self.match_types(vec![TokenType::LeftParen]) {
            let expr = self.expression()?;
            self.consume(TokenType::RightParen, "Expect ')' after expression.")?;
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

    // NOTE: If token type is matched, the token is consumed with the call to advance()
    fn match_type(&mut self, token_type: &TokenType) -> bool {
        if self.check(token_type) {
            self.advance();
            return true;
        }
        false
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

    fn consume(&mut self, token_type: TokenType, error_message: &str) -> PResult<Token> {
        if self.check(&token_type) {
            return Ok(self.advance().clone());
        }

        let actual = self.peek()?.clone();

        return Err(Error::MismatchedToken {
            actual: actual.token_type().clone(),
            expected: token_type,
            line: actual.line().clone(),
            message: error_message.to_owned(),
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
