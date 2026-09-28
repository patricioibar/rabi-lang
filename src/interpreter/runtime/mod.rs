mod expressions;
mod statements;

#[cfg(test)]
mod runtime_test;

use std::io::Write;

use super::scope::Scope;
use super::value::Value;
use crate::statement::Statement;

pub struct Runtime<W: Write> {
    global_scope: Scope,
    current_scope: Scope,
    output: W,
}

const RETURN_OUTSIDE_FUNCTION: &str = "'return' outside of a function";
const BREAK_OUTSIDE_LOOP: &str = "'break' outside of a loop";
const CONTINUE_OUTSIDE_LOOP: &str = "'continue' outside of a loop";

pub enum ControlFlow {
    Return(Value),
    Break,
    Continue,
    None,
}

impl<W: Write> Runtime<W> {
    pub(super) fn new(output: W) -> Self {
        let global_scope = Scope::new(None);
        Runtime {
            current_scope: global_scope.clone(),
            global_scope,
            output,
        }
    }

    pub(super) fn run_plain(&mut self, statements: &[Statement]) -> Result<(), String> {
        for statement in statements {
            match self.execute_statement(statement)? {
                ControlFlow::None => {}
                ControlFlow::Return(_) => return Err(RETURN_OUTSIDE_FUNCTION.to_string()),
                ControlFlow::Break => return Err(BREAK_OUTSIDE_LOOP.to_string()),
                ControlFlow::Continue => return Err(CONTINUE_OUTSIDE_LOOP.to_string()),
            }
        }
        Ok(())
    }

    fn run_block(&mut self, statements: &[Statement]) -> Result<ControlFlow, String> {
        self.run_block_with(None, statements)
    }

    /// Runs `statements` in a fresh scope, which `binding` is defined in first.
    fn run_block_with(
        &mut self,
        binding: Option<(&str, Value)>,
        statements: &[Statement],
    ) -> Result<ControlFlow, String> {
        let enclosing_scope = self.current_scope.clone();
        self.current_scope = Scope::new(Some(enclosing_scope.clone()));
        if let Some((name, value)) = binding {
            self.current_scope.define(name.to_string(), value);
        }
        let result = self.run_returning_flow(statements);
        self.current_scope = enclosing_scope;
        result
    }

    fn run_returning_flow(&mut self, statements: &[Statement]) -> Result<ControlFlow, String> {
        for statement in statements {
            let control_flow = self.execute_statement(statement)?;
            match control_flow {
                ControlFlow::Return(_) | ControlFlow::Break | ControlFlow::Continue => {
                    return Ok(control_flow);
                }
                ControlFlow::None => {}
            }
        }
        Ok(ControlFlow::None)
    }
}
