use std::io::{BufRead, BufReader, Write};

use clap::Parser;

use crate::mode::Mode;

mod expression;
mod mode;
mod parser;
mod scanner;
mod token;

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
    let mut lines = input.lines();
    loop {
        print!("> ");
        std::io::stdout().flush().map_err(|_| 1)?;
        let Some(line) = lines.next().transpose().map_err(|e| {
            eprintln!("Error reading input: {}", e);
            2
        })?
        else {
            break;
        };
        run(line, &mode).map_err(|e| {
            eprintln!("Error processing line: {}", e);
            3
        })?;
    }

    Ok(())
}

fn run(line: String, mode: &Mode) -> Result<(), String> {
    let tokens = scanner::scan_line(line)?;
    if mode == &Mode::Scanning {
        for token in tokens {
            println!("{:?}", token);
        }
        return Ok(());
    }

    let expressions = parser::parse(tokens)?;
    if mode == &Mode::Parsing {
        for expression in expressions {
            println!("{:?}", expression);
        }
        return Ok(());
    }
    Ok(())
}
