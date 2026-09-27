//! Tests for the expression rules. These reach an internal seam: the rules are
//! the parser's implementation, not its interface.

use crate::{
    expression::Expression,
    parser::{cursor::Cursor, expressions::expression},
    scanner::scan,
    token::Token,
};

// --- Helpers ---

/// Scans `source` and drops the stream terminators, leaving just the tokens the
/// expression rules are meant to see.
fn tokens(source: &str) -> Vec<Token> {
    let mut tokens = scan(source.as_bytes()).expect("source should scan");
    assert_eq!(tokens.pop(), Some(Token::Eof));
    if tokens.last() == Some(&Token::NewLine) {
        tokens.pop();
    }
    tokens
}

/// Parses one expression and asserts the whole input was consumed.
fn parse(source: &str) -> Expression {
    let (expr, rest) = parse_partial(source);
    assert!(
        rest.is_empty(),
        "{source:?} left unconsumed tokens: {rest:?}"
    );
    expr
}

/// Parses one expression and returns whatever tokens are left over.
fn parse_partial(source: &str) -> (Expression, Vec<Token>) {
    let mut cursor = Cursor::new(tokens(source));
    let expr = expression(&mut cursor)
        .unwrap_or_else(|e| panic!("{source:?} should parse, got error: {e}"));
    (expr, cursor.remaining())
}

fn error(source: &str) -> String {
    let mut cursor = Cursor::new(tokens(source));
    expression(&mut cursor).expect_err(&format!("{source:?} should not parse"))
}

/// Parses a token stream directly, for inputs the scanner cannot produce.
fn parse_tokens(tokens: Vec<Token>) -> Result<Expression, String> {
    expression(&mut Cursor::new(tokens))
}

// --- Builders, so the expected trees stay readable ---

fn int(n: i64) -> Expression {
    Expression::Literal {
        value: Token::Integer(n),
    }
}

fn var(name: &str) -> Expression {
    Expression::Variable {
        name: name.to_string(),
    }
}

fn binary(left: Expression, operator: Token, right: Expression) -> Expression {
    Expression::Binary {
        left: Box::new(left),
        operator,
        right: Box::new(right),
    }
}

fn unary(operator: Token, operand: Expression) -> Expression {
    Expression::Unary {
        operator,
        operand: Box::new(operand),
    }
}

fn group(expression: Expression) -> Expression {
    Expression::Grouping {
        expression: Box::new(expression),
    }
}

fn call(function: Expression, arguments: Vec<Expression>) -> Expression {
    Expression::Call {
        function: Box::new(function),
        arguments,
    }
}

fn assign(name: &str, value: Expression) -> Expression {
    Expression::Assignment {
        name: name.to_string(),
        value: Box::new(value),
    }
}

// --- Primary: literals ---

#[test]
fn integer_literal() {
    assert_eq!(parse("42"), int(42));
}

#[test]
fn decimal_literal() {
    assert_eq!(
        parse("3.5"),
        Expression::Literal {
            value: Token::Decimal(3.5)
        }
    );
}

#[test]
fn string_literal() {
    assert_eq!(
        parse("\"hi\""),
        Expression::Literal {
            value: Token::StringLiteral("hi".into())
        }
    );
}

#[test]
fn boolean_literals() {
    assert_eq!(parse("true"), Expression::Literal { value: Token::True });
    assert_eq!(
        parse("false"),
        Expression::Literal {
            value: Token::False
        }
    );
}

// --- Primary: variables ---

#[test]
fn variable_reference() {
    assert_eq!(parse("x"), var("x"));
    assert_eq!(parse("some_name"), var("some_name"));
}

// --- Primary: grouping ---

#[test]
fn grouping_wraps_its_inner_expression() {
    assert_eq!(parse("(1)"), group(int(1)));
}

#[test]
fn grouping_can_nest() {
    assert_eq!(parse("((1))"), group(group(int(1))));
}

#[test]
fn grouping_overrides_precedence() {
    assert_eq!(
        parse("(1 + 2) * 3"),
        binary(
            group(binary(int(1), Token::Plus, int(2))),
            Token::Asterisk,
            int(3)
        )
    );
}

#[test]
fn an_empty_group_is_an_error() {
    assert_eq!(error("()"), "Expected expression");
}

#[test]
fn an_unclosed_group_is_an_error() {
    assert_eq!(error("(1"), "Expected ')' after grouping expression");
}

// --- Unary ---

