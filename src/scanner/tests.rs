//! Tests for the Scanner's char-level implementation. These reach an internal
//! seam: neither `tokenize` nor `split_indentation` is part of the Scanner's
//! interface.

use crate::{
    scanner::{
        split_indentation,
        tokenizer::{next_token, tokenize},
    },
    token::Token,
};

fn one(source: &str) -> Token {
    let tokens = tokenize(source).expect("should tokenize");
    assert_eq!(
        tokens.len(),
        1,
        "expected exactly one token from {source:?}"
    );
    tokens.into_iter().next().unwrap()
}

// --- Empty and whitespace-only input ---

#[test]
fn empty_input_yields_no_tokens() {
    assert_eq!(tokenize("").unwrap(), vec![]);
}

#[test]
fn spaces_and_newlines_are_skipped() {
    assert_eq!(tokenize("   \n \r\n   ").unwrap(), vec![]);
}

#[test]
fn tabs_are_whitespace_to_the_tokenizer() {
    // Indentation is the Scanner's concern,
    // it transforms leading whitespaces into indent and dedent tokens
    assert_eq!(tokenize("\t\t").unwrap(), vec![]);
}

#[test]
fn spaces_between_tokens_do_not_produce_tokens() {
    assert_eq!(
        tokenize("  1   +   2  ").unwrap(),
        vec![Token::Integer(1), Token::Plus, Token::Integer(2)]
    );
}

// --- Single-character operators and punctuation ---

#[test]
fn single_character_operators() {
    assert_eq!(one("+"), Token::Plus);
    assert_eq!(one("-"), Token::Minus);
    assert_eq!(one("*"), Token::Asterisk);
    assert_eq!(one("/"), Token::Slash);
    assert_eq!(one("="), Token::Equal);
    assert_eq!(one("<"), Token::Less);
    assert_eq!(one(">"), Token::Greater);
    assert_eq!(one("!"), Token::Bang);
}

#[test]
fn punctuation() {
    assert_eq!(one("("), Token::LeftParen);
    assert_eq!(one(")"), Token::RightParen);
    assert_eq!(one("["), Token::LeftBracket);
    assert_eq!(one("]"), Token::RightBracket);
    assert_eq!(one(":"), Token::Colon);
    assert_eq!(one(","), Token::Comma);
}

#[test]
fn all_punctuation_in_sequence() {
    assert_eq!(
        tokenize("()[],:").unwrap(),
        vec![
            Token::LeftParen,
            Token::RightParen,
            Token::LeftBracket,
            Token::RightBracket,
            Token::Comma,
            Token::Colon
        ]
    );
}

// --- Two-character operators ---

#[test]
fn two_character_comparison_operators() {
    assert_eq!(one("<="), Token::LessEqual);
    assert_eq!(one(">="), Token::GreaterEqual);
    assert_eq!(one("!="), Token::BangEqual);
    assert_eq!(one("=="), Token::EqualEqual);
}

#[test]
fn comparison_operators_do_not_over_consume() {
    assert_eq!(
        tokenize("<1").unwrap(),
        vec![Token::Less, Token::Integer(1)]
    );
    assert_eq!(
        tokenize(">1").unwrap(),
        vec![Token::Greater, Token::Integer(1)]
    );
    assert_eq!(
        tokenize("!x").unwrap(),
        vec![Token::Bang, Token::Identifier("x".into())]
    );
}

#[test]
fn equals_does_not_over_consume() {
    assert_eq!(
        tokenize("=1").unwrap(),
        vec![Token::Equal, Token::Integer(1)]
    );
    assert_eq!(
        tokenize("=x").unwrap(),
        vec![Token::Equal, Token::Identifier("x".into())]
    );
}

#[test]
fn three_equals_is_equal_equal_then_equal() {
    assert_eq!(
        tokenize("===").unwrap(),
        vec![Token::EqualEqual, Token::Equal]
    );
}

