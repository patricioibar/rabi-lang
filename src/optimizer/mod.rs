use crate::bytecode::Value;
use crate::expression::Expression;
use crate::statement::Statement;
use crate::token::Token;

#[cfg(test)]
mod optimizer_test;

pub fn optimize(mut statements: Vec<Statement>) -> Vec<Statement> {
    block(&mut statements);
    statements
}

fn block(statements: &mut [Statement]) {
    statements.iter_mut().for_each(statement);
}

#[rustfmt::skip]
fn statement(statement: &mut Statement) {
    match statement {
        Statement::ExpressionStatement(value) | Statement::PrintStatement(value) => expression(value),
        Statement::VariableDeclaration { initializer: value, .. }
        | Statement::ReturnStatement { value } => value.iter_mut().for_each(expression),
        Statement::FunctionDeclaration { body, .. } => block(body),
        Statement::IfStatement { condition, then_branch, else_branch } => {
            expression(condition);
            block(then_branch);
            if let Some(else_branch) = else_branch {
                block(else_branch);
            }
        }
        Statement::WhileStatement { condition: value, block: body }
        | Statement::ForStatement { iterable: value, block: body, .. } => {
            expression(value);
            block(body);
        }
        Statement::BreakStatement | Statement::ContinueStatement => {}
    }
}

#[rustfmt::skip]
fn expression(value: &mut Expression) {
    match value {
        Expression::Literal { .. } | Expression::Variable { .. } => {}
        Expression::Grouping { expression: inner }
        | Expression::Unary { operand: inner, .. }
        | Expression::Len { operand: inner }
        | Expression::Assignment { value: inner, .. } => expression(inner),
        Expression::Binary { left, right, .. }
        | Expression::Index { collection: left, index: right } => {
            expression(left);
            expression(right);
        }
        Expression::IndexAssignment { collection, index, value } => {
            expression(collection);
            expression(index);
            expression(value);
        }
        Expression::ArrayLiteral { elements } => elements.iter_mut().for_each(expression),
        Expression::Call { function, arguments } => {
            expression(function);
            arguments.iter_mut().for_each(expression);
        }
    }
    if let Some(folded) = constant_fold(value).and_then(literal_or_none) {
        *value = folded;
    }
}

/// Folds the expression if it is a constant expression, otherwise returns None.
fn constant_fold(value: &Expression) -> Option<Value> {
    match value {
        Expression::Grouping { expression } => constant_or_none(expression),
        Expression::Unary { operator, operand } => {
            let operand = constant_or_none(operand)?;
            match operator {
                Token::Minus => operand.negate().ok(),
                Token::Bang => Some(Value::Boolean(!operand.is_truthy())),
                _ => None,
            }
        }
        Expression::Binary {
            left,
            operator: Token::And,
            right,
        } => {
            if !constant_or_none(left)?.is_truthy() {
                return Some(Value::Boolean(false));
            }
            Some(Value::Boolean(constant_or_none(right)?.is_truthy()))
        }
        Expression::Binary {
            left,
            operator: Token::Or,
            right,
        } => {
            let left = constant_or_none(left)?;
            if left.is_truthy() {
                Some(left)
            } else {
                constant_or_none(right)
            }
        }
        Expression::Binary {
            left,
            operator,
            right,
        } => binary(operator, constant_or_none(left)?, constant_or_none(right)?),
        _ => None,
    }
}

fn binary(operator: &Token, a: Value, b: Value) -> Option<Value> {
    let is_string = |value: &Value| matches!(value, Value::String(_));
    let result = match operator {
        Token::Plus => a + b,
        Token::Minus => a - b,
        Token::Asterisk if is_string(&a) || is_string(&b) => return None,
        Token::Asterisk => a * b,
        Token::Slash => a / b,
        Token::Percent => a.modulo(b),
        Token::EqualEqual => Ok(Value::Boolean(a == b)),
        Token::BangEqual => Ok(Value::Boolean(a != b)),
        Token::Greater => a.greater_than(&b).map(Value::Boolean),
        Token::GreaterEqual => a.greater_equal_than(&b).map(Value::Boolean),
        Token::Less => a.less_than(&b).map(Value::Boolean),
        Token::LessEqual => a.less_equal_than(&b).map(Value::Boolean),
        _ => return None,
    };
    result.ok()
}

fn constant_or_none(value: &Expression) -> Option<Value> {
    let Expression::Literal { value } = value else {
        return None;
    };
    match value {
        Token::Integer(i) => Some(Value::Integer(*i)),
        Token::Decimal(d) => Some(Value::Decimal(*d)),
        Token::StringLiteral(s) => Some(Value::string(s.as_str())),
        Token::True => Some(Value::Boolean(true)),
        Token::False => Some(Value::Boolean(false)),
        Token::Null => Some(Value::Null),
        _ => None,
    }
}

fn literal_or_none(value: Value) -> Option<Expression> {
    let token = match value {
        Value::Integer(i) => Token::Integer(i),
        Value::Decimal(d) => Token::Decimal(d),
        Value::String(s) => Token::StringLiteral(s.to_string()),
        Value::Boolean(true) => Token::True,
        Value::Boolean(false) => Token::False,
        Value::Null => Token::Null,
        _ => return None,
    };
    Some(Expression::Literal { value: token })
}
