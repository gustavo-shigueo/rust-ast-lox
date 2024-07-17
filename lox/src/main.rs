use clap::Parser;
use color_eyre::{owo_colors::OwoColorize, Result};
use std::{io::Write, path::Path};

use lexer::{Error, Scanner};

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
    run(&source)
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

        run(&buffer)?;
    }
}

fn run(source: &str) -> Result<()> {
    let scanner = Scanner::new(source);
    match scanner.scan() {
        Ok(tokens) => {
            dbg!(tokens);
        }
        Err(errors) => {
            for error in errors {
                report(source, error)
            }
        }
    }
    Ok(())
}

fn report(source: &str, error: Error) {
    eprintln!("{}: {}", "Error".red().bold(), error.source);

    eprintln!("  line: {}", error.line + 1);
    eprintln!("  column: {}", error.column + 1);

    let code = source.lines().nth(error.line as usize).unwrap_or_default();
    eprintln!("{code}");
    for _ in 0..error.column {
        eprint!(" ")
    }

    eprintln!("^--- Here")
}