#[test]
fn separated_equals_are_two_equal_tokens() {
    assert_eq!(tokenize("= =").unwrap(), vec![Token::Equal, Token::Equal]);
}

#[test]
fn assignment_and_equality_are_distinct() {
    assert_eq!(
        tokenize("x = y == z").unwrap(),
        vec![
            Token::Identifier("x".into()),
            Token::Equal,
            Token::Identifier("y".into()),
            Token::EqualEqual,
            Token::Identifier("z".into())
        ]
    );
}

#[test]
fn maximal_munch_at_end_of_input() {
    // `peek()` returning None must fall back to the one-character token.
    assert_eq!(tokenize("=").unwrap(), vec![Token::Equal]);
    assert_eq!(tokenize("<").unwrap(), vec![Token::Less]);
    assert_eq!(tokenize(">").unwrap(), vec![Token::Greater]);
    assert_eq!(tokenize("!").unwrap(), vec![Token::Bang]);
}

// --- Integers ---

#[test]
fn integers() {
    assert_eq!(one("0"), Token::Integer(0));
    assert_eq!(one("7"), Token::Integer(7));
    assert_eq!(one("1234567890"), Token::Integer(1234567890));
}

#[test]
fn integer_with_leading_zeros() {
    assert_eq!(one("007"), Token::Integer(7));
}

#[test]
fn integer_max_value() {
    assert_eq!(one("9223372036854775807"), Token::Integer(i64::MAX));
}

#[test]
fn integer_overflow_is_an_error() {
    let err = tokenize("9223372036854775808").unwrap_err();
    assert!(err.starts_with("Error parsing integer"), "got {err:?}");
}

#[test]
fn minus_is_not_part_of_a_numeric_literal() {
    // Negation is left to the parser.
    assert_eq!(
        tokenize("-5").unwrap(),
        vec![Token::Minus, Token::Integer(5)]
    );
}

#[test]
fn integer_followed_by_operator() {
    assert_eq!(
        tokenize("10+20").unwrap(),
        vec![Token::Integer(10), Token::Plus, Token::Integer(20)]
    );
}

// --- Decimals ---

#[test]
fn decimals() {
    assert_eq!(one("1.5"), Token::Decimal(1.5));
    assert_eq!(one("0.25"), Token::Decimal(0.25));
    assert_eq!(one("3.0"), Token::Decimal(3.0));
}

#[test]
fn decimal_with_trailing_dot() {
    assert_eq!(one("2."), Token::Decimal(2.0));
}

#[test]
fn decimal_with_many_digits() {
    assert_eq!(one("123.456"), Token::Decimal(123.456));
}

#[test]
fn number_with_two_dots_is_an_error() {
    let err = tokenize("1.2.3").unwrap_err();
    assert!(err.starts_with("Error parsing decimal"), "got {err:?}");
}

#[test]
fn leading_dot_is_not_a_decimal() {
    // '.' is not a valid token on its own.
    let err = tokenize(".5").unwrap_err();
    assert_eq!(err, "Unrecognized character: '.'");
}

// --- String literals ---

#[test]
fn empty_string_literal() {
    assert_eq!(one("\"\""), Token::StringLiteral(String::new()));
}

#[test]
fn simple_string_literal() {
    assert_eq!(one("\"hello\""), Token::StringLiteral("hello".into()));
}

#[test]
fn string_literal_preserves_inner_whitespace_and_symbols() {
    assert_eq!(
        one("\"  a, b + c: 1  \""),
        Token::StringLiteral("  a, b + c: 1  ".into())
    );
}

#[test]
fn string_literal_preserves_keywords_verbatim() {
    assert_eq!(
        one("\"if while return\""),
        Token::StringLiteral("if while return".into())
    );
}

#[test]
fn string_literal_preserves_comment_marker() {
    assert_eq!(
        one("\"# not a comment\""),
        Token::StringLiteral("# not a comment".into())
    );
}

