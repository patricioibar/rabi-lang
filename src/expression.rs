use std::todo;

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

type TokenCursor = std::iter::Peekable<std::vec::IntoIter<Token>>;

impl Expression {
    pub fn get_next(_cursor: &mut TokenCursor) -> Result<Self, String> {
        todo!()
    }
}
