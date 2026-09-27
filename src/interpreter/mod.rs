mod runtime;
mod scope;
mod value;

#[cfg(test)]
mod runtime_test;

#[cfg(test)]
mod value_test;

use crate::statement::Statement;

/// A reusable interpreter. It owns the global scope, so whatever a batch of
/// statements defines is still there for the next batch: this is what lets
/// the REPL remember variables and functions from one line to the next.
pub struct Interpreter {
    runtime: runtime::Runtime,
}

impl Default for Interpreter {
    fn default() -> Self {
        Self::new()
    }
}

impl Interpreter {
    pub fn new() -> Self {
        Interpreter {
            runtime: runtime::Runtime::new(),
        }
    }

    pub fn run(&mut self, statements: Vec<Statement>) -> Result<(), String> {
        self.runtime
            .run_plain(statements)
            .map_err(|e| format!("Error interpreting: {}", e))
    }
}