#[test]
fn string_literal_preserves_characters_that_are_otherwise_illegal() {
    assert_eq!(one("\"@ $ % &\""), Token::StringLiteral("@ $ % &".into()));
}

#[test]
fn string_literal_accepts_non_ascii() {
    assert_eq!(
        one("\"héllo ñ 日本\""),
        Token::StringLiteral("héllo ñ 日本".into())
    );
}

#[test]
fn two_adjacent_string_literals() {
    assert_eq!(
        tokenize("\"a\"\"b\"").unwrap(),
        vec![
            Token::StringLiteral("a".into()),
            Token::StringLiteral("b".into())
        ]
    );
}

#[test]
fn string_literal_followed_by_operator() {
    assert_eq!(
        tokenize("\"a\" + \"b\"").unwrap(),
        vec![
            Token::StringLiteral("a".into()),
            Token::Plus,
            Token::StringLiteral("b".into())
        ]
    );
}

#[test]
fn no_escape_sequences_are_interpreted() {
    // A backslash is an ordinary character inside a literal.
    assert_eq!(one("\"a\\nb\""), Token::StringLiteral("a\\nb".into()));
}

#[test]
fn backslash_does_not_escape_the_closing_quote() {
    assert_eq!(
        tokenize("\"a\\\"").unwrap(),
        vec![Token::StringLiteral("a\\".into())]
    );
}

#[test]
fn unterminated_string_literal_is_an_error() {
    let err = tokenize("\"abc").unwrap_err();
    assert_eq!(err, "Unterminated string literal: \"abc");
}

#[test]
fn a_lone_quote_is_an_unterminated_string_literal() {
    let err = tokenize("\"").unwrap_err();
    assert_eq!(err, "Unterminated string literal: \"");
}

#[test]
fn an_odd_number_of_quotes_is_an_error() {
    let err = tokenize("\"a\" \"b\" \"c").unwrap_err();
    assert_eq!(err, "Unterminated string literal: \"c");
}

#[test]
fn an_unterminated_string_literal_after_valid_tokens_is_an_error() {
    let err = tokenize("let x = \"oops").unwrap_err();
    assert_eq!(err, "Unterminated string literal: \"oops");
}

#[test]
fn a_terminated_literal_before_an_unterminated_one_is_still_returned() {
    let mut cursor = "\"ok\" \"bad".chars().peekable();
    assert_eq!(
        next_token(&mut cursor),
        Ok(Some(Token::StringLiteral("ok".into())))
    );
    assert_eq!(
        next_token(&mut cursor),
        Err("Unterminated string literal: \"bad".to_string())
    );
}

// --- Identifiers ---

#[test]
fn identifiers() {
    assert_eq!(one("x"), Token::Identifier("x".into()));
    assert_eq!(one("foo"), Token::Identifier("foo".into()));
    assert_eq!(one("FooBar"), Token::Identifier("FooBar".into()));
}

#[test]
fn identifiers_may_start_with_underscore() {
    assert_eq!(one("_"), Token::Identifier("_".into()));
    assert_eq!(one("_private"), Token::Identifier("_private".into()));
}

#[test]
fn identifiers_may_contain_digits_and_underscores() {
    assert_eq!(one("a1_b2"), Token::Identifier("a1_b2".into()));
    assert_eq!(one("x2"), Token::Identifier("x2".into()));
}

#[test]
fn identifiers_may_not_start_with_a_digit() {
    assert_eq!(
        tokenize("1abc").unwrap(),
        vec![Token::Integer(1), Token::Identifier("abc".into())]
    );
}

#[test]
fn keyword_prefixes_and_extensions_are_identifiers() {
    assert_eq!(one("i"), Token::Identifier("i".into()));
    assert_eq!(one("iff"), Token::Identifier("iff".into()));
    assert_eq!(one("ifx"), Token::Identifier("ifx".into()));
    assert_eq!(one("returned"), Token::Identifier("returned".into()));
    assert_eq!(one("_if"), Token::Identifier("_if".into()));
}

