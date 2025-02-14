use std::{
    fmt::Display,
    ops::{Add, Div, Mul, Neg, Not, Sub},
};

use thiserror::Error;

use crate::{
    environment::{self, Environment},
    expr::{self, Expr, Stmt, Visitor},
    token::{Literal, Token, TokenType},
};

// NOTE: Difference between Literal and Value
// A literal is something that appears in the user's source code, and is part of the parser's domain.
// A value is produced by computation and don't necessarily exist in the code itself. They are an interpreter concept, part of the runtime world.
#[derive(Debug, Clone)]
pub enum Value {
    Number(f64),
    String(String),
    Bool(bool),
    Nil,
}

impl Value {
    fn number(&self) -> Option<f64> {
        if let Value::Number(n) = self {
            return Some(*n);
        }

        None
    }

    fn string(&self) -> Option<String> {
        if let Value::String(s) = self {
            return Some(s.clone());
        }

        None
    }

    fn is_true(&self) -> bool {
        match self {
            Value::Number(_) => true,
            Value::String(_) => true,
            Value::Bool(b) => *b,
            Value::Nil => false,
        }
    }
}

impl Display for Value {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Value::Number(n) => {
                let mut s = format!("{:.1}", n);
                if s.ends_with(".0") {
                    s = format!("{:}", n);
                }
                write!(f, "{s}")
            }
            Value::String(s) => write!(f, "{s}"),
            Value::Bool(b) => write!(f, "{}", b.to_string()),
            Value::Nil => write!(f, "nil"),
        }
    }
}

#[derive(Error, Debug, Clone)]
pub enum VError {
    // TODO(thusanarul): Make this error msg better and add display to Value to make it more dynamic
    #[error("Cannot apply {operator} operator {operator_type} to {value_type}")]
    InvalidOperation {
        operator: String,
        operator_type: String,
        value_type: String,
    },
}

pub type VResult = Result<Value, VError>;

impl PartialEq for Value {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Value::Number(left), Value::Number(right)) => left == right,
            (Value::String(left), Value::String(right)) => left == right,
            (Value::Bool(left), Value::Bool(right)) => left == right,
            (Value::Nil, Value::Nil) => true,
            (Value::Nil, _) => false,
            (_, _) => false,
        }
    }
}

impl PartialOrd for Value {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        if let (Some(left), Some(right)) = (self.number(), other.number()) {
            return left.partial_cmp(&right);
        }
        None
    }
}

impl Add for Value {
    type Output = VResult;

    fn add(self, rhs: Self) -> Self::Output {
        if let (Some(left), Some(right)) = (self.number(), rhs.number()) {
            return Ok(Value::Number(left + right));
        }

        if let (Some(left), Some(right)) = (self.string(), rhs.string()) {
            return Ok(Value::String(format!("{left}{right}")));
        }

        Err(VError::InvalidOperation {
            operator: "Binary".to_owned(),
            operator_type: "+".to_owned(),
            value_type: format!("{self:?}"),
        })
    }
}

impl Sub for Value {
    type Output = VResult;

    fn sub(self, rhs: Self) -> Self::Output {
        if let (Some(left), Some(right)) = (self.number(), rhs.number()) {
            return Ok(Value::Number(left - right));
        }

        Err(VError::InvalidOperation {
            operator: "Binary".to_owned(),
            operator_type: "-".to_owned(),
            value_type: format!("{self:?}"),
        })
    }
}

impl Div for Value {
    type Output = VResult;

    fn div(self, rhs: Self) -> Self::Output {
        if let (Some(left), Some(right)) = (self.number(), rhs.number()) {
            // TODO(thusanarul): Check if right is zero and report division by zero error. Need to extend VError to support this.
            return Ok(Value::Number(left / right));
        }

        Err(VError::InvalidOperation {
            operator: "Binary".to_owned(),
            operator_type: "/".to_owned(),
            value_type: format!("{self:?}"),
        })
    }
}

impl Mul for Value {
    type Output = VResult;

