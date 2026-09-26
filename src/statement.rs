use crate::{expression::Expression, token::Token};

#[derive(Debug, PartialEq)]
pub enum Statement {
    ExpressionStatement(Expression),
    PrintStatement(Expression),
    VariableDeclaration {
        name: String,
        initializer: Option<Expression>,
    },
    FunctionDeclaration {
        name: String,
        parameters: Vec<String>,
        body: Vec<Statement>,
    },
    ReturnStatement {
        value: Option<Expression>,
    },
    IfStatement {
        condition: Expression,
        then_branch: Vec<Statement>,
        else_branch: Option<Vec<Statement>>,
    },
    WhileStatement {
        condition: Expression,
        block: Vec<Statement>,
    },
    BreakStatement,
    ContinueStatement,
}

type TokenCursor = std::iter::Peekable<std::vec::IntoIter<Token>>;

impl Statement {
    pub fn get_next(cursor: &mut TokenCursor) -> Result<Self, String> {
        Self::skip_blank_lines(cursor);

        let stmt = if let Some(token) = cursor.peek() {
            match token {
                Token::Let => {
                    cursor.next();
                    Self::variable_declaration(cursor)?
                }
                Token::Function => {
                    cursor.next();
                    return Self::function_declaration(cursor);
                }
                Token::If => {
                    cursor.next();
                    return Self::if_statement(cursor);
                }
                Token::While => {
                    cursor.next();
                    return Self::while_statement(cursor);
                }
                Token::For => {
                    cursor.next();
                    // todo!
                    return Err("for loops not yet supported".to_string());
                }
                Token::Return => {
                    cursor.next();
                    Self::return_statement(cursor)?
                }
                Token::Break => {
                    cursor.next();
                    Statement::BreakStatement
                }
                Token::Continue => {
                    cursor.next();
                    Statement::ContinueStatement
                }
                _ => {
                    let expr = Expression::get_next(cursor)?;
                    Statement::ExpressionStatement(expr)
                }
            }
        } else {
            return Err("Unexpected end of input while parsing statement".to_string());
        };

        if !is_statement_terminator(cursor.peek()) {
            return Err(format!(
                "Expected newline or end of file after statement, found {:?}",
                cursor.peek()
            ));
        }
        if cursor.peek() == Some(&Token::NewLine) {
            cursor.next();
        }
        Ok(stmt)
    }

    pub fn skip_blank_lines(cursor: &mut TokenCursor) {
        while cursor.peek() == Some(&Token::NewLine) {
            cursor.next();
        }
    }

    fn variable_declaration(cursor: &mut TokenCursor) -> Result<Self, String> {
        let name = match cursor.next() {
            Some(Token::Identifier(name)) => name,
            t => return Err(format!("Expected identifier after 'let', found {:?}", t)),
        };

        let initializer = if let Some(Token::Equal) = cursor.peek() {
            cursor.next();
            Some(Expression::get_next(cursor)?)
        } else {
            None
        };
        Ok(Statement::VariableDeclaration { name, initializer })
    }

    fn function_declaration(cursor: &mut TokenCursor) -> Result<Self, String> {
        let name = match cursor.next() {
            Some(Token::Identifier(name)) => name,
            t => return Err(format!("Expected identifier after 'func', found {:?}", t)),
        };

        if cursor.next() != Some(Token::LeftParen) {
            return Err("Expected '(' after function name".to_string());
        }

        let mut parameters = Vec::new();
        while let Some(token) = cursor.peek() {
            match token {
                Token::Identifier(param) => {
                    parameters.push(param.clone());
                    cursor.next();
                    if cursor.peek() == Some(&Token::Comma) {
                        cursor.next();
                    }
                }
                Token::RightParen => break,
                _ => return Err(format!("Unexpected token in parameter list: {:?}", token)),
            }
        }

        if cursor.next() != Some(Token::RightParen) {
            return Err("Expected ')' after function parameters".to_string());
        }

        if cursor.next() != Some(Token::Colon) {
            return Err("Expected ':' to start function body".to_string());
        }

        let is_inline = if cursor.peek() == Some(&Token::NewLine) {
            cursor.next();
            if cursor.next() != Some(Token::Indent) {
                return Err("Expected indent after ':' in function declaration".to_string());
            }
            false
        } else {
            true
        };

        let body = if is_inline {
            vec![Self::get_next(cursor)?]
        } else {
            Self::block(cursor)?
        };

        Ok(Statement::FunctionDeclaration {
            name,
            parameters,
            body,
        })
    }

    fn if_statement(cursor: &mut TokenCursor) -> Result<Self, String> {
        let condition = Expression::get_next(cursor)?;

        if cursor.next() != Some(Token::Colon) {
            return Err("Expected ':' after if condition".to_string());
        }

        let is_inline = if cursor.peek() == Some(&Token::NewLine) {
            cursor.next();
            if cursor.next() != Some(Token::Indent) {
                return Err("Expected indent after ':' in if statement".to_string());
            }
            false
        } else {
            true
        };

        let then_branch = if is_inline {
            vec![Self::get_next(cursor)?]
        } else {
            Self::block(cursor)?
        };

        let else_branch = if let Some(Token::Else) = cursor.peek() {
            cursor.next();
            if cursor.next() != Some(Token::Colon) {
                return Err("Expected ':' after 'else'".to_string());
            }

            let is_inline_else = if cursor.peek() == Some(&Token::NewLine) {
                cursor.next();
                if cursor.next() != Some(Token::Indent) {
                    return Err("Expected indent after ':' in else statement".to_string());
                }
                false
            } else {
                true
            };

            let else_body = if is_inline_else {
                vec![Self::get_next(cursor)?]
            } else {
                Self::block(cursor)?
            };
            Some(else_body)
        } else {
            None
        };

        Ok(Statement::IfStatement {
            condition,
            then_branch,
            else_branch,
        })
    }

    fn while_statement(cursor: &mut TokenCursor) -> Result<Self, String> {
        let condition = Expression::get_next(cursor)?;

        if cursor.next() != Some(Token::Colon) {
            return Err("Expected ':' after while condition".to_string());
        }

        let is_inline = if cursor.peek() == Some(&Token::NewLine) {
            cursor.next();
            if cursor.next() != Some(Token::Indent) {
                return Err("Expected indent after ':' in while statement".to_string());
            }
            false
        } else {
            true
        };

        let block = if is_inline {
            vec![Self::get_next(cursor)?]
        } else {
            Self::block(cursor)?
        };

        Ok(Statement::WhileStatement { condition, block })
    }

    fn return_statement(cursor: &mut TokenCursor) -> Result<Self, String> {
        let value = if is_statement_terminator(cursor.peek()) {
            None
        } else {
            Some(Expression::get_next(cursor)?)
        };

        Ok(Statement::ReturnStatement { value })
    }

    fn block(cursor: &mut TokenCursor) -> Result<Vec<Self>, String> {
        let mut statements = Vec::new();
        loop {
            Self::skip_blank_lines(cursor);
            match cursor.peek() {
                Some(Token::Dedent) => {
                    cursor.next();
                    break;
                }
                Some(Token::Eof) | None => break,
                _ => {
                    let stmt = Self::get_next(cursor)?;
                    statements.push(stmt);
                }
            }
        }
        Ok(statements)
    }
}

fn is_statement_terminator(token: Option<&Token>) -> bool {
    matches!(
        token,
        Some(Token::NewLine) | Some(Token::Eof) | Some(Token::Dedent) | Some(Token::Else) | None
    )
}
