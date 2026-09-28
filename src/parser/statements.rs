use super::{cursor::Cursor, expressions::expression};
use crate::{statement::Statement, token::Token};

pub(super) fn statement(cursor: &mut Cursor) -> Result<Statement, String> {
    cursor.skip_blank_lines();

    let stmt = match cursor.peek_cloned() {
        Some(Token::Let) => {
            cursor.advance();
            variable_declaration(cursor)?
        }
        Some(Token::Function) => {
            cursor.advance();
            return function_declaration(cursor);
        }
        Some(Token::If) => {
            cursor.advance();
            return if_statement(cursor);
        }
        Some(Token::While) => {
            cursor.advance();
            return while_statement(cursor);
        }
        Some(Token::For) => {
            cursor.advance();
            return for_statement(cursor);
        }
        Some(Token::Return) => {
            cursor.advance();
            return_statement(cursor)?
        }
        Some(Token::Break) => {
            cursor.advance();
            Statement::BreakStatement
        }
        Some(Token::Continue) => {
            cursor.advance();
            Statement::ContinueStatement
        }
        Some(Token::Identifier(s)) if s == "print" => {
            cursor.advance();
            let expr = expression(cursor)?;
            Statement::PrintStatement(expr)
        }
        Some(Token::Indent) | Some(Token::Dedent) => {
            return Err("Unexpected indentation at start of statement".to_string());
        }
        Some(_) => Statement::ExpressionStatement(expression(cursor)?),
        None => return Err("Unexpected end of input while parsing statement".to_string()),
    };

    cursor.end_statement()?;
    Ok(stmt)
}

fn variable_declaration(cursor: &mut Cursor) -> Result<Statement, String> {
    let name = identifier(cursor, "let")?;

    let initializer = if cursor.accept(&Token::Equal) {
        Some(expression(cursor)?)
    } else {
        None
    };
    Ok(Statement::VariableDeclaration { name, initializer })
}

fn function_declaration(cursor: &mut Cursor) -> Result<Statement, String> {
    let name = identifier(cursor, "func")?;

    cursor.expect(&Token::LeftParen, "after function name")?;
    let parameters = parameter_list(cursor)?;
    cursor.expect(&Token::RightParen, "after function parameters")?;

    let body = block_body(cursor, "to start function body")?;

    Ok(Statement::FunctionDeclaration {
        name,
        parameters,
        body,
    })
}

fn if_statement(cursor: &mut Cursor) -> Result<Statement, String> {
    let condition = expression(cursor)?;
    let then_branch = block_body(cursor, "after if condition")?;

    let else_branch = if cursor.accept(&Token::Else) {
        if cursor.accept(&Token::If) {
            Some(vec![if_statement(cursor)?])
        } else {
            Some(block_body(cursor, "after else")?)
        }
    } else {
        None
    };

    Ok(Statement::IfStatement {
        condition,
        then_branch,
        else_branch,
    })
}

fn while_statement(cursor: &mut Cursor) -> Result<Statement, String> {
    let condition = expression(cursor)?;
    let block = block_body(cursor, "after while condition")?;

    Ok(Statement::WhileStatement { condition, block })
}

fn for_statement(cursor: &mut Cursor) -> Result<Statement, String> {
    let variable = identifier(cursor, "for")?;
    cursor.expect(&Token::In, "after the loop variable")?;
    let iterable = expression(cursor)?;
    let block = block_body(cursor, "after the iterated expression")?;

    Ok(Statement::ForStatement {
        variable,
        iterable,
        block,
    })
}

fn return_statement(cursor: &mut Cursor) -> Result<Statement, String> {
    let value = if cursor.at_statement_terminator() {
        None
    } else {
        Some(expression(cursor)?)
    };

    Ok(Statement::ReturnStatement { value })
}

fn block_body(cursor: &mut Cursor, context: &str) -> Result<Vec<Statement>, String> {
    cursor.expect(&Token::Colon, context)?;

    if cursor.accept(&Token::NewLine) {
        cursor.skip_blank_lines();
        cursor.expect(&Token::Indent, "after ':'")?;
        indented_block(cursor)
    } else {
        Ok(vec![statement(cursor)?])
    }
}

fn indented_block(cursor: &mut Cursor) -> Result<Vec<Statement>, String> {
    let mut statements = Vec::new();
    loop {
        cursor.skip_blank_lines();
        if cursor.accept(&Token::Dedent) || cursor.at_end() {
            break;
        }
        statements.push(statement(cursor)?);
    }
    Ok(statements)
}

fn parameter_list(cursor: &mut Cursor) -> Result<Vec<String>, String> {
    let mut parameters = Vec::new();
    loop {
        match cursor.peek_cloned() {
            Some(Token::Identifier(parameter)) => {
                cursor.advance();
                parameters.push(parameter);
                cursor.accept(&Token::Comma);
            }
            Some(Token::RightParen) | None => break,
            Some(other) => {
                return Err(format!("Unexpected token in parameter list: {:?}", other));
            }
        }
    }
    Ok(parameters)
}

fn identifier(cursor: &mut Cursor, keyword: &str) -> Result<String, String> {
    match cursor.advance() {
        Some(Token::Identifier(name)) => Ok(name),
        found => Err(format!(
            "Expected identifier after '{}', found {:?}",
            keyword, found
        )),
    }
}
