use std::fmt::Write;

use crate::token::{self, Token};

#[derive(Debug, Clone)]
pub enum Expr {
    Assign(Name, Box<Expr>),
    Binary(Box<Expr>, BinaryOperator, Box<Expr>),
    Grouping(Box<Expr>),
    Literal(token::Literal),
    Logical {
        left: Box<Expr>,
        operator: Token,
        right: Box<Expr>,
    },
    Unary(UnaryOperator, Box<Expr>),
    Variable(Name),
    // ternary condition. it was a challenge.
    Condition(Box<Expr>, Box<Expr>, Box<Expr>),
}

impl From<Box<Expr>> for Expr {
    fn from(value: Box<Expr>) -> Self {
        value.as_ref().to_owned()
    }
}

type BinaryOperator = Token;
type UnaryOperator = Token;
type Name = Token;

#[derive(Debug, Clone)]
pub enum Stmt {
    Expression(Expr),
    If {
        condition: Expr,
        then_branch: Box<Stmt>,
        else_branch: Option<Box<Stmt>>,
    },
    Print(Expr),
    Var(Name, Option<Expr>),
    While {
        condition: Expr,
        body: Box<Stmt>,
    },
    Block(Vec<Stmt>),
}

pub trait Visitor<T> {
    type ExprOutput;
    type StmtOutput;
    fn visit_expr(&mut self, expr: &Expr) -> Self::ExprOutput;
    fn visit_stmt(&mut self, stmt: &Stmt) -> Self::StmtOutput;
}

pub struct AstPrinter;

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

    pub fn print(&mut self, stmts: &Vec<Stmt>) -> String {
        let mut output = vec![];
        for stmt in stmts {
            output.push(self.visit_stmt(stmt));
        }
        return output.join("\n");
    }
}

impl Visitor<String> for AstPrinter {
    type ExprOutput = String;
    type StmtOutput = String;
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
            Expr::Condition(cond, inner_true, inner_false) => buf
                .write_str(&self.parenthesize(
                    "cond",
                    vec![cond.as_ref(), inner_true.as_ref(), inner_false.as_ref()],
                ))
                .expect("Failed to write string"),
            Expr::Variable(name) => buf
                .write_str(&format!("(var {name})",))
                .expect("Failed to write string"),
            Expr::Assign(name, expr) => buf
                .write_str(&format!(
                    "(assign {})",
                    self.parenthesize(name.lexeme(), vec![expr.as_ref()])
                ))
                .expect("Failed to write string"),
            Expr::Logical {
                left,
                operator,
                right,
            } => buf
                .write_str(&format!(
                    "({:?} (left {}) (right {}))",
                    operator.token_type(),
                    self.visit_expr(left),
                    self.visit_expr(right)
                ))
                .expect("Failed to write string"),
        };

        return buf;
    }

    fn visit_stmt(&mut self, stmt: &Stmt) -> Self::ExprOutput {
        match stmt {
            Stmt::Expression(expr) => format!("{}", self.visit_expr(expr)),
            Stmt::Print(expr) => {
                format!("(print {})", self.visit_expr(expr))
            }
            Stmt::Var(name, initializer) => {
                if let Some(i) = initializer {
                    format!("(var {name} {})", self.visit_expr(i))
                } else {
                    format!("(var {name} nil)")
                }
            }
            Stmt::Block(stmts) => {
                let statements: String = stmts
                    .iter()
                    .map(|s| self.visit_stmt(&s))
                    .collect::<Vec<String>>()
                    .join(" ");

                format!("(block ({}))", statements)
            }
            Stmt::If {
                condition,
                then_branch,
                else_branch,
            } => {
                if let Some(else_branch) = else_branch {
                    return format!(
                        "(cond {} (then {}) (else {}))",
                        self.visit_expr(condition),
                        self.visit_stmt(then_branch),
                        self.visit_stmt(else_branch)
                    );
                };

                format!(
                    "(cond {} (then {}))",
                    self.visit_expr(condition),
                    self.visit_stmt(then_branch)
                )
            }
            Stmt::While { condition, body } => {
                format!(
                    "(while (cond {}) (body {})",
                    self.visit_expr(condition),
                    self.visit_stmt(body)
                )
            }
        }
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

        let pretty = AstPrinter::new().print(&vec![Stmt::Expression(expression)]);

        assert_eq!("(* (- 123) (group 45.67))", pretty)
    }
}
