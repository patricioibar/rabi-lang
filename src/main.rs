use std::io::{BufRead, BufReader};

use clap::Parser;

use crate::mode::Mode;

mod mode;

#[derive(Debug, Parser)]
#[command(name = "pibar-language")]
struct Args {
    /// Executes lexical analysis.
    #[arg(long, conflicts_with = "parsing")]
    scanning: bool,

    /// Executes syntactic analysis.
    #[arg(long, conflicts_with = "scanning")]
    parsing: bool,

    /// Input file.
    filename: Option<String>,
}

fn main() -> Result<(), i32> {
    let args = Args::parse();
    let mode = Mode::from(&args)?;
    if let Some(_filename) = &args.filename {
        // TO DO: file input
        eprintln!("File input not yet supported");
        return Err(1);
    }

    let input = BufReader::new(std::io::stdin());
    for line in input.lines() {
        let line = line
            .inspect_err(|e| eprintln!("Error reading input: {}", e))
            .map_err(|_| 2)?;
        run(line, &mode)?;
    }

    Ok(())
}

fn run(_line: String, _mode: &Mode) -> Result<(), i32> {
    Ok(())
}