#[test]
fn unary_minus() {
    assert_eq!(parse("-1"), unary(Token::Minus, int(1)));
}

#[test]
fn unary_bang() {
    assert_eq!(parse("!x"), unary(Token::Bang, var("x")));
}

#[test]
fn unary_operators_stack() {
    assert_eq!(
        parse("--1"),
        unary(Token::Minus, unary(Token::Minus, int(1)))
    );
    assert_eq!(
        parse("!!x"),
        unary(Token::Bang, unary(Token::Bang, var("x")))
    );
    assert_eq!(
        parse("!-x"),
        unary(Token::Bang, unary(Token::Minus, var("x")))
    );
}

#[test]
fn unary_binds_tighter_than_multiplication() {
    assert_eq!(
        parse("-1 * 2"),
        binary(unary(Token::Minus, int(1)), Token::Asterisk, int(2))
    );
}

#[test]
fn unary_without_an_operand_is_an_error() {
    assert_eq!(error("-"), "Expected expression");
    assert_eq!(error("!"), "Expected expression");
}

// --- Binary operators, one per precedence level ---

#[test]
fn factor_operators() {
    assert_eq!(parse("6 * 7"), binary(int(6), Token::Asterisk, int(7)));
    assert_eq!(parse("6 / 7"), binary(int(6), Token::Slash, int(7)));
}

#[test]
fn term_operators() {
    assert_eq!(parse("1 + 2"), binary(int(1), Token::Plus, int(2)));
    assert_eq!(parse("1 - 2"), binary(int(1), Token::Minus, int(2)));
}

#[test]
fn comparison_operators() {
    for operator in [
        Token::Less,
        Token::LessEqual,
        Token::Greater,
        Token::GreaterEqual,
    ] {
        let source = match operator {
            Token::Less => "a < b",
            Token::LessEqual => "a <= b",
            Token::Greater => "a > b",
            _ => "a >= b",
        };
        assert_eq!(parse(source), binary(var("a"), operator, var("b")));
    }
}

#[test]
fn equality_operators() {
    assert_eq!(
        parse("a == b"),
        binary(var("a"), Token::EqualEqual, var("b"))
    );
    assert_eq!(
        parse("a != b"),
        binary(var("a"), Token::BangEqual, var("b"))
    );
}

#[test]
fn logical_operators() {
    assert_eq!(parse("a and b"), binary(var("a"), Token::And, var("b")));
    assert_eq!(parse("a or b"), binary(var("a"), Token::Or, var("b")));
}

// --- Precedence between levels ---

#[test]
fn multiplication_binds_tighter_than_addition() {
    assert_eq!(
        parse("1 + 2 * 3"),
        binary(int(1), Token::Plus, binary(int(2), Token::Asterisk, int(3)))
    );
    assert_eq!(
        parse("1 * 2 + 3"),
        binary(binary(int(1), Token::Asterisk, int(2)), Token::Plus, int(3))
    );
}

#[test]
fn division_binds_tighter_than_subtraction() {
    assert_eq!(
        parse("1 - 4 / 2"),
        binary(int(1), Token::Minus, binary(int(4), Token::Slash, int(2)))
    );
}

#[test]
fn arithmetic_binds_tighter_than_comparison() {
    assert_eq!(
        parse("1 + 2 < 4"),
        binary(binary(int(1), Token::Plus, int(2)), Token::Less, int(4))
    );
}

#[test]
fn comparison_binds_tighter_than_equality() {
    assert_eq!(
        parse("a < b == c"),
        binary(
            binary(var("a"), Token::Less, var("b")),
            Token::EqualEqual,
            var("c")
        )
    );
}

#[test]
fn equality_binds_tighter_than_and() {
    assert_eq!(
        parse("a == b and c"),
        binary(
            binary(var("a"), Token::EqualEqual, var("b")),
            Token::And,
            var("c")
        )
    );
}

#[test]
fn and_binds_tighter_than_or() {
    assert_eq!(
        parse("a and b or c"),
        binary(binary(var("a"), Token::And, var("b")), Token::Or, var("c"))
    );
    assert_eq!(
        parse("a or b and c"),
        binary(var("a"), Token::Or, binary(var("b"), Token::And, var("c")))
    );
}

