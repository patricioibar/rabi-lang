use std::io::{BufReader, Write};

use clap::{Parser, ValueEnum};

use rabi::{
    compiler,
    interpreter::Interpreter,
    optimizer, parser,
    scanner::{self, Scanner},
    statement::Statement,
    token::Token,
    vm::Vm,
};

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

    /// Execution backend.
    #[arg(long, value_enum, default_value_t = Backend::Interpreter)]
    backend: Backend,

    /// Input file.
    filename: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum)]
enum Backend {
    /// Tree-walking interpreter.
    Interpreter,
    /// Bytecode compiler and virtual machine.
    Vm,
}

enum Runner {
    Interpreter(Interpreter),
    Vm(Vm),
}

impl Runner {
    fn new(backend: Backend) -> Self {
        match backend {
            Backend::Interpreter => Runner::Interpreter(Interpreter::new()),
            Backend::Vm => Runner::Vm(Vm::new()),
        }
    }

    fn run(&mut self, statements: Vec<Statement>) -> Result<(), String> {
        match self {
            Runner::Interpreter(interpreter) => interpreter.run(&statements),
            Runner::Vm(vm) => {
                let script = compiler::compile(optimizer::optimize(statements))
                    .map_err(|e| format!("Error compiling: {}", e))?;
                vm.run(script)
            }
        }
    }
}

fn main() -> Result<(), i32> {
    let args = Args::parse();
    let mode = Mode::from(&args)?;

    if let Some(filename) = &args.filename {
        file_mode(filename, mode, args.backend)?;
    } else {
        inline_mode(mode, args.backend)?;
    }

    Ok(())
}

fn inline_mode(mode: Mode, backend: Backend) -> Result<(), i32> {
    let mut runner = Runner::new(backend);
    let mut scanner = Scanner::new();
    loop {
        let prompt = if scanner.is_block_open() {
            "... "
        } else {
            "> "
        };
        print!("{}", prompt);
        std::io::stdout().flush().map_err(|_| 1)?;
        // stdin is read one line at a time and not buffered, so the
        // `input` operator can read the lines that follow
        let mut line = String::new();
        let read = std::io::stdin().read_line(&mut line).map_err(|e| {
            eprintln!("Error reading input: {}", e);
            2
        })?;
        if read == 0 {
            break;
        }
        let line = line.trim_end_matches(['\n', '\r']);

        // a blank line closes the block being typed
        let blank = line.trim().is_empty();

        if let Err(e) = scanner.scan_one_line(line) {
            eprintln!("Error scanning line: {}", e);
            scanner.finish(); // drop what was typed so far
            continue;
        }
        if scanner.is_block_open() && !blank {
            continue;
        }

        if let Err(e) = run_tokens(scanner.finish(), &mode, &mut runner) {
            eprintln!("Error processing line: {}", e);
        }
    }
    Ok(())
}

fn file_mode(filename: &str, mode: Mode, backend: Backend) -> Result<(), i32> {
    let file = std::fs::File::open(filename).map_err(|e| {
        eprintln!("Error opening file {}: {}", filename, e);
        1
    })?;
    let tokens = scanner::scan(BufReader::new(file)).map_err(|e| {
        eprintln!("Error scanning file {}: {}", filename, e);
        2
    })?;
    run_tokens(tokens, &mode, &mut Runner::new(backend)).map_err(|e| {
        eprintln!("Error processing file {}: {}", filename, e);
        3
    })
}

fn run_tokens(tokens: Vec<Token>, mode: &Mode, runner: &mut Runner) -> Result<(), String> {
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

    runner.run(statements)
}
