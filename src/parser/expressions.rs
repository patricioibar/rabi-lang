use super::cursor::Cursor;
use crate::{expression::Expression, token::Token};

pub(super) fn expression(cursor: &mut Cursor) -> Result<Expression, String> {
    assignment(cursor)
}

fn assignment(cursor: &mut Cursor) -> Result<Expression, String> {
    let expr = logic_or(cursor)?;

    if cursor.accept(&Token::Equal) {
        return match expr {
            Expression::Variable { name } => Ok(Expression::Assignment {
                name,
                value: Box::new(assignment(cursor)?),
            }),
            Expression::Index { collection, index } => Ok(Expression::IndexAssignment {
                collection,
                index,
                value: Box::new(assignment(cursor)?),
            }),
            _ => Err("Invalid assignment target".to_string()),
        };
    }

    Ok(expr)
}

fn logic_or(cursor: &mut Cursor) -> Result<Expression, String> {
    let mut expr = logic_and(cursor)?;

    while cursor.accept(&Token::Or) {
        let right = logic_and(cursor)?;
        expr = Expression::Binary {
            left: Box::new(expr),
            operator: Token::Or,
            right: Box::new(right),
        };
    }

    Ok(expr)
}

fn logic_and(cursor: &mut Cursor) -> Result<Expression, String> {
    let mut expr = equality(cursor)?;

    while cursor.accept(&Token::And) {
        let right = equality(cursor)?;
        expr = Expression::Binary {
            left: Box::new(expr),
            operator: Token::And,
            right: Box::new(right),
        };
    }

    Ok(expr)
}

fn equality(cursor: &mut Cursor) -> Result<Expression, String> {
    let mut expr = comparison(cursor)?;

    while cursor.peek_any(&[Token::BangEqual, Token::EqualEqual]) {
        let operator = cursor.advance_or_err()?;
        let right = comparison(cursor)?;
        expr = Expression::Binary {
            left: Box::new(expr),
            operator,
            right: Box::new(right),
        };
    }

    Ok(expr)
}

fn comparison(cursor: &mut Cursor) -> Result<Expression, String> {
    let mut expr = term(cursor)?;

    while cursor.peek_any(&[
        Token::Less,
        Token::LessEqual,
        Token::Greater,
        Token::GreaterEqual,
    ]) {
        let operator = cursor.advance_or_err()?;
        let right = term(cursor)?;
        expr = Expression::Binary {
            left: Box::new(expr),
            operator,
            right: Box::new(right),
        };
    }

    Ok(expr)
}

fn term(cursor: &mut Cursor) -> Result<Expression, String> {
    let mut expr = factor(cursor)?;

    while cursor.peek_any(&[Token::Plus, Token::Minus]) {
        let operator = cursor.advance_or_err()?;
        let right = factor(cursor)?;
        expr = Expression::Binary {
            left: Box::new(expr),
            operator,
            right: Box::new(right),
        };
    }

    Ok(expr)
}

fn factor(cursor: &mut Cursor) -> Result<Expression, String> {
    let mut expr = unary(cursor)?;

    while cursor.peek_any(&[Token::Asterisk, Token::Slash, Token::Percent]) {
        let operator = cursor.advance_or_err()?;
        let right = unary(cursor)?;
        expr = Expression::Binary {
            left: Box::new(expr),
            operator,
            right: Box::new(right),
        };
    }

    Ok(expr)
}

fn unary(cursor: &mut Cursor) -> Result<Expression, String> {
    if cursor.peek_any(&[Token::Bang, Token::Minus]) {
        let operator = cursor.advance_or_err()?;
        let operand = unary(cursor)?;
        return Ok(Expression::Unary {
            operator,
            operand: Box::new(operand),
        });
    }

    if matches!(cursor.peek(), Some(Token::Identifier(name)) if name == "len") {
        cursor.advance();
        return Ok(Expression::Len {
            operand: Box::new(unary(cursor)?),
        });
    }

    call_or_index(cursor)
}

fn call_or_index(cursor: &mut Cursor) -> Result<Expression, String> {
    let mut expr = primary(cursor)?;

    loop {
        if cursor.accept(&Token::LeftParen) {
            let arguments = expression_list(cursor, &Token::RightParen)?;
            cursor.expect(&Token::RightParen, "after function arguments")?;
            expr = Expression::Call {
                function: Box::new(expr),
                arguments,
            };
        } else if cursor.accept(&Token::LeftBracket) {
            let index = expression(cursor)?;
            cursor.expect(&Token::RightBracket, "after index expression")?;
            expr = Expression::Index {
                collection: Box::new(expr),
                index: Box::new(index),
            };
        } else {
            return Ok(expr);
        }
    }
}

fn expression_list(cursor: &mut Cursor, terminator: &Token) -> Result<Vec<Expression>, String> {
    let mut expressions = Vec::new();

    while !cursor.peek_is(terminator) && cursor.peek().is_some() {
        expressions.push(expression(cursor)?);
        if !cursor.accept(&Token::Comma) {
            break;
        }
    }

    Ok(expressions)
}

fn primary(cursor: &mut Cursor) -> Result<Expression, String> {
    match cursor.advance() {
        Some(t) if t.is_literal() => Ok(Expression::Literal { value: t }),
        Some(Token::Identifier(name)) => Ok(Expression::Variable { name }),
        Some(Token::LeftBracket) => {
            let elements = expression_list(cursor, &Token::RightBracket)?;
            cursor.expect(&Token::RightBracket, "after array elements")?;
            Ok(Expression::ArrayLiteral { elements })
        }
        Some(Token::LeftParen) => {
            let expr = expression(cursor)?;
            cursor.expect(&Token::RightParen, "after grouping expression")?;
            Ok(Expression::Grouping {
                expression: Box::new(expr),
            })
        }
        _ => Err("Expected expression".into()),
    }
}
