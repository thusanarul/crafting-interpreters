mod expr;
mod interpreter;
mod parser;
mod scanner;
mod token;

use std::{
    env, fs,
    io::{self, Write},
    process,
};

use expr::AstPrinter;
use interpreter::Interpreter;
use parser::Parser;
use scanner::Scanner;
use thiserror::Error;
use token::Token;

#[derive(Error, Debug)]
enum Error {
    #[error("io error: {0}")]
    Io(#[from] io::Error),
    #[error("scanner errors: {0:?}")]
    ScannerError(#[from] scanner::Errors),
    #[error("runtime error: {0:?}")]
    RuntimeError(#[from] interpreter::IError),
}

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() > 2 {
        println!("Usage: jlox [script]");
        process::exit(64)
    } else if args.len() == 2 {
        if let Err(err) = run_file(&args[1]) {
            match err {
                Error::RuntimeError(_) => process::exit(70),
                _ => process::exit(65),
            }
        }
    } else {
        run_prompt()
    }
}

fn run_file(path: &String) -> Result<(), Error> {
    let bytes: Vec<u8> = fs::read(path)?;

    run(&bytes)?;
    Ok(())
}

fn run_prompt() {
    let _ = io::stdout().flush();

    let _ = inner_prompt_runner();
}

fn inner_prompt_runner() -> Result<(), Error> {
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

        if let Err(err) = run(buf.as_bytes()) {
            eprintln!("{err}")
        }
    }

    Ok(())
}

fn run(bytes: &[u8]) -> Result<(), Error> {
    let mut scanner = Scanner::new(bytes);

    let tokens: Vec<Token> = scanner.scan_tokens()?;
    let mut parser = Parser::new(tokens);

    let stmts = parser.parse();

    if let Err(err) = stmts.clone() {
        eprintln!("{}", err);
        return Ok(());
    }

    println!("{}", AstPrinter::new().print(&stmts.clone().unwrap()));

    let interpreter = Interpreter::new();

    interpreter.interpret(&stmts.unwrap());

    Ok(())
}
