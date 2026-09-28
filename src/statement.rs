use crate::expression::Expression;

#[derive(Debug, PartialEq, Clone)]
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
    ForStatement {
        variable: String,
        iterable: Expression,
        block: Vec<Statement>,
    },
    BreakStatement,
    ContinueStatement,
}
