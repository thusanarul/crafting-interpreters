use std::any::Any;

use crate::{
    expr::{Expr, Visitor},
    token::Literal,
};

// NOTE: Difference between Literal and Value
// A literal is something that appears in the user's source code, and is part of the parser's domain.
// A value is produced by computation and don't necessarily exist in the code itself. They are an interpreter concept, part of the runtime world.
pub enum Value {
    Number(f64),
    String(String),
    Bool(bool),
    Nil,
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

impl Interpreter {
    fn interpret_literal(&self, literal: &Literal) -> Value {
        literal.into()
    }

    fn interpret_grouping(&mut self, expr: &Expr) -> Value {
        self.visit_expr(expr)
    }
}

impl Visitor<Value> for Interpreter {
    fn visit_expr(&mut self, expr: &Expr) -> Value {
        match expr {
            Expr::Binary(expr, token, expr1) => todo!(),
            Expr::Grouping(expr) => self.interpret_grouping(expr.as_ref()),
            Expr::Literal(literal) => self.interpret_literal(literal),
            Expr::Unary(token, expr) => todo!(),
            Expr::Condition(expr, expr1, expr2) => todo!(),
        }
    }
}
