use crate::token::{self, Token};

enum Expr {
    Binary(Box<Expr>, BinaryOperator, Box<Expr>),
    Grouping(Box<Expr>),
    Literal(token::Literal),
    Unary(UnaryOperator, Box<Expr>),
}

type BinaryOperator = Token;
type UnaryOperator = Token;
