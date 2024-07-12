use clap::Parser;
use std::path::Path;

#[derive(Parser)]
struct Args {
    #[arg(short, long)]
    pub source: Box<Path>,
}

fn main() {
    let args = Args::parse();

    run_file(&args.source)
}

fn run_file(path: &Path) {
    dbg!(&path);
}
