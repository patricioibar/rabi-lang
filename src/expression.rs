use crate::token::Token;

#[derive(Debug)]
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

type Cursor = std::iter::Peekable<std::vec::IntoIter<Token>>;

impl Expression {
    pub fn get_next(cursor: &mut Cursor) -> Result<Self, String> {
        if let Some(token) = cursor.next() {
            match token {
                t if t.is_literal() => Ok(Expression::Literal { value: t }),

                Token::Bang | Token::Minus => {
                    let operand = Expression::get_next(cursor)?;
                    Ok(Expression::Unary {
                        operator: token,
                        operand: Box::new(operand),
                    })
                }

                Token::Let => match cursor.next() {
                    Some(Token::Identifier(name)) => match cursor.next() {
                        Some(Token::Equal) => {
                            let value = Expression::get_next(cursor)?;
                            Ok(Expression::Assignment {
                                name,
                                value: Box::new(value),
                            })
                        }
                        _ => Err(format!(
                            "Expected '=' after identifier in assignment, got {:?}",
                            cursor.peek()
                        )),
                    },
                    _ => Err(format!(
                        "Expected identifier after 'let', got {:?}",
                        cursor.peek()
                    )),
                },
                Token::Identifier(name) => Ok(Expression::Variable { name }),
                _ => Err(format!("Unexpected token: {:?}", token)),
            }
        } else {
            Err("No more tokens".to_string())
        }
    }
}