    fn mul(self, rhs: Self) -> Self::Output {
        if let (Some(left), Some(right)) = (self.number(), rhs.number()) {
            return Ok(Value::Number(left * right));
        }

        Err(VError::InvalidOperation {
            operator: "Binary".to_owned(),
            operator_type: "*".to_owned(),
            value_type: format!("{self:?}"),
        })
    }
}

// Ruby's simple rule: false and nil are falsey. Everything else is truthy.
// So in Not trait impl, everything is just the opposite of the above line.
impl Not for Value {
    type Output = VResult;

    fn not(self) -> Self::Output {
        match self {
            Value::Number(_) => Ok(Value::Bool(false)),
            Value::String(_) => Ok(Value::Bool(false)),
            Value::Bool(b) => Ok(Value::Bool(!b)),
            Value::Nil => Ok(Value::Bool(true)),
        }
    }
}

impl Neg for Value {
    type Output = VResult;

    fn neg(self) -> Self::Output {
        if let Some(left) = self.number() {
            return Ok(Value::Number(-left));
        }
        Err(VError::InvalidOperation {
            operator: "Unary".to_owned(),
            operator_type: "-".to_owned(),
            value_type: "{self:?}".to_owned(),
        })
    }
}

impl From<&Literal> for Value {
    fn from(value: &Literal) -> Self {
        match value {
            Literal::Number(n) => Value::Number(*n),
            Literal::String(s) => Value::String(s.clone()),
            Literal::True => Value::Bool(true),
            Literal::False => Value::Bool(false),
            Literal::Nil => Value::Nil,
        }
    }
}

#[derive(Error, Debug, Clone)]
pub enum IError {
    #[error("Unary op error: {source} at line {}", token.line())]
    UnaryOpError {
        #[source]
        source: VError,
        token: Token,
    },
    #[error("Binary op error: {source} at line {}", token.line())]
    BinaryOpError {
        #[source]
        source: VError,
        token: Token,
    },
    #[error("Environment error: {source} at line {}", token.line())]
    EnvironmentError {
        #[source]
        source: environment::Error,
        token: Token,
    },
    #[error("Reached unexpected state when evaluating token at line {}.", token.line())]
    UnexpectedError { token: Token },
}

impl IError {
    fn unary_op_error(err: VError, token: &Token) -> Self {
        Self::UnaryOpError {
            source: err,
            token: token.clone(),
        }
    }

    fn binary_op_error(err: VError, token: &Token) -> Self {
        Self::BinaryOpError {
            source: err,
            token: token.clone(),
        }
    }

    fn environment_error(err: environment::Error, token: &Token) -> Self {
        Self::EnvironmentError {
            source: err,
            token: token.clone(),
        }
    }
}

type IResult<V> = Result<V, IError>;

pub struct Interpreter {
    environment: Environment,
}

impl Interpreter {
    pub fn new() -> Self {
        Self {
            environment: Environment::new(None),
        }
    }

    pub fn interpret(&mut self, stmts: &Vec<Stmt>) {
        for stmt in stmts {
            if let Err(err) = self.visit_stmt(stmt) {
                eprintln!("{err}");
            }
        }
    }

    fn interpret_literal(&self, literal: &Literal) -> IResult<Value> {
        Ok(literal.into())
    }

    fn interpret_grouping(&mut self, expr: &Expr) -> IResult<Value> {
        self.visit_expr(expr)
    }

    fn interpret_unary(&mut self, token: &Token, right: &Expr) -> IResult<Value> {
        let right = self.visit_expr(right)?;
        let operator = token.token_type();

        // TODO(thusanarul): verify this works
        match operator {
            TokenType::Bang => {
                let new_value = !right;
                new_value.map_err(|err| IError::unary_op_error(err, &token))
            }
            TokenType::Minus => {
                let new_value = -right;
                new_value.map_err(|err| IError::unary_op_error(err, &token))
            }
            _ => Err(IError::UnexpectedError {
                token: token.clone(),
            }),
        }
    }