#[test]
fn keywords_are_case_sensitive() {
    assert_eq!(one("If"), Token::Identifier("If".into()));
    assert_eq!(one("TRUE"), Token::Identifier("TRUE".into()));
}

#[test]
fn identifier_terminates_at_punctuation() {
    assert_eq!(
        tokenize("foo(bar)").unwrap(),
        vec![
            Token::Identifier("foo".into()),
            Token::LeftParen,
            Token::Identifier("bar".into()),
            Token::RightParen
        ]
    );
}

// --- Non-ASCII identifiers ---

#[test]
fn non_ascii_letters_may_start_an_identifier() {
    assert_eq!(one("ñ"), Token::Identifier("ñ".into()));
    assert_eq!(one("año"), Token::Identifier("año".into()));
    assert_eq!(one("café"), Token::Identifier("café".into()));
    assert_eq!(one("über_wert"), Token::Identifier("über_wert".into()));
    assert_eq!(one("日本語"), Token::Identifier("日本語".into()));
    assert_eq!(one("Δx"), Token::Identifier("Δx".into()));
}

#[test]
fn non_ascii_identifiers_are_delimited_like_ascii_ones() {
    assert_eq!(
        tokenize("año + 1").unwrap(),
        vec![
            Token::Identifier("año".into()),
            Token::Plus,
            Token::Integer(1)
        ]
    );
    assert_eq!(
        tokenize("café(té)").unwrap(),
        vec![
            Token::Identifier("café".into()),
            Token::LeftParen,
            Token::Identifier("té".into()),
            Token::RightParen
        ]
    );
}

#[test]
fn non_ascii_letters_do_not_shadow_keywords() {
    // Keyword matching is still exact, so a decorated keyword is a name.
    assert_eq!(one("ifé"), Token::Identifier("ifé".into()));
    assert_eq!(one("éif"), Token::Identifier("éif".into()));
}

#[test]
fn non_ascii_digits_are_not_numeric_literals() {
    // `is_ascii_digit` keeps these out of the number branch, so they get the
    // clear "unrecognized" error instead of a raw parse failure.
    for source in ["²", "٣", "½", "１"] {
        let mut cursor = source.chars().peekable();
        assert_eq!(
            next_token(&mut cursor),
            Err(format!("Unrecognized character: '{source}'")),
            "expected {source:?} to be rejected"
        );
    }
}

#[test]
fn numeric_letters_are_identifiers_not_numbers() {
    // Roman numerals are alphabetic in Unicode, so the identifier branch
    // claims them before the number branch is reached.
    assert_eq!(one("ⅰ"), Token::Identifier("ⅰ".into()));
}

#[test]
fn a_non_ascii_digit_after_a_number_ends_the_literal() {
    let mut cursor = "1²".chars().peekable();
    assert_eq!(next_token(&mut cursor), Ok(Some(Token::Integer(1))));
    assert_eq!(
        next_token(&mut cursor),
        Err("Unrecognized character: '²'".to_string())
    );
}

#[test]
fn only_ascii_digits_form_numbers() {
    // Every literal must parse, so no number can reach `str::parse` with a
    // digit `str::parse` cannot handle.
    assert_eq!(one("0123456789"), Token::Integer(123456789));
}

#[test]
fn symbols_and_emoji_are_still_unrecognized() {
    // These are not alphabetic, so they remain errors.
    for source in ["🎉", "§", "€"] {
        let mut cursor = source.chars().peekable();
        assert_eq!(
            next_token(&mut cursor),
            Err(format!("Unrecognized character: '{source}'")),
            "expected {source:?} to be rejected"
        );
    }
}

// --- Keywords ---

