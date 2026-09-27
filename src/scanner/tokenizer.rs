use crate::token::Token;

type Cursor<'a> = std::iter::Peekable<std::str::Chars<'a>>;

pub(super) fn tokenize(line: &str) -> Result<Vec<Token>, String> {
    let mut cursor = line.chars().peekable();
    let mut tokens = Vec::new();
    while let Some(token) = next_token(&mut cursor)? {
        tokens.push(token);
    }
    Ok(tokens)
}

pub(super) fn next_token(cursor: &mut Cursor) -> Result<Option<Token>, String> {
    loop {
        match cursor.next() {
            Some(c) => {
                let token = match c {
                    c if c.is_whitespace() => continue,
                    ',' => Token::Comma,
                    '+' => Token::Plus,
                    '-' => Token::Minus,
                    '*' => Token::Asterisk,
                    '/' => Token::Slash,
                    '%' => Token::Percent,
                    '(' => Token::LeftParen,
                    ')' => Token::RightParen,
                    ':' => Token::Colon,
                    '=' => match cursor.peek() {
                        Some('=') => {
                            cursor.next();
                            Token::EqualEqual
                        }
                        _ => Token::Equal,
                    },
                    '<' => match cursor.peek() {
                        Some('=') => {
                            cursor.next();
                            Token::LessEqual
                        }
                        _ => Token::Less,
                    },
                    '>' => match cursor.peek() {
                        Some('=') => {
                            cursor.next();
                            Token::GreaterEqual
                        }
                        _ => Token::Greater,
                    },
                    '!' => match cursor.peek() {
                        Some('=') => {
                            cursor.next();
                            Token::BangEqual
                        }
                        _ => Token::Bang,
                    },
                    '"' => {
                        let mut string_literal = String::new();
                        let mut terminated = false;
                        for next in cursor.by_ref() {
                            if next == '"' {
                                terminated = true;
                                break;
                            }
                            string_literal.push(next);
                        }
                        if !terminated {
                            return Err(format!(
                                "Unterminated string literal: \"{}",
                                string_literal
                            ));
                        }
                        Token::StringLiteral(string_literal)
                    }
                    c if c.is_alphabetic() || c == '_' => {
                        let mut word = String::from(c);
                        while let Some(&next) = cursor.peek() {
                            if next.is_alphanumeric() || next == '_' {
                                word.push(next);
                                cursor.next();
                            } else {
                                break;
                            }
                        }
                        let token = match word.as_str() {
                            "and" => Token::And,
                            "or" => Token::Or,
                            "true" => Token::True,
                            "false" => Token::False,
                            "null" => Token::Null,
                            "if" => Token::If,
                            "else" => Token::Else,
                            "while" => Token::While,
                            "for" => Token::For,
                            "break" => Token::Break,
                            "continue" => Token::Continue,
                            "func" => Token::Function,
                            "return" => Token::Return,
                            "let" => Token::Let,
                            _ => Token::Identifier(word),
                        };
                        return Ok(Some(token));
                    }
                    c if c.is_ascii_digit() => {
                        let mut number = String::from(c);
                        while let Some(&next) = cursor.peek() {
                            if next.is_ascii_digit() || next == '.' {
                                number.push(next);
                                cursor.next();
                            } else {
                                break;
                            }
                        }
                        let token = if number.contains('.') {
                            Token::Decimal(
                                number
                                    .parse()
                                    .map_err(|e| format!("Error parsing decimal: {}", e))?,
                            )
                        } else {
                            Token::Integer(
                                number
                                    .parse()
                                    .map_err(|e| format!("Error parsing integer: {}", e))?,
                            )
                        };
                        return Ok(Some(token));
                    }
                    '#' => {
                        // Skip comments until the end of the line
                        while let Some(&next) = cursor.peek() {
                            if next == '\n' {
                                break;
                            }
                            cursor.next();
                        }
                        continue;
                    }
                    _ => {
                        return Err(format!("Unrecognized character: '{}'", c));
                    }
                };
                return Ok(Some(token));
            }
            None => return Ok(None),
        }
    }
}