    fn interpret_binary(&mut self, token: &Token, left: &Expr, right: &Expr) -> IResult<Value> {
        // Evaluate operands left-to-right order
        let left = self.visit_expr(left)?;
        let right = self.visit_expr(right)?;

        let operator = token.token_type();

        match operator {
            TokenType::Minus => {
                let new_value = left - right;
                new_value.map_err(|err| IError::binary_op_error(err, &token))
            }
            TokenType::Slash => {
                let new_value = left / right;
                new_value.map_err(|err| IError::binary_op_error(err, &token))
            }
            TokenType::Star => {
                let new_value = left * right;
                new_value.map_err(|err| IError::binary_op_error(err, &token))
            }
            TokenType::Plus => {
                let new_value = left + right;
                new_value.map_err(|err| IError::binary_op_error(err, &token))
            }
            TokenType::Greater => Ok(Value::Bool(left > right)),
            TokenType::GreaterEqual => Ok(Value::Bool(left >= right)),
            TokenType::Less => Ok(Value::Bool(left < right)),
            TokenType::LessEqual => Ok(Value::Bool(left <= right)),
            TokenType::BangEqual => Ok(Value::Bool(left != right)),
            TokenType::EqualEqual => Ok(Value::Bool(left == right)),
            _ => Err(IError::UnexpectedError {
                token: token.clone(),
            }),
        }
    }

    fn interpret_ternary_condition(
        &mut self,
        condition: &Expr,
        inner_true: &Expr,
        inner_false: &Expr,
    ) -> IResult<Value> {
        let c = self.visit_expr(condition)?;

        return if c.is_true() {
            self.visit_expr(inner_true)
        } else {
            self.visit_expr(inner_false)
        };
    }

    fn execute_block(&mut self, statements: &[Stmt], environment: &Environment) -> IResult<()> {
        let previous = self.environment.clone();

        self.environment = environment.clone();

        for stmt in statements {
            // TODO: Find better pattern somewhat similar to try/finally
            if let Err(err) = self.visit_stmt(&stmt) {
                self.environment = previous;
                return Err(err);
            }
        }

        self.environment = previous;

        Ok(())
    }
}

impl Visitor<Value> for Interpreter {
    type ExprOutput = IResult<Value>;
    type StmtOutput = IResult<()>;
    fn visit_expr(&mut self, expr: &Expr) -> Self::ExprOutput {
        match expr {
            Expr::Binary(left, token, right) => self.interpret_binary(token, left, right),
            Expr::Grouping(expr) => self.interpret_grouping(expr.as_ref()),
            Expr::Literal(literal) => self.interpret_literal(literal),
            Expr::Unary(token, expr) => self.interpret_unary(token, expr.as_ref()),
            Expr::Condition(condition, inner_true, inner_false) => {
                self.interpret_ternary_condition(condition, inner_true, inner_false)
            }
            Expr::Variable(token) => self
                .environment
                .get(token)
                .map(|value| value.clone())
                .map_err(|err| IError::environment_error(err, token)),
            Expr::Assign(name, expr) => {
                let value = self.visit_expr(expr.as_ref())?;
                self.environment
                    .assign(name, &value)
                    .map_err(|err| IError::environment_error(err, name))?;

                return Ok(value);
            }
        }
    }

    fn visit_stmt(&mut self, stmt: &Stmt) -> Self::StmtOutput {
        match stmt {
            expr::Stmt::Expression(expr) => {
                self.visit_expr(expr)?;
            }
            expr::Stmt::Print(expr) => {
                let value = self.visit_expr(expr)?;
                println!("{value}");
            }
            expr::Stmt::Var(name, initializer) => {
                let mut value = Value::Nil;
                if let Some(expr) = initializer {
                    value = self.visit_expr(&expr)?;
                }

                self.environment.define(name.lexeme(), value);
            }
            Stmt::Block(stmts) => {
                self.execute_block(stmts, &Environment::new(Some(&self.environment)))?;
            }
        };

        Ok(())
    }
}