#[test]
fn a_full_precedence_chain() {
    // -1 * 2 + 3 < 4 == true and false
    assert_eq!(
        parse("-1 * 2 + 3 < 4 == true and false"),
        binary(
            binary(
                binary(
                    binary(
                        binary(unary(Token::Minus, int(1)), Token::Asterisk, int(2)),
                        Token::Plus,
                        int(3)
                    ),
                    Token::Less,
                    int(4)
                ),
                Token::EqualEqual,
                Expression::Literal { value: Token::True }
            ),
            Token::And,
            Expression::Literal {
                value: Token::False
            }
        )
    );
}

// --- Assignment ---

#[test]
fn assignment_to_a_variable() {
    assert_eq!(parse("x = 1"), assign("x", int(1)));
}

#[test]
fn assignment_is_right_associative() {
    assert_eq!(parse("x = y = 1"), assign("x", assign("y", int(1))));
}

#[test]
fn assignment_has_the_lowest_precedence() {
    assert_eq!(
        parse("x = 1 + 2"),
        assign("x", binary(int(1), Token::Plus, int(2)))
    );
    assert_eq!(
        parse("x = a or b"),
        assign("x", binary(var("a"), Token::Or, var("b")))
    );
}

#[test]
fn assigning_to_a_literal_is_an_error() {
    assert_eq!(error("1 = 2"), "Invalid assignment target");
}

#[test]
fn assigning_to_a_group_is_an_error() {
    assert_eq!(error("(x) = 1"), "Invalid assignment target");
}

#[test]
fn assigning_to_a_call_is_an_error() {
    assert_eq!(error("f() = 1"), "Invalid assignment target");
}

#[test]
fn assignment_without_a_value_is_an_error() {
    assert_eq!(error("x ="), "Expected expression");
}

// --- Function calls ---

#[test]
fn call_with_no_arguments() {
    assert_eq!(parse("f()"), call(var("f"), vec![]));
}

#[test]
fn call_with_one_argument() {
    assert_eq!(parse("f(1)"), call(var("f"), vec![int(1)]));
}

#[test]
fn call_with_several_arguments() {
    assert_eq!(
        parse("f(1, x, 2 + 3)"),
        call(
            var("f"),
            vec![int(1), var("x"), binary(int(2), Token::Plus, int(3))]
        )
    );
}

#[test]
fn call_arguments_may_be_assignments() {
    // Arguments are parsed at the lowest precedence level.
    assert_eq!(parse("f(x = 1)"), call(var("f"), vec![assign("x", int(1))]));
}

#[test]
fn calls_chain_left_to_right() {
    assert_eq!(
        parse("f(1)(2)"),
        call(call(var("f"), vec![int(1)]), vec![int(2)])
    );
}

#[test]
fn nested_calls() {
    assert_eq!(
        parse("f(g(1))"),
        call(var("f"), vec![call(var("g"), vec![int(1)])])
    );
}

#[test]
fn a_call_binds_tighter_than_arithmetic() {
    assert_eq!(
        parse("f(1) + 2"),
        binary(call(var("f"), vec![int(1)]), Token::Plus, int(2))
    );
}

#[test]
fn a_call_can_be_negated() {
    assert_eq!(
        parse("-f(1)"),
        unary(Token::Minus, call(var("f"), vec![int(1)]))
    );
}

#[test]
fn a_group_can_be_called() {
    assert_eq!(parse("(f)()"), call(group(var("f")), vec![]));
}

#[test]
fn a_missing_closing_paren_in_a_call_is_an_error() {
    assert_eq!(error("f(1"), "Expected ')' after function arguments");
}

#[test]
fn a_trailing_comma_in_a_call_is_allowed() {
    // After consuming the comma the loop re-peeks, sees `)` and stops.
    assert_eq!(parse("f(1,)"), call(var("f"), vec![int(1)]));
    assert_eq!(parse("f(1, 2,)"), call(var("f"), vec![int(1), int(2)]));
}

#[test]
fn a_leading_comma_in_a_call_is_an_error() {
    assert_eq!(error("f(,1)"), "Expected expression");
}

// --- Empty and malformed input ---

#[test]
fn empty_input_is_an_error() {
    assert_eq!(error(""), "Expected expression");
}

#[test]
fn a_dangling_operator_is_an_error() {
    assert_eq!(error("1 +"), "Expected expression");
    assert_eq!(error("1 *"), "Expected expression");
    assert_eq!(error("a and"), "Expected expression");
}

#[test]
fn a_leading_binary_operator_is_an_error() {
    assert_eq!(error("* 1"), "Expected expression");
    assert_eq!(error("/ 1"), "Expected expression");
    assert_eq!(error("and a"), "Expected expression");
}

