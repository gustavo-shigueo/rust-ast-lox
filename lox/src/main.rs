use clap::Parser;
use color_eyre::Result;
use parser::Parser as LoxParser;
use std::{io::Write, path::Path};

use lexer::Lexer;
use lox_core::report;

#[derive(Parser)]
struct Args {
    #[arg(short, long)]
    pub source: Option<Box<Path>>,
}

fn main() -> Result<()> {
    let args = Args::parse();

    match args.source {
        Some(ref path) => run_file(path)?,
        None => run_prompt()?,
    };

    Ok(())
}

fn run_file(path: &Path) -> Result<()> {
    let source = std::fs::read_to_string(path)?;
    run(&source);
    Ok(())
}

fn run_prompt() -> Result<()> {
    let mut stdout = std::io::stdout();
    let stdin = std::io::stdin();
    let mut buffer = String::new();

    loop {
        _ = stdout.write_all(b"> ");
        _ = stdout.flush();
        buffer.clear();
        stdin.read_line(&mut buffer)?;

        if buffer.trim().is_empty() {
            return Ok(());
        }

        run(&buffer);
    }
}

fn run(source: &str) {
    let mut lexer = Lexer::new(source);
    let output: Vec<_> = lexer.scan();

    if lexer.had_errors() {
        for item in output.iter() {
            match item {
                Err(ref error) => report(source, error),
                Ok(_) => continue,
            }
        }

        return;
    }

    let tokens: Vec<_> = output.into_iter().filter_map(|x| x.ok()).collect();

    let mut parser = LoxParser::new(source, &tokens);
    match parser.parse() {
        Ok(ast) => println!("{}", dbg!(ast)),
        Err(ref error) => report(source, error),
    }
}
