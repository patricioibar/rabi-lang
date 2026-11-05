use super::optimize;
use crate::expression::Expression;
use crate::statement::Statement;
use crate::token::Token;
use crate::{parser, scanner};

fn parsed(source: &str) -> Vec<Statement> {
    let tokens = scanner::scan(source.as_bytes()).unwrap();
    parser::parse(tokens).unwrap()
}

fn optimized(source: &str) -> Vec<Statement> {
    optimize(parsed(source))
}

fn printed(source: &str) -> Expression {
    match optimized(&format!("print {source}\n")).remove(0) {
        Statement::PrintStatement(value) => value,
        other => panic!("expected a print, got {other:?}"),
    }
}

fn unchanged(source: &str) {
    let statements = parsed(&format!("print {source}\n"));
    assert_eq!(optimize(statements.clone()), statements, "{source}");
}

fn literal(value: Token) -> Expression {
    Expression::Literal { value }
}

fn integer(value: i64) -> Expression {
    literal(Token::Integer(value))
}

fn variable(name: &str) -> Expression {
    Expression::Variable {
        name: name.to_string(),
    }
}

// ---------------------------------------------------------------- arithmetic

#[test]
fn arithmetic_on_literals_is_computed() {
    assert_eq!(printed("2 + 3 * 4"), integer(14));
    assert_eq!(printed("(2 + 3) * 4"), integer(20));
    assert_eq!(printed("10 - 2 - 3"), integer(5));
    assert_eq!(printed("7 / 2"), integer(3));
    assert_eq!(printed("-7 % 2"), integer(-1));
    assert_eq!(printed("1 + 0.5"), literal(Token::Decimal(1.5)));
}

#[test]
fn a_negative_literal_becomes_a_single_literal() {
    assert_eq!(printed("-5"), integer(-5));
    assert_eq!(printed("- -5"), integer(5));
    assert_eq!(printed("-1.5"), literal(Token::Decimal(-1.5)));
}

#[test]
fn strings_are_concatenated() {
    assert_eq!(
        printed("\"ab\" + \"cd\""),
        literal(Token::StringLiteral("abcd".to_string()))
    );
}

#[test]
fn string_repetition_is_left_for_run_time() {
    unchanged("\"ab\" * 3");
    unchanged("3 * \"ab\"");
}

// ----------------------------------------------------------- comparison, not

#[test]
fn comparisons_and_equality_on_literals_are_computed() {
    assert_eq!(printed("1 < 2"), literal(Token::True));
    assert_eq!(printed("2 <= 1"), literal(Token::False));
    assert_eq!(printed("3 > 2"), literal(Token::True));
    assert_eq!(printed("3 >= 4"), literal(Token::False));
    assert_eq!(printed("1 == 1.0"), literal(Token::True));
    assert_eq!(printed("\"a\" != \"a\""), literal(Token::False));
    assert_eq!(printed("null == false"), literal(Token::False));
}

#[test]
fn not_on_a_literal_is_computed() {
    assert_eq!(printed("!true"), literal(Token::False));
    assert_eq!(printed("!0"), literal(Token::True));
    assert_eq!(printed("!\"\""), literal(Token::True));
}

// ----------------------------------------------------------------- errors

#[test]
fn an_operation_that_would_fail_is_left_for_run_time() {
    unchanged("1 / 0");
    unchanged("1 % 0");
    unchanged("9223372036854775807 + 1");
    unchanged("-\"hi\"");
    unchanged("\"a\" + 1");
    unchanged("\"a\" < 1");
}

#[test]
fn only_the_failing_part_is_left_unfolded() {
    assert_eq!(
        printed("(1 + 1) / (2 - 2)"),
        Expression::Binary {
            left: Box::new(integer(2)),
            operator: Token::Slash,
            right: Box::new(integer(0)),
        }
    );
}

// --------------------------------------------------------------- variables

#[test]
fn an_operation_with_a_variable_is_kept_with_its_constant_parts_folded() {
    assert_eq!(
        optimized("let x = 1\nprint x + 2 * 3\n")[1],
        Statement::PrintStatement(Expression::Binary {
            left: Box::new(variable("x")),
            operator: Token::Plus,
            right: Box::new(integer(6)),
        })
    );
}

#[test]
fn operations_are_not_reassociated() {
    unchanged("x + 1 + 2");
}

#[test]
fn a_constant_grouping_becomes_its_value() {
    unchanged("(x)");
    assert_eq!(printed("(1)"), integer(1));
    assert_eq!(
        printed("x * (1 + 2)"),
        Expression::Binary {
            left: Box::new(variable("x")),
            operator: Token::Asterisk,
            right: Box::new(integer(3)),
        }
    );
}

// ------------------------------------------------------------ and, or

#[test]
fn or_is_computed_when_its_result_is_a_constant() {
    assert_eq!(printed("1 or x"), integer(1));
    assert_eq!(
        printed("null or \"b\""),
        literal(Token::StringLiteral("b".to_string()))
    );
}

#[test]
fn and_is_computed_when_its_result_is_a_constant() {
    assert_eq!(printed("0 and x"), literal(Token::False));
    assert_eq!(printed("1 and 2"), literal(Token::True));
    assert_eq!(printed("1 and \"\""), literal(Token::False));
}

#[test]
fn and_or_whose_result_depends_on_a_variable_are_kept() {
    unchanged("x and 1");
    unchanged("x or 1");
    unchanged("0 or x");
    unchanged("1 and x");
}

// ------------------------------------------------------------ everywhere

#[test]
fn folding_reaches_every_nested_expression() {
    let statements = optimized(
        "func f(a):\n    while a < 2 * 5:\n        a = a + 1 * 1\n    return [1 + 1][0 * 1]\n",
    );
    let Statement::FunctionDeclaration { body, .. } = &statements[0] else {
        panic!("expected a function");
    };
    let local = || variable("a");
    assert_eq!(
        body[..],
        [
            Statement::WhileStatement {
                condition: Expression::Binary {
                    left: Box::new(local()),
                    operator: Token::Less,
                    right: Box::new(integer(10)),
                },
                block: vec![Statement::ExpressionStatement(Expression::Assignment {
                    name: "a".to_string(),
                    value: Box::new(Expression::Binary {
                        left: Box::new(local()),
                        operator: Token::Plus,
                        right: Box::new(integer(1)),
                    }),
                })],
            },
            Statement::ReturnStatement {
                value: Some(Expression::Index {
                    collection: Box::new(Expression::ArrayLiteral {
                        elements: vec![integer(2)],
                    }),
                    index: Box::new(integer(0)),
                }),
            },
        ]
    );
}

#[test]
fn an_array_literal_is_never_folded_into_a_constant() {
    unchanged("[1, 2]");
    unchanged("[1] == [1]");
}
