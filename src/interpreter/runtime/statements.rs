use std::io::Write;

use super::{ControlFlow, Runtime};
use crate::expression::Expression;
use crate::interpreter::value::Value;
use crate::statement::Statement;

impl<W: Write> Runtime<W> {
    pub(super) fn execute_statement(
        &mut self,
        statement: &Statement,
    ) -> Result<ControlFlow, String> {
        match statement {
            Statement::ExpressionStatement(expression) => {
                self.evaluate_expression(expression)?;
            }
            Statement::PrintStatement(expression) => {
                let value = self.evaluate_expression(expression)?;
                writeln!(self.output, "{}", value)
                    .map_err(|e| format!("Could not write output: {}", e))?;
            }
            Statement::VariableDeclaration { name, initializer } => {
                self.declare_variable(name, initializer.as_ref())?;
            }
            Statement::FunctionDeclaration {
                name,
                parameters,
                body,
            } => self.declare_function(name, parameters, body),
            Statement::IfStatement {
                condition,
                then_branch,
                else_branch,
            } => {
                return self.execute_if_statement(condition, then_branch, else_branch.as_deref());
            }
            Statement::WhileStatement { condition, block } => {
                return self.execute_while_statement(condition, block);
            }
            Statement::ForStatement {
                variable,
                iterable,
                block,
            } => {
                return self.execute_for_statement(variable, iterable, block);
            }
            Statement::ReturnStatement { value } => {
                let value = if let Some(expr) = value {
                    self.evaluate_expression(expr)?
                } else {
                    Value::Null
                };
                return Ok(ControlFlow::Return(value));
            }
            Statement::BreakStatement => return Ok(ControlFlow::Break),
            Statement::ContinueStatement => return Ok(ControlFlow::Continue),
        }
        Ok(ControlFlow::None)
    }

    fn declare_variable(
        &mut self,
        name: &str,
        initializer: Option<&Expression>,
    ) -> Result<(), String> {
        let value = if let Some(init_expr) = initializer {
            self.evaluate_expression(init_expr)?
        } else {
            Value::Null
        };
        self.current_scope.define(name.to_string(), value);
        Ok(())
    }

    fn declare_function(&mut self, name: &str, parameters: &[String], body: &[Statement]) {
        let function = Value::function(name.to_string(), parameters.to_vec(), body.to_vec());
        self.current_scope.define(name.to_string(), function);
    }

    fn execute_if_statement(
        &mut self,
        condition: &Expression,
        then_branch: &[Statement],
        else_branch: Option<&[Statement]>,
    ) -> Result<ControlFlow, String> {
        let condition_value = self.evaluate_expression(condition)?;
        if condition_value.is_truthy() {
            self.run_block(then_branch)
        } else if let Some(else_statements) = else_branch {
            self.run_block(else_statements)
        } else {
            Ok(ControlFlow::None)
        }
    }

    fn execute_while_statement(
        &mut self,
        condition: &Expression,
        block: &[Statement],
    ) -> Result<ControlFlow, String> {
        loop {
            let condition_value = self.evaluate_expression(condition)?;
            if !condition_value.is_truthy() {
                break;
            }

            match self.run_block(block)? {
                ControlFlow::Break => break,
                ControlFlow::Continue => continue,
                ControlFlow::Return(value) => return Ok(ControlFlow::Return(value)),
                ControlFlow::None => {}
            }
        }
        Ok(ControlFlow::None)
    }

    fn execute_for_statement(
        &mut self,
        variable: &str,
        iterable: &Expression,
        block: &[Statement],
    ) -> Result<ControlFlow, String> {
        let iterable = self.evaluate_expression(iterable)?;
        let Value::Array(elements) = &iterable else {
            return Err(format!(
                "Unsupported type for iteration: {}",
                iterable.type_name()
            ));
        };

        let mut position = 0;
        loop {
            // Read one element at a time, never holding the borrow across the
            // block, so the block may mutate the array it is walking.
            let element = elements.borrow().get(position).cloned();
            let Some(element) = element else {
                break;
            };
            position += 1;

            match self.run_block_with(Some((variable, element)), block)? {
                ControlFlow::Break => break,
                ControlFlow::Continue => continue,
                ControlFlow::Return(value) => return Ok(ControlFlow::Return(value)),
                ControlFlow::None => {}
            }
        }
        Ok(ControlFlow::None)
    }
}
