use std::matches;

#[derive(Debug, Clone, PartialEq)]
pub enum Token {
    // Literals
    Identifier(String),
    Integer(i64),
    Decimal(f64),
    StringLiteral(String),
    True,
    False,

    // Operators
    Plus,
    Minus,
    Asterisk,
    Slash,
    And,
    Or,
    Equal,
    Bang,
    BangEqual,
    Less,
    LessEqual,
    Greater,
    GreaterEqual,

    // Punctuation
    LeftParen,
    RightParen,
    Colon,
    Comma,
    Tab,

    // Keywords
    If,
    Else,
    While,
    For,
    Break,
    Continue,
    Function,
    Return,
    Let,

    // Special Tokens
    Eof,
    NewLine,
    Indent,
    Dedent,
}

type Cursor = std::iter::Peekable<std::str::Chars<'static>>;

impl Token {
    pub fn get_next(cursor: &mut Cursor) -> Result<Option<Self>, String> {
        loop {
            match cursor.next() {
                Some(c) => {
                    let token = match c {
                        '\t' => Token::Tab,
                        c if c.is_whitespace() => continue,
                        ',' => Token::Comma,
                        '+' => Token::Plus,
                        '-' => Token::Minus,
                        '*' => Token::Asterisk,
                        '/' => Token::Slash,
                        '=' => Token::Equal,
                        '(' => Token::LeftParen,
                        ')' => Token::RightParen,
                        ':' => Token::Colon,
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
                            while let Some(next) = cursor.next() {
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
                        c if c.is_numeric() => {
                            let mut number = String::from(c);
                            while let Some(&next) = cursor.peek() {
                                if next.is_numeric() || next == '.' {
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
                            continue; // Skip to the next iteration to get the next token
                        }
                        _ => {
                            return Err(format!("Unrecognized character: '{}'", c));
                        }
                    };
                    return Ok(Some(token));
                }
                None => return Ok(None), // End of input
            }
        }
    }

    pub fn is_literal(&self) -> bool {
        matches!(
            self,
            Token::Integer(_)
                | Token::Decimal(_)
                | Token::StringLiteral(_)
                | Token::True
                | Token::False
        )
    }
}
