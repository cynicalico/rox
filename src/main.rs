#![allow(dead_code)]

use crate::rvm::interpret;
use clap::Parser;
use std::io::Write;
use std::path::PathBuf;
use std::process::exit;
use std::{fs, io};

mod chunk;
mod common;
mod compiler;
mod rvm;
mod scanner;
mod value;

/// rox interpreter
#[derive(Parser, Debug)]
#[command(about, long_about = None)]
struct Args {
    /// Run file
    #[arg(required = false)]
    script: Option<PathBuf>,
}

fn main() {
    let args = Args::parse();

    if let Some(script) = args.script {
        run_file(script);
    } else {
        repl();
    }
}

fn repl() {
    loop {
        print!("> ");
        io::stdout().flush().unwrap();

        let mut line = String::new();
        match io::stdin().read_line(&mut line) {
            Ok(bytes_read) => {
                if bytes_read == 0 {
                    break;
                }

                let _ = interpret(&line);
                line.clear();
            }
            Err(err) => {
                eprintln!("Failed to read line: {err}");
            }
        }
    }
}

fn run_file(path: PathBuf) {
    match fs::read_to_string(path) {
        Ok(source) => {
            let res = interpret(&source);

            if res.is_err() {
                exit(1);
            }
        }
        Err(err) => {
            eprintln!("Failed to read script: {err}");
            exit(1);
        }
    }
}
