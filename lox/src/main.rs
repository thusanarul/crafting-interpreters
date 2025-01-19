mod expr;
mod interpreter;
mod parser;
mod scanner;
mod token;

use std::{
    env,
    fmt::Display,
    fs,
    io::{self, Write},
    process,
};

use expr::AstPrinter;
use parser::Parser;
use scanner::Scanner;
use thiserror::Error;
use token::Token;

#[derive(Error, Debug)]
enum Error {
    #[error("io error: {0}")]
    Io(#[from] io::Error),
    #[error("scanner errors: {0}")]
    ScannerError(scanner::Errors),
}

impl Display for scanner::Errors {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // TODO: Do this better
        write!(f, "{:?}", self)
    }
}

impl From<scanner::Errors> for Error {
    fn from(value: scanner::Errors) -> Self {
        Self::ScannerError(value)
    }
}

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() > 2 {
        println!("Usage: jlox [script]");
        process::exit(64)
    } else if args.len() == 2 {
        run_file(&args[1]).unwrap()
    } else {
        run_prompt().unwrap();
    }
}

fn run_file(path: &String) -> Result<(), Error> {
    let bytes: Vec<u8> = fs::read(path)?;

    run(&bytes)?;
    Ok(())
}

fn run_prompt() -> Result<(), Error> {
    let _ = io::stdout().flush();
    let mut buf = String::new();

    loop {
        print!("> ");
        // Flush stdout because we call print! and not println!. The buffer is only flushed when we print a newline.
        io::stdout().flush()?;
        buf.clear();
        io::stdin().read_line(&mut buf)?;

        if buf == "" {
            break;
        }

        run(buf.as_bytes())?
    }

    Ok(())
}

fn run(bytes: &[u8]) -> Result<(), Error> {
    let mut scanner = Scanner::new(bytes);

    let tokens: Vec<Token> = scanner.scan_tokens()?;
    let mut parser = Parser::new(tokens);

    let expr = parser.parse();

    if let Err(err) = expr.clone() {
        eprintln!("{}", err);
        return Ok(());
    }

    println!("{}", AstPrinter::new().print(&expr.unwrap()));

    Ok(())
}