#[test]
fn all_keywords() {
    assert_eq!(one("and"), Token::And);
    assert_eq!(one("or"), Token::Or);
    assert_eq!(one("true"), Token::True);
    assert_eq!(one("false"), Token::False);
    assert_eq!(one("if"), Token::If);
    assert_eq!(one("else"), Token::Else);
    assert_eq!(one("while"), Token::While);
    assert_eq!(one("for"), Token::For);
    assert_eq!(one("break"), Token::Break);
    assert_eq!(one("continue"), Token::Continue);
    assert_eq!(one("func"), Token::Function);
    assert_eq!(one("return"), Token::Return);
    assert_eq!(one("let"), Token::Let);
}

#[test]
fn keywords_in_sequence() {
    assert_eq!(
        tokenize("if true and false else").unwrap(),
        vec![
            Token::If,
            Token::True,
            Token::And,
            Token::False,
            Token::Else
        ]
    );
}

// --- Comments ---

#[test]
fn a_whole_line_comment_yields_no_tokens() {
    assert_eq!(tokenize("# just a comment").unwrap(), vec![]);
}

#[test]
fn bare_hash_yields_no_tokens() {
    assert_eq!(tokenize("#").unwrap(), vec![]);
}

#[test]
fn a_trailing_comment_is_stripped() {
    assert_eq!(
        tokenize("let x = 1 # assign one").unwrap(),
        vec![
            Token::Let,
            Token::Identifier("x".into()),
            Token::Equal,
            Token::Integer(1)
        ]
    );
}

#[test]
fn a_comment_stops_at_the_newline() {
    assert_eq!(tokenize("# comment\n1").unwrap(), vec![Token::Integer(1)]);
}

#[test]
fn a_comment_does_not_need_a_space_after_the_hash() {
    assert_eq!(
        tokenize("1 #x\n2").unwrap(),
        vec![Token::Integer(1), Token::Integer(2)]
    );
}

#[test]
fn illegal_characters_inside_a_comment_are_ignored() {
    assert_eq!(
        tokenize("1 # @ $ % & 'x'").unwrap(),
        vec![Token::Integer(1)]
    );
}

// --- Unrecognized characters ---

#[test]
fn an_unrecognized_character_is_an_error() {
    let err = tokenize("1 @ 2").unwrap_err();
    assert_eq!(err, "Unrecognized character: '@'");
}

#[test]
fn various_unrecognized_characters() {
    for source in [
        "@", "$", "&", "|", "^", "~", ";", "{", "}", "?", ".", "'", "\\",
    ] {
        let mut cursor = source.chars().peekable();
        assert_eq!(
            next_token(&mut cursor),
            Err(format!("Unrecognized character: '{source}'")),
            "expected {source:?} to be rejected"
        );
    }
}

#[test]
fn an_unrecognized_character_is_reported_only_once_reached() {
    // Tokens before the offending character are still returned normally.
    let mut cursor = "1 + @".chars().peekable();
    assert_eq!(next_token(&mut cursor), Ok(Some(Token::Integer(1))));
    assert_eq!(next_token(&mut cursor), Ok(Some(Token::Plus)));
    assert_eq!(
        next_token(&mut cursor),
        Err("Unrecognized character: '@'".to_string())
    );
}

#[test]
fn ok_none_means_end_of_input_only() {
    for source in ["", "   ", "# comment"] {
        let mut cursor = source.chars().peekable();
        assert_eq!(next_token(&mut cursor), Ok(None), "for {source:?}");
    }
}

// --- Cursor behaviour ---

#[test]
fn get_next_consumes_exactly_one_token_at_a_time() {
    let mut cursor = "let x".chars().peekable();
    assert_eq!(next_token(&mut cursor), Ok(Some(Token::Let)));
    assert_eq!(cursor.clone().collect::<String>(), " x");
    assert_eq!(
        next_token(&mut cursor),
        Ok(Some(Token::Identifier("x".into())))
    );
    assert_eq!(cursor.next(), None);
}

