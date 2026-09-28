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
    ArrayLiteral {
        elements: Vec<Expression>,
    },
    Index {
        collection: Box<Expression>,
        index: Box<Expression>,
    },
    Len {
        operand: Box<Expression>,
    },
    Assignment {
        name: String,
        value: Box<Expression>,
    },
    IndexAssignment {
        collection: Box<Expression>,
        index: Box<Expression>,
        value: Box<Expression>,
    },
    Call {
        function: Box<Expression>,
        arguments: Vec<Expression>,
    },
}
