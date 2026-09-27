use std::io::Write;

use super::value::{Value, overflow};
use crate::expression::Expression;
use crate::statement::Statement;
use crate::token::Token;

use super::scope::Scope;

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

    pub(super) fn run_plain(&mut self, statements: Vec<Statement>) -> Result<(), String> {
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

    fn run_block(&mut self, statements: Vec<Statement>) -> Result<ControlFlow, String> {
        let enclosing_scope = self.current_scope.clone();
        self.current_scope = Scope::new(Some(enclosing_scope.clone()));
        let result = self.run_returning_flow(statements);
        self.current_scope = enclosing_scope;
        result
    }

    fn run_returning_flow(&mut self, statements: Vec<Statement>) -> Result<ControlFlow, String> {
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

    fn execute_statement(&mut self, statement: Statement) -> Result<ControlFlow, String> {
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
                self.declare_variable(name, initializer)?;
            }
            Statement::FunctionDeclaration {
                name,
                parameters,
                body,
            } => self.declare_function(name, parameters, body)?,
            Statement::IfStatement {
                condition,
                then_branch,
                else_branch,
            } => return self.execute_if_statement(condition, then_branch, else_branch),
            Statement::WhileStatement { condition, block } => {
                return self.execute_while_statement(condition, block);
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

    fn evaluate_expression(&mut self, expression: Expression) -> Result<Value, String> {
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
            Expression::Assignment { name, value } => self.assign_variable(name, *value),
            Expression::Call {
                function,
                arguments,
            } => self.call_function(*function, arguments),
        }
    }

    fn declare_variable(
        &mut self,
        name: String,
        initializer: Option<Expression>,
    ) -> Result<(), String> {
        let value = if let Some(init_expr) = initializer {
            self.evaluate_expression(init_expr)?
        } else {
            Value::Null
        };
        self.current_scope.define(name, value);
        Ok(())
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
        match operator {
            Token::Plus => self.evaluate_expression(left)? + (self.evaluate_expression(right)?),
            Token::Minus => self.evaluate_expression(left)? - (self.evaluate_expression(right)?),
            Token::Asterisk => {
                self.evaluate_expression(left)? * (self.evaluate_expression(right)?)
            }
            Token::Slash => self.evaluate_expression(left)? / (self.evaluate_expression(right)?),
            Token::Percent => self
                .evaluate_expression(left)?
                .modulo(self.evaluate_expression(right)?),
            Token::EqualEqual => {
                let res = self.evaluate_expression(left)? == (self.evaluate_expression(right)?);
                Ok(Value::Boolean(res))
            }
            Token::BangEqual => {
                let res = self.evaluate_expression(left)? != (self.evaluate_expression(right)?);
                Ok(Value::Boolean(res))
            }
            Token::Less => {
                let res = self
                    .evaluate_expression(left)?
                    .less_than(&self.evaluate_expression(right)?)?;
                Ok(Value::Boolean(res))
            }
            Token::LessEqual => {
                let res = self
                    .evaluate_expression(left)?
                    .less_equal_than(&self.evaluate_expression(right)?)?;
                Ok(Value::Boolean(res))
            }
            Token::Greater => {
                let res = self
                    .evaluate_expression(left)?
                    .greater_than(&self.evaluate_expression(right)?)?;
                Ok(Value::Boolean(res))
            }
            Token::GreaterEqual => {
                let res = self
                    .evaluate_expression(left)?
                    .greater_equal_than(&self.evaluate_expression(right)?)?;
                Ok(Value::Boolean(res))
            }
            Token::And => {
                let left_value = self.evaluate_expression(left)?;
                if !left_value.is_truthy() {
                    return Ok(Value::Boolean(false));
                }
                let right_value = self.evaluate_expression(right)?;
                Ok(Value::Boolean(right_value.is_truthy()))
            }
            Token::Or => {
                let left_value = self.evaluate_expression(left)?;
                if left_value.is_truthy() {
                    return Ok(left_value);
                }
                let right_value = self.evaluate_expression(right)?;
                if right_value.is_truthy() {
                    return Ok(right_value);
                }
                Ok(Value::Boolean(false))
            }
            _ => Err(format!("Unsupported binary operator: {:?}", operator)),
        }
    }

    fn declare_function(
        &mut self,
        name: String,
        parameters: Vec<String>,
        body: Vec<Statement>,
    ) -> Result<(), String> {
        let function_value = Value::Function {
            name: name.clone(),
            parameters,
            body,
        };
        self.current_scope.define(name, function_value);
        Ok(())
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

    fn execute_if_statement(
        &mut self,
        condition: Expression,
        then_branch: Vec<Statement>,
        else_branch: Option<Vec<Statement>>,
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
        condition: Expression,
        block: Vec<Statement>,
    ) -> Result<ControlFlow, String> {
        loop {
            let condition_value = self.evaluate_expression(condition.clone())?;
            if !condition_value.is_truthy() {
                break;
            }

            match self.run_block(block.clone())? {
                ControlFlow::Break => break,
                ControlFlow::Continue => continue,
                ControlFlow::Return(value) => return Ok(ControlFlow::Return(value)),
                ControlFlow::None => {}
            }
        }
        Ok(ControlFlow::None)
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
