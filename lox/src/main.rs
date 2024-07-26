use clap::Parser as Clap;
use color_eyre::Result;
use std::{io::Write, path::Path};

use interpreter::Interpreter;
use lexer::Lexer;
use parser::Parser;

#[derive(Clap)]
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

    let mut interpreter = Interpreter::new();

    run(&mut interpreter, &source)?;
    Ok(())
}

fn run_prompt() -> Result<()> {
    let mut interpreter = Interpreter::new();

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

        _ = run(&mut interpreter, &buffer);
    }
}

fn run(interpreter: &mut Interpreter, source: &str) -> Result<()> {
    let lexer = Lexer::new(source);
    let tokens = lexer.scan();

    let mut parser = Parser::new(source, &tokens);
    let ast = parser.parse();

    interpreter.interpret(source, &ast);

    Ok(())
}
