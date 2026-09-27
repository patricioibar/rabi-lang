use std::io::{BufRead, BufReader, Write};

use clap::Parser;

use rabi::{interpreter::Interpreter, parser, scanner, token::Token};

use crate::mode::Mode;

mod mode;

#[derive(Debug, Parser)]
#[command(name = "rabi")]
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

    if let Some(filename) = &args.filename {
        file_mode(filename, mode)?;
    } else {
        inline_mode(mode)?;
    }

    Ok(())
}

fn inline_mode(mode: Mode) -> Result<(), i32> {
    let input = BufReader::new(std::io::stdin());
    let mut lines = input.lines();
    let mut interpreter = Interpreter::new();
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
        let tokens = scanner::scan(line.as_bytes());
        let Ok(tokens) = tokens else {
            eprintln!("Error scanning line: {}", tokens.unwrap_err());
            continue;
        };
        if let Err(e) = run_tokens(tokens, &mode, &mut interpreter) {
            eprintln!("Error processing line: {}", e);
        }
    }
    Ok(())
}

fn file_mode(filename: &str, mode: Mode) -> Result<(), i32> {
    let file = std::fs::File::open(filename).map_err(|e| {
        eprintln!("Error opening file {}: {}", filename, e);
        1
    })?;
    let tokens = scanner::scan(BufReader::new(file)).map_err(|e| {
        eprintln!("Error scanning file {}: {}", filename, e);
        2
    })?;
    run_tokens(tokens, &mode, &mut Interpreter::new()).map_err(|e| {
        eprintln!("Error processing file {}: {}", filename, e);
        3
    })
}

fn run_tokens(
    tokens: Vec<Token>,
    mode: &Mode,
    interpreter: &mut Interpreter,
) -> Result<(), String> {
    if mode == &Mode::Scanning {
        for token in tokens {
            println!("{:?}", token);
        }
        return Ok(());
    }

    let statements = parser::parse(tokens)?;
    if mode == &Mode::Parsing {
        for statement in statements {
            println!("{:?}", statement);
        }
        return Ok(());
    }

    interpreter.run(statements)?;

    Ok(())
}
