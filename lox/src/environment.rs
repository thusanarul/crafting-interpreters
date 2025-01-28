use std::collections::HashMap;

use thiserror::Error;

use crate::{interpreter::Value, token::Token};

#[derive(Error, Debug, Clone)]
pub(crate) enum Error {
    #[error("Undefined variable {0}")]
    UndefinedVariable(String),
}

type EResult<T> = Result<T, Error>;

pub struct Environment {
    values: HashMap<String, Value>,
}

impl Environment {
    pub fn new() -> Self {
        Self {
            values: HashMap::new(),
        }
    }

    pub fn define(&mut self, name: &str, value: Value) {
        self.values.insert(name.to_owned(), value);
    }

    pub fn get(&self, token: &Token) -> EResult<&Value> {
        let name = token.lexeme();
        self.values
            .get(name)
            .ok_or(Error::UndefinedVariable(name.to_owned()))
    }
}
