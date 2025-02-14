use std::collections::HashMap;

use thiserror::Error;

use crate::{interpreter::Value, token::Token};

#[derive(Error, Debug, Clone)]
pub(crate) enum Error {
    #[error("Undefined variable {0}")]
    UndefinedVariable(String),
}

type EResult<T> = Result<T, Error>;

#[derive(Debug, Clone)]
pub struct Environment {
    enclosing: Option<Box<Environment>>,
    values: HashMap<String, Value>,
}

impl Environment {
    pub fn new(enclosing: Option<&Environment>) -> Self {
        Self {
            enclosing: enclosing.map(|e| Box::new(e.clone())),
            values: HashMap::new(),
        }
    }

    pub fn assign(&mut self, token: &Token, value: &Value) -> EResult<()> {
        let name = token.lexeme();
        if self.values.contains_key(name) {
            self.values.insert(name.to_owned(), value.clone());
            return Ok(());
        }

        if let Some(enclosing) = &mut self.enclosing {
            return enclosing.assign(token, value);
        }

        Err(Error::UndefinedVariable(name.to_owned()))
    }

    pub fn define(&mut self, name: &str, value: Value) {
        self.values.insert(name.to_owned(), value);
    }

    pub fn get(&self, token: &Token) -> EResult<&Value> {
        let name = token.lexeme();
        if self.values.contains_key(name) {
            return Ok(self.values.get(name).unwrap());
        }

        if let Some(enclosing) = &self.enclosing {
            return enclosing.get(token);
        }

        Err(Error::UndefinedVariable(name.to_owned()))
    }
}
