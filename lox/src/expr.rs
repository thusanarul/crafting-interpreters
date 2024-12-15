use std::fmt::Write;

use crate::token::{self, Token};

enum Expr {
    Binary(Box<Expr>, BinaryOperator, Box<Expr>),
    Grouping(Box<Expr>),
    Literal(token::Literal),
    Unary(UnaryOperator, Box<Expr>),
}

type BinaryOperator = Token;
type UnaryOperator = Token;

trait Visitor<T> {
    fn visit_expr(&mut self, expr: &Expr) -> T;
}

struct AstPrinter;

impl AstPrinter {
    pub fn new() -> Self {
        Self
    }

    fn parenthesize(&mut self, name: &str, exprs: Vec<&Expr>) -> String {
        let mut buf = String::new();

        buf.write_str(&format!("({name}"))
            .expect("Failed to write string");

        for expr in exprs {
            buf.write_str(" ").expect("Failed to write string");
            buf.write_str(&self.visit_expr(expr))
                .expect("Failed to write string");
        }

        buf.write_str(")").expect("Failed to write string");

        return buf;
    }

    pub fn print(&mut self, expr: &Expr) -> String {
        self.visit_expr(expr)
    }
}

impl Visitor<String> for AstPrinter {
    fn visit_expr(&mut self, expr: &Expr) -> String {
        let mut buf = String::new();

        match expr {
            Expr::Literal(literal) => {
                buf.write_str(&literal.to_string())
                    .expect("Failed to write string");
            }
            Expr::Binary(lhs, op, rhs) => {
                buf.write_str(&self.parenthesize(op.lexeme(), vec![lhs.as_ref(), rhs.as_ref()]))
                    .expect("Failed to write string");
            }
            Expr::Grouping(expr) => {
                buf.write_str(&self.parenthesize("group", vec![expr.as_ref()]))
                    .expect("Failed to write string");
            }
            Expr::Unary(op, rhs) => {
                buf.write_str(&self.parenthesize(op.lexeme(), vec![rhs.as_ref()]))
                    .expect("Failed to write string");
            }
        };

        return buf;
    }
}

#[cfg(test)]
mod tests {
    use token::{Literal, TokenType};

    use super::*;

    #[test]
    fn ast_printer() {
        let expression = Expr::Binary(
            Box::new(Expr::Unary(
                Token::new(TokenType::Minus, "-".to_owned(), None, 1),
                Box::new(Expr::Literal(Literal::Number(123.0))),
            )),
            Token::new(TokenType::Star, "*".to_owned(), None, 1),
            Box::new(Expr::Grouping(Box::new(Expr::Literal(Literal::Number(
                45.67,
            ))))),
        );

        let pretty = AstPrinter::new().print(&expression);

        assert_eq!("(* (- 123) (group 45.67))", pretty)
    }
}
