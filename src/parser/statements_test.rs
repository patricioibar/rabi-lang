//! Tests for what a single statement rule does with the token that ends it.
//! These reach an internal seam: the terminators other than `NewLine` are left
//! in place for an enclosing rule to read, which is not visible through the
//! parser's interface. Everything else is tested in `tests/parser.rs`.

use crate::{expression::Expression, parser::{cursor::Cursor, statements::statement}, statement::Statement, token::Token};

/// The statement every case here parses: the expression statement `1`.
fn one() -> Statement {
    Statement::ExpressionStatement(Expression::Literal {
        value: Token::Integer(1),
    })
}

#[test]
fn a_statement_may_end_at_end_of_input() {
    let mut cursor = Cursor::new(vec![Token::Integer(1)]);
    assert_eq!(statement(&mut cursor), Ok(one()));
    assert!(cursor.remaining().is_empty());
}

#[test]
fn an_empty_token_stream_is_an_error() {
    let mut cursor = Cursor::new(Vec::new());
    assert_eq!(
        statement(&mut cursor),
        Err("Unexpected end of input while parsing statement".to_string())
    );
}

#[test]
fn a_statement_consumes_its_terminating_newline() {
    let mut cursor = Cursor::new(vec![Token::Integer(1), Token::NewLine]);
    assert_eq!(statement(&mut cursor), Ok(one()));
    assert!(cursor.remaining().is_empty());
}

#[test]
fn a_statement_leaves_eof_in_place() {
    let mut cursor = Cursor::new(vec![Token::Integer(1), Token::Eof]);
    assert_eq!(statement(&mut cursor), Ok(one()));
    assert_eq!(cursor.remaining(), vec![Token::Eof]);
}

#[test]
fn a_dedent_terminates_a_statement_and_is_left_in_place() {
    let mut cursor = Cursor::new(vec![Token::Integer(1), Token::Dedent]);
    assert_eq!(statement(&mut cursor), Ok(one()));
    assert_eq!(cursor.remaining(), vec![Token::Dedent]);
}