#[test]
fn a_keyword_is_not_an_expression() {
    assert_eq!(error("if"), "Expected expression");
    assert_eq!(error("return"), "Expected expression");
}

#[test]
fn structural_tokens_are_not_expressions() {
    for token in [Token::NewLine, Token::Indent, Token::Dedent, Token::Eof] {
        assert_eq!(
            parse_tokens(vec![token.clone()]),
            Err("Expected expression".to_string()),
            "for {token:?}"
        );
    }
}

// --- Consumption: the parser must stop at the first token it cannot use ---

#[test]
fn parsing_stops_before_a_newline() {
    let mut cursor = Cursor::new(vec![Token::Integer(1), Token::NewLine]);
    assert_eq!(expression(&mut cursor), Ok(int(1)));
    assert_eq!(cursor.remaining(), vec![Token::NewLine]);
}

#[test]
fn parsing_stops_before_a_colon() {
    let (expr, rest) = parse_partial("x:");
    assert_eq!(expr, var("x"));
    assert_eq!(rest, vec![Token::Colon]);
}

#[test]
fn parsing_stops_before_a_comma() {
    let (expr, rest) = parse_partial("x, y");
    assert_eq!(expr, var("x"));
    assert_eq!(rest, vec![Token::Comma, Token::Identifier("y".into())]);
}

// --- Associativity of the binary levels ---

#[test]
fn chained_same_precedence_operators_are_left_associative() {
    assert_eq!(
        parse("1 + 2 + 3"),
        binary(binary(int(1), Token::Plus, int(2)), Token::Plus, int(3))
    );
}

#[test]
fn long_chains_stay_left_associative() {
    assert_eq!(
        parse("1 + 2 + 3 + 4"),
        binary(
            binary(binary(int(1), Token::Plus, int(2)), Token::Plus, int(3)),
            Token::Plus,
            int(4)
        )
    );
}

#[test]
fn every_binary_level_is_left_associative() {
    for (source, operator) in [
        ("1 * 2 * 3", Token::Asterisk),
        ("1 / 2 / 3", Token::Slash),
        ("1 - 2 - 3", Token::Minus),
    ] {
        assert_eq!(
            parse(source),
            binary(binary(int(1), operator.clone(), int(2)), operator, int(3)),
            "for {source:?}"
        );
    }
    for (source, operator) in [
        ("a < b < c", Token::Less),
        ("a == b == c", Token::EqualEqual),
        ("a != b != c", Token::BangEqual),
        ("a and b and c", Token::And),
        ("a or b or c", Token::Or),
    ] {
        assert_eq!(
            parse(source),
            binary(
                binary(var("a"), operator.clone(), var("b")),
                operator,
                var("c")
            ),
            "for {source:?}"
        );
    }
}

#[test]
fn subtraction_groups_to_the_left() {
    // `10 - 2 - 3` must be (10 - 2) - 3 = 5, not 10 - (2 - 3) = 11.
    assert_eq!(
        parse("10 - 2 - 3"),
        binary(binary(int(10), Token::Minus, int(2)), Token::Minus, int(3))
    );
}

#[test]
fn mixed_precedence_in_a_long_chain() {
    // 1 + 2 * 3 - 4  =>  (1 + (2 * 3)) - 4
    assert_eq!(
        parse("1 + 2 * 3 - 4"),
        binary(
            binary(int(1), Token::Plus, binary(int(2), Token::Asterisk, int(3))),
            Token::Minus,
            int(4)
        )
    );
}

#[test]
fn a_chain_of_comparisons_and_logic() {
    // a < b and c < d or e  =>  ((a<b) and (c<d)) or e
    assert_eq!(
        parse("a < b and c < d or e"),
        binary(
            binary(
                binary(var("a"), Token::Less, var("b")),
                Token::And,
                binary(var("c"), Token::Less, var("d"))
            ),
            Token::Or,
            var("e")
        )
    );
}

#[test]
fn an_unclosed_call_at_end_of_input_is_an_error() {
    // The argument loop must stop when the cursor runs out instead of spinning.
    let tokens = vec![Token::Identifier("f".into()), Token::LeftParen];
    assert_eq!(
        parse_tokens(tokens),
        Err("Expected ')' after function arguments".to_string())
    );
}

#[test]
fn an_unclosed_call_with_arguments_is_an_error() {
    assert_eq!(error("f(1, 2"), "Expected ')' after function arguments");
}
