mod scanner;

use std::{
    env, fs,
    io::{self, Write},
    process,
};

use scanner::{Scanner, Token};
use thiserror::Error;

#[derive(Error, Debug)]
enum Error {
    #[error("io error: {0}")]
    Io(#[from] io::Error),
}

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() > 1 {
        println!("Usage: jlox [script]");
        process::exit(64)
    } else if args.len() == 1 {
        run_file(&args[0]).unwrap()
    } else {
        run_prompt().unwrap();
    }
}

fn run_file(path: &String) -> Result<(), Error> {
    let bytes: Vec<u8> = fs::read(path)?;

    run(&bytes);
    Ok(())
}

fn run_prompt() -> Result<(), Error> {
    let _ = io::stdout().flush();
    let mut buf = String::new();

    loop {
        println!("> ");
        io::stdin().read_line(&mut buf)?;

        if buf == "" {
            break;
        }

        run(buf.as_bytes())
    }

    Ok(())
}

fn run(bytes: &[u8]) {
    let scanner = Scanner::new(bytes);

    let tokens: Vec<Token> = scanner.scan_tokens();

    for token in tokens {
        println!("{:?}", token)
    }
}
