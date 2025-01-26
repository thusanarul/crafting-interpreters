use std::ops::{Add, Div, Mul, Neg, Not, Sub};

use thiserror::Error;

use crate::{
    expr::{Expr, Visitor},
    token::{Literal, TokenType},
};

// NOTE: Difference between Literal and Value
// A literal is something that appears in the user's source code, and is part of the parser's domain.
// A value is produced by computation and don't necessarily exist in the code itself. They are an interpreter concept, part of the runtime world.
#[derive(Debug)]
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

    fn nil(&self) -> Option<()> {
        if let Value::Nil = self {
            return Some(());
        }

        None
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
            return Ok(Value::String(left + &right));
        }

        Err(VError::InvalidOperation {
            operator: "Binary".to_owned(),
            operator_type: "+".to_owned(),
            value_type: "{self:?}".to_owned(),
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
            value_type: "{self:?}".to_owned(),
        })
    }
}

impl Div for Value {
    type Output = VResult;

    fn div(self, rhs: Self) -> Self::Output {
        if let (Some(left), Some(right)) = (self.number(), rhs.number()) {
            return Ok(Value::Number(left / right));
        }

        Err(VError::InvalidOperation {
            operator: "Binary".to_owned(),
            operator_type: "/".to_owned(),
            value_type: "{self:?}".to_owned(),
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
            value_type: "{self:?}".to_owned(),
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

struct Interpreter;

#[derive(Error, Debug, Clone)]
enum IError {
    #[error("Unary op error: {0}")]
    UnaryOpError(#[source] VError),
    #[error("Binary op error: {0}")]
    BinaryOpError(#[source] VError),
    #[error("Reached unexpected state when evaluating {operator:?} operator.")]
    UnexpectedError { operator: TokenType },
}

type IResult = Result<Value, IError>;

impl Interpreter {
    fn interpret_literal(&self, literal: &Literal) -> Value {
        literal.into()
    }

    fn interpret_grouping(&mut self, expr: &Expr) -> Value {
        self.visit_expr(expr)
    }

    fn interpret_unary(&mut self, operator: &TokenType, right: &Expr) -> IResult {
        let right = self.visit_expr(right);

        match operator {
            TokenType::Bang => {
                let new_value = !right;
                new_value.map_err(IError::UnaryOpError)
            }
            TokenType::Minus => {
                let new_value = -right;
                new_value.map_err(IError::BinaryOpError)
            }
            _ => Err(IError::UnexpectedError {
                operator: *operator,
            }),
        }
    }

    fn interpret_binary(&mut self, operator: &TokenType, left: &Expr, right: &Expr) -> IResult {
        // Evaluate operands left-to-right order
        let left = self.visit_expr(left);
        let right = self.visit_expr(right);

        match operator {
            TokenType::Minus => {
                let new_value = left - right;
                new_value.map_err(IError::BinaryOpError)
            }
            TokenType::Slash => {
                let new_value = left / right;
                new_value.map_err(IError::BinaryOpError)
            }
            TokenType::Star => {
                let new_value = left * right;
                new_value.map_err(IError::BinaryOpError)
            }
            TokenType::Plus => {
                let new_value = left + right;
                new_value.map_err(IError::BinaryOpError)
            }
            TokenType::Greater => Ok(Value::Bool(left > right)),
            TokenType::GreaterEqual => Ok(Value::Bool(left >= right)),
            TokenType::Less => Ok(Value::Bool(left < right)),
            TokenType::LessEqual => Ok(Value::Bool(left <= right)),
            TokenType::BangEqual => Ok(Value::Bool(left != right)),
            TokenType::EqualEqual => Ok(Value::Bool(left == right)),
            _ => Err(IError::UnexpectedError {
                operator: *operator,
            }),
        }
    }
}

impl Visitor<Value> for Interpreter {
    fn visit_expr(&mut self, expr: &Expr) -> Value {
        match expr {
            Expr::Binary(left, token, right) => {
                if let Ok(value) = self.interpret_binary(token.token_type(), left, right) {
                    return value;
                }

                // TODO(thusanarul): figure out how to report error and stop execution?
                panic!()
            }
            Expr::Grouping(expr) => self.interpret_grouping(expr.as_ref()),
            Expr::Literal(literal) => self.interpret_literal(literal),
            Expr::Unary(token, expr) => {
                if let Ok(value) = self.interpret_unary(token.token_type(), expr.as_ref()) {
                    return value;
                }
                // TODO(thusanarul): figure out how to report error and stop execution?
                panic!()
            }
            Expr::Condition(expr, expr1, expr2) => todo!(),
        }
    }
}