#[test]
fn get_next_keeps_returning_none_after_the_end() {
    let mut cursor = "1".chars().peekable();
    assert_eq!(next_token(&mut cursor), Ok(Some(Token::Integer(1))));
    assert_eq!(next_token(&mut cursor), Ok(None));
    assert_eq!(next_token(&mut cursor), Ok(None));
}

// --- Realistic program fragments ---

#[test]
fn a_full_statement() {
    assert_eq!(
        tokenize("func max(a, b):").unwrap(),
        vec![
            Token::Function,
            Token::Identifier("max".into()),
            Token::LeftParen,
            Token::Identifier("a".into()),
            Token::Comma,
            Token::Identifier("b".into()),
            Token::RightParen,
            Token::Colon
        ]
    );
}

#[test]
fn a_full_condition() {
    assert_eq!(
        tokenize("while i <= 10 and x != 3.5:").unwrap(),
        vec![
            Token::While,
            Token::Identifier("i".into()),
            Token::LessEqual,
            Token::Integer(10),
            Token::And,
            Token::Identifier("x".into()),
            Token::BangEqual,
            Token::Decimal(3.5),
            Token::Colon
        ]
    );
}

// --- is_literal ---

#[test]
fn literal_tokens_are_literals() {
    for token in [
        Token::Integer(1),
        Token::Decimal(1.0),
        Token::StringLiteral("s".into()),
        Token::True,
        Token::False,
    ] {
        assert!(token.is_literal(), "{token:?} should be a literal");
    }
}

#[test]
fn non_literal_tokens_are_not_literals() {
    for token in [
        Token::Identifier("x".into()),
        Token::Plus,
        Token::Minus,
        Token::And,
        Token::Or,
        Token::Equal,
        Token::EqualEqual,
        Token::BangEqual,
        Token::Bang,
        Token::LeftParen,
        Token::Colon,
        Token::If,
        Token::Let,
        Token::Return,
        Token::Eof,
        Token::NewLine,
        Token::Indent,
        Token::Dedent,
    ] {
        assert!(!token.is_literal(), "{token:?} should not be a literal");
    }
}

// --- Derived traits ---

#[test]
fn equality_compares_payloads() {
    assert_eq!(Token::Identifier("a".into()), Token::Identifier("a".into()));
    assert_ne!(Token::Identifier("a".into()), Token::Identifier("b".into()));
    assert_ne!(Token::Integer(1), Token::Decimal(1.0));
    assert_ne!(Token::True, Token::False);
}

// --- Indentation ---

#[test]
fn one_level_is_a_tab_or_four_spaces() {
    for indent in [
        "",
        "\t",
        "    ",
        "\t\t",
        "        ",
        "\t\t\t",
        "            ",
    ] {
        let expected = indent.len() / if indent.starts_with('\t') { 1 } else { 4 };
        let line = format!("{indent}let x = 1");
        let (depth, rest) = split_indentation(&line).unwrap();
        assert_eq!(depth, expected, "depth of {indent:?}");
        assert_eq!(rest, "let x = 1", "rest after {indent:?}");
    }
}

#[test]
fn tabs_and_spaces_give_the_same_depth() {
    for (tabs, spaces) in [("\tx", "    x"), ("\t\tx", "        x")] {
        assert_eq!(
            split_indentation(tabs).unwrap(),
            split_indentation(spaces).unwrap()
        );
    }
}

#[test]
fn mixing_tabs_and_spaces_in_one_indent_is_rejected() {
    for line in [" \tlet x = 1", "\t let x = 1"] {
        let error = split_indentation(line).expect_err("should not split");
        assert!(error.contains("mixes tabs and spaces"), "{line:?}: {error}");
    }
}

#[test]
fn spaces_that_do_not_fill_a_level_are_rejected() {
    for (line, spaces) in [("  x", 2), ("      x", 6), ("         x", 9)] {
        let error = split_indentation(line).expect_err("should not split");
        assert!(
            error.contains(&format!("{spaces} spaces is not a multiple of 4")),
            "{line:?}: {error}"
        );
    }
}
