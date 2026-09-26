use crate::token::Token;

#[derive(Debug, PartialEq)]
pub enum Expression {
    Literal {
        value: Token,
    },
    Unary {
        operator: Token,
        operand: Box<Expression>,
    },
    Binary {
        left: Box<Expression>,
        operator: Token,
        right: Box<Expression>,
    },
    Grouping {
        expression: Box<Expression>,
    },
    Variable {
        name: String,
    },
    Assignment {
        name: String,
        value: Box<Expression>,
    },
    Call {
        function: Box<Expression>,
        arguments: Vec<Expression>,
    },
}

type TokenCursor = std::iter::Peekable<std::vec::IntoIter<Token>>;

impl Expression {
    pub fn get_next(cursor: &mut TokenCursor) -> Result<Self, String> {
        Self::assignment(cursor)
    }

    fn assignment(cursor: &mut TokenCursor) -> Result<Expression, String> {
        let expr = Self::logic_or(cursor)?;

        if let Some(Token::Equal) = cursor.peek() {
            cursor.next(); // consume the '=' token
            if let Expression::Variable { name } = expr {
                let value = Self::assignment(cursor)?;
                return Ok(Expression::Assignment {
                    name,
                    value: Box::new(value),
                });
            } else {
                return Err("Invalid assignment target.".to_string());
            }
        }

        Ok(expr)
    }

    fn logic_or(cursor: &mut TokenCursor) -> Result<Expression, String> {
        let mut expr = Self::logic_and(cursor)?;

        while matches!(cursor.peek(), Some(Token::Or)) {
            cursor.next();
            let right = Self::logic_and(cursor)?;
            expr = Expression::Binary {
                left: Box::new(expr),
                operator: Token::Or,
                right: Box::new(right),
            };
        }

        Ok(expr)
    }

    fn logic_and(cursor: &mut TokenCursor) -> Result<Expression, String> {
        let mut expr = Self::equality(cursor)?;

        while matches!(cursor.peek(), Some(Token::And)) {
            cursor.next();
            let right = Self::equality(cursor)?;
            expr = Expression::Binary {
                left: Box::new(expr),
                operator: Token::And,
                right: Box::new(right),
            };
        }
        Ok(expr)
    }

    fn equality(cursor: &mut TokenCursor) -> Result<Expression, String> {
        let mut expr = Self::comparison(cursor)?;

        while matches!(
            cursor.peek(),
            Some(Token::BangEqual) | Some(Token::EqualEqual)
        ) {
            let operator = next_or_err(cursor)?;
            let right = Self::comparison(cursor)?;
            expr = Expression::Binary {
                left: Box::new(expr),
                operator,
                right: Box::new(right),
            };
        }

        Ok(expr)
    }

    fn comparison(cursor: &mut TokenCursor) -> Result<Expression, String> {
        let mut expr = Self::term(cursor)?;

        while matches!(
            cursor.peek(),
            Some(Token::Less)
                | Some(Token::LessEqual)
                | Some(Token::Greater)
                | Some(Token::GreaterEqual)
        ) {
            let operator = next_or_err(cursor)?;
            let right = Self::term(cursor)?;
            expr = Expression::Binary {
                left: Box::new(expr),
                operator,
                right: Box::new(right),
            };
        }

        Ok(expr)
    }

    fn term(cursor: &mut TokenCursor) -> Result<Expression, String> {
        let mut expr = Self::factor(cursor)?;

        while matches!(cursor.peek(), Some(Token::Plus) | Some(Token::Minus)) {
            let operator = next_or_err(cursor)?;
            let right = Self::factor(cursor)?;
            expr = Expression::Binary {
                left: Box::new(expr),
                operator,
                right: Box::new(right),
            };
        }

        Ok(expr)
    }

    fn factor(cursor: &mut TokenCursor) -> Result<Expression, String> {
        let mut expr = Self::unary(cursor)?;

        while matches!(cursor.peek(), Some(Token::Asterisk) | Some(Token::Slash)) {
            let operator = next_or_err(cursor)?;
            let right = Self::unary(cursor)?;
            expr = Expression::Binary {
                left: Box::new(expr),
                operator,
                right: Box::new(right),
            };
        }

        Ok(expr)
    }

    fn unary(cursor: &mut TokenCursor) -> Result<Expression, String> {
        if matches!(cursor.peek(), Some(Token::Bang) | Some(Token::Minus)) {
            let operator = next_or_err(cursor)?;
            let operand = Self::unary(cursor)?;
            return Ok(Expression::Unary {
                operator,
                operand: Box::new(operand),
            });
        }

        Self::function_call(cursor)
    }

    fn function_call(cursor: &mut TokenCursor) -> Result<Expression, String> {
        let mut expr = Self::primary(cursor)?;

        while matches!(cursor.peek(), Some(Token::LeftParen)) {
            cursor.next(); // consume '('
            let mut arguments = Vec::new();

            loop {
                match cursor.peek() {
                    Some(Token::RightParen) => break,
                    Some(_) => {
                        let arg = Self::get_next(cursor)?;
                        arguments.push(arg);
                        if matches!(cursor.peek(), Some(Token::Comma)) {
                            cursor.next(); // consume ','
                        } else {
                            break;
                        }
                    }
                    _ => break,
                }
            }

            if !matches!(cursor.next(), Some(Token::RightParen)) {
                return Err("Expected ')' after function arguments.".to_string());
            }

            expr = Expression::Call {
                function: Box::new(expr),
                arguments,
            };
        }

        Ok(expr)
    }

    fn primary(cursor: &mut TokenCursor) -> Result<Expression, String> {
        match cursor.next() {
            Some(t) if t.is_literal() => Ok(Expression::Literal { value: t }),
            Some(Token::Identifier(name)) => Ok(Expression::Variable { name }),
            Some(Token::LeftParen) => {
                let expr = Self::get_next(cursor)?;
                match cursor.next() {
                    Some(Token::RightParen) => Ok(Expression::Grouping {
                        expression: Box::new(expr),
                    }),
                    _ => Err("Expected ')' after grouping expression.".to_string()),
                }
            }
            _ => Err("Expected expression".into()),
        }
    }
}

fn next_or_err(cursor: &mut TokenCursor) -> Result<Token, String> {
    cursor
        .next()
        .ok_or_else(|| "Unexpected end of input".into())
}
