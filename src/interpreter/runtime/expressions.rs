use std::io::Write;

use super::{BREAK_OUTSIDE_LOOP, CONTINUE_OUTSIDE_LOOP, ControlFlow, Runtime};
use crate::expression::Expression;
use crate::interpreter::scope::Scope;
use crate::interpreter::value::Value;
use crate::token::Token;

impl<W: Write> Runtime<W> {
    pub(super) fn evaluate_expression(&mut self, expression: &Expression) -> Result<Value, String> {
        match expression {
            Expression::Literal { value } => eval_literal(value),
            Expression::Unary { operator, operand } => self.eval_unary(operator, operand),
            Expression::Binary {
                left,
                operator,
                right,
            } => self.eval_binary(left, operator, right),
            Expression::Grouping { expression } => self.evaluate_expression(expression),
            Expression::Variable { name } => self.get_variable(name),
            Expression::ArrayLiteral { elements } => self.eval_array_literal(elements),
            Expression::Index { collection, index } => self.eval_index(collection, index),
            Expression::Len { operand } => self.eval_len(operand),
            Expression::Assignment { name, value } => self.assign_variable(name, value),
            Expression::IndexAssignment {
                collection,
                index,
                value,
            } => self.assign_index(collection, index, value),
            Expression::Call {
                function,
                arguments,
            } => self.call_function(function, arguments),
        }
    }

    fn get_variable(&self, name: &str) -> Result<Value, String> {
        if let Some(value) = self.current_scope.get(name) {
            Ok(value)
        } else {
            Err(format!("Variable '{}' not defined", name))
        }
    }

    fn assign_variable(&mut self, name: &str, value: &Expression) -> Result<Value, String> {
        let evaluated_value = self.evaluate_expression(value)?;
        self.current_scope.set(name, evaluated_value.clone())?;
        Ok(evaluated_value)
    }

    fn eval_array_literal(&mut self, elements: &[Expression]) -> Result<Value, String> {
        let mut values = Vec::with_capacity(elements.len());
        for element in elements {
            values.push(self.evaluate_expression(element)?);
        }
        Ok(Value::array(values))
    }

    fn eval_len(&mut self, operand: &Expression) -> Result<Value, String> {
        self.evaluate_expression(operand)?.len()
    }

    fn eval_index(&mut self, collection: &Expression, index: &Expression) -> Result<Value, String> {
        let collection = self.evaluate_expression(collection)?;
        let index = self.evaluate_expression(index)?;
        collection.get_index(&index)
    }

    fn assign_index(
        &mut self,
        collection: &Expression,
        index: &Expression,
        value: &Expression,
    ) -> Result<Value, String> {
        let collection = self.evaluate_expression(collection)?;
        let index = self.evaluate_expression(index)?;
        let value = self.evaluate_expression(value)?;
        collection.set_index(&index, value.clone())?;
        Ok(value)
    }

    fn eval_unary(&mut self, operator: &Token, operand: &Expression) -> Result<Value, String> {
        match operator {
            Token::Minus => {
                let value = self.evaluate_expression(operand)?;
                value.negate()
            }
            Token::Bang => {
                let value = self.evaluate_expression(operand)?;
                Ok(Value::Boolean(!value.is_truthy()))
            }
            _ => Err(format!("Unsupported unary operator: {:?}", operator)),
        }
    }

    fn eval_binary(
        &mut self,
        left: &Expression,
        operator: &Token,
        right: &Expression,
    ) -> Result<Value, String> {
        if matches!(operator, Token::And | Token::Or) {
            let left_value = self.evaluate_expression(left)?;
            if matches!(operator, Token::And) && !left_value.is_truthy() {
                return Ok(Value::Boolean(false));
            }
            if matches!(operator, Token::Or) && left_value.is_truthy() {
                return Ok(left_value);
            }
            let right_value = self.evaluate_expression(right)?;
            // `or` yields the operand that decided the result; `and` only its truthiness
            return Ok(match operator {
                Token::Or => right_value,
                _ => Value::Boolean(right_value.is_truthy()),
            });
        }

        let left_value = self.evaluate_expression(left)?;
        let right_value = self.evaluate_expression(right)?;
        match operator {
            Token::Plus => left_value + (right_value),
            Token::Minus => left_value - (right_value),
            Token::Asterisk => left_value * (right_value),
            Token::Slash => left_value / (right_value),
            Token::Percent => left_value.modulo(right_value),
            Token::EqualEqual => {
                let res = left_value == (right_value);
                Ok(Value::Boolean(res))
            }
            Token::BangEqual => {
                let res = left_value != (right_value);
                Ok(Value::Boolean(res))
            }
            Token::Less => {
                let res = left_value.less_than(&right_value)?;
                Ok(Value::Boolean(res))
            }
            Token::LessEqual => {
                let res = left_value.less_equal_than(&right_value)?;
                Ok(Value::Boolean(res))
            }
            Token::Greater => {
                let res = left_value.greater_than(&right_value)?;
                Ok(Value::Boolean(res))
            }
            Token::GreaterEqual => {
                let res = left_value.greater_equal_than(&right_value)?;
                Ok(Value::Boolean(res))
            }
            _ => Err(format!("Unsupported binary operator: {:?}", operator)),
        }
    }

    fn call_function(
        &mut self,
        function: &Expression,
        arguments: &[Expression],
    ) -> Result<Value, String> {
        let callee = self.evaluate_expression(function)?;
        let mut argument_values = Vec::with_capacity(arguments.len());
        for argument in arguments {
            argument_values.push(self.evaluate_expression(argument)?);
        }

        let Value::Function(function) = callee else {
            return Err("Attempted to call a non-function value".to_string());
        };

        if argument_values.len() != function.parameters.len() {
            return Err(format!(
                "Expected {} arguments but got {}",
                function.parameters.len(),
                argument_values.len()
            ));
        }

        let new_scope = Scope::new(Some(self.global_scope.clone()));
        for (parameter, argument_value) in function.parameters.iter().zip(argument_values) {
            new_scope.define(parameter.clone(), argument_value);
        }

        let previous_scope = self.current_scope.clone();
        self.current_scope = new_scope;
        let result = self.run_returning_flow(&function.body);
        self.current_scope = previous_scope;

        match result? {
            ControlFlow::Return(value) => Ok(value),
            ControlFlow::None => Ok(Value::Null),
            ControlFlow::Break => Err(BREAK_OUTSIDE_LOOP.to_string()),
            ControlFlow::Continue => Err(CONTINUE_OUTSIDE_LOOP.to_string()),
        }
    }
}

fn eval_literal(value: &Token) -> Result<Value, String> {
    match value {
        Token::Integer(i) => Ok(Value::Integer(*i)),
        Token::Decimal(d) => Ok(Value::Decimal(*d)),
        Token::StringLiteral(s) => Ok(Value::string(s.as_str())),
        Token::True => Ok(Value::Boolean(true)),
        Token::False => Ok(Value::Boolean(false)),
        Token::Null => Ok(Value::Null),
        _ => Err(format!("Unsupported literal token: {:?}", value)),
    }
}
