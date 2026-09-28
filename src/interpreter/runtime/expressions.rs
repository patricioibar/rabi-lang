use std::cell::RefCell;
use std::io::Write;
use std::rc::Rc;

use super::{BREAK_OUTSIDE_LOOP, CONTINUE_OUTSIDE_LOOP, ControlFlow, Runtime};
use crate::expression::Expression;
use crate::interpreter::scope::Scope;
use crate::interpreter::value::{Value, overflow};
use crate::token::Token;

/// The elements an [`Expression::Index`] resolves to, aliased with every other
/// binding holding the same array.
type SharedElements = Rc<RefCell<Vec<Value>>>;

impl<W: Write> Runtime<W> {
    pub(super) fn evaluate_expression(&mut self, expression: Expression) -> Result<Value, String> {
        match expression {
            Expression::Literal { value } => eval_literal(value),
            Expression::Unary { operator, operand } => self.eval_unary(operator, *operand),
            Expression::Binary {
                left,
                operator,
                right,
            } => self.eval_binary(*left, operator, *right),
            Expression::Grouping { expression } => self.evaluate_expression(*expression),
            Expression::Variable { name } => self.get_variable(&name),
            Expression::ArrayLiteral { elements } => self.eval_array_literal(elements),
            Expression::Index { collection, index } => self.eval_index(*collection, *index),
            Expression::Len { operand } => self.eval_len(*operand),
            Expression::Assignment { name, value } => self.assign_variable(name, *value),
            Expression::IndexAssignment {
                collection,
                index,
                value,
            } => self.assign_index(*collection, *index, *value),
            Expression::Call {
                function,
                arguments,
            } => self.call_function(*function, arguments),
        }
    }

    fn get_variable(&self, name: &str) -> Result<Value, String> {
        if let Some(value) = self.current_scope.get(name) {
            Ok(value)
        } else {
            Err(format!("Variable '{}' not defined", name))
        }
    }

    fn assign_variable(&mut self, name: String, value: Expression) -> Result<Value, String> {
        let evaluated_value = self.evaluate_expression(value)?;
        self.current_scope
            .set(name.clone(), evaluated_value.clone())?;
        Ok(evaluated_value)
    }

    fn eval_array_literal(&mut self, elements: Vec<Expression>) -> Result<Value, String> {
        let mut values = Vec::with_capacity(elements.len());
        for element in elements {
            values.push(self.evaluate_expression(element)?);
        }
        Ok(Value::array(values))
    }

    fn eval_len(&mut self, operand: Expression) -> Result<Value, String> {
        match self.evaluate_expression(operand)? {
            Value::Array(elements) => Ok(Value::Integer(elements.borrow().len() as i64)),
            other => Err(format!("Unsupported type for len: {}", other.type_name())),
        }
    }

    fn eval_index(&mut self, collection: Expression, index: Expression) -> Result<Value, String> {
        let (array, position) = self.eval_index_target(collection, index)?;
        Ok(array.borrow()[position].clone())
    }

    fn assign_index(
        &mut self,
        collection: Expression,
        index: Expression,
        value: Expression,
    ) -> Result<Value, String> {
        let (array, position) = self.eval_index_target(collection, index)?;
        let value = self.evaluate_expression(value)?;
        array.borrow_mut()[position] = value.clone();
        Ok(value)
    }

    /// resolve `collection[index]`
    fn eval_index_target(
        &mut self,
        collection: Expression,
        index: Expression,
    ) -> Result<(SharedElements, usize), String> {
        let collection = self.evaluate_expression(collection)?;
        let Value::Array(array) = &collection else {
            return Err(format!(
                "Unsupported type for indexing: {}",
                collection.type_name()
            ));
        };

        let index = self.evaluate_expression(index)?;
        let Value::Integer(position) = index else {
            return Err(format!(
                "Array index must be an Integer, found {}",
                index.type_name()
            ));
        };

        let length = array.borrow().len();
        if position < 0 || position as usize >= length {
            return Err(format!(
                "Array index out of bounds: {} (length {})",
                position, length
            ));
        }

        Ok((array.clone(), position as usize))
    }

    fn eval_unary(&mut self, operator: Token, operand: Expression) -> Result<Value, String> {
        match operator {
            Token::Minus => {
                let value = self.evaluate_expression(operand)?;
                match value {
                    Value::Integer(i) => i
                        .checked_neg()
                        .map(Value::Integer)
                        .ok_or_else(|| overflow("negation")),
                    Value::Decimal(d) => Ok(Value::Decimal(-d)),
                    _ => Err(format!(
                        "Unsupported operand type for unary minus: {:?}",
                        value
                    )),
                }
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
        left: Expression,
        operator: Token,
        right: Expression,
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
            return Ok(Value::Boolean(right_value.is_truthy()));
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
        function: Expression,
        arguments: Vec<Expression>,
    ) -> Result<Value, String> {
        let Value::Function {
            parameters,
            body,
            name: _,
        } = self.evaluate_expression(function)?
        else {
            return Err("Attempted to call a non-function value".to_string());
        };

        if arguments.len() != parameters.len() {
            return Err(format!(
                "Expected {} arguments but got {}",
                parameters.len(),
                arguments.len()
            ));
        }

        let new_scope = Scope::new(Some(self.global_scope.clone()));
        for (param, arg_expr) in parameters.into_iter().zip(arguments) {
            let arg_value = self.evaluate_expression(arg_expr)?;
            new_scope.define(param, arg_value);
        }

        let previous_scope = self.current_scope.clone();
        self.current_scope = new_scope;
        let result = self.run_returning_flow(body);
        self.current_scope = previous_scope;

        match result? {
            ControlFlow::Return(value) => Ok(value),
            ControlFlow::None => Ok(Value::Null),
            ControlFlow::Break => Err(BREAK_OUTSIDE_LOOP.to_string()),
            ControlFlow::Continue => Err(CONTINUE_OUTSIDE_LOOP.to_string()),
        }
    }
}

fn eval_literal(value: Token) -> Result<Value, String> {
    match value {
        Token::Integer(i) => Ok(Value::Integer(i)),
        Token::Decimal(d) => Ok(Value::Decimal(d)),
        Token::StringLiteral(s) => Ok(Value::String(s)),
        Token::True => Ok(Value::Boolean(true)),
        Token::False => Ok(Value::Boolean(false)),
        Token::Null => Ok(Value::Null),
        _ => Err(format!("Unsupported literal token: {:?}", value)),
    }
}
