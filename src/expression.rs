use crate::token::Token;

#[derive(Debug, PartialEq, Clone)]
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
