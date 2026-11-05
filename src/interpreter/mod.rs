mod runtime;
mod scope;
mod value;

#[cfg(test)]
mod value_test;

use std::io::{Stdout, Write};

use crate::input::Input;

use crate::statement::Statement;

pub struct Interpreter<W: Write = Stdout> {
    runtime: runtime::Runtime<W>,
}

impl Default for Interpreter<Stdout> {
    fn default() -> Self {
        Self::new()
    }
}

impl Interpreter<Stdout> {
    pub fn new() -> Self {
        Self::with_output(std::io::stdout())
    }
}

impl<W: Write> Interpreter<W> {
    pub fn with_output(output: W) -> Self {
        Self::with_io(output, Box::new(std::io::stdin()))
    }

    pub fn with_io(output: W, input: Box<dyn Input>) -> Self {
        Interpreter {
            runtime: runtime::Runtime::new(output, input),
        }
    }

    pub fn run(&mut self, statements: &[Statement]) -> Result<(), String> {
        self.runtime
            .run_plain(statements)
            .map_err(|e| format!("Error interpreting: {}", e))
    }
}
