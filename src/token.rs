#[derive(Debug)]
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
    Function,
    Return,
    Let,

    // Special Tokens
    Eof,
    NewLine,
}

type Cursor = std::iter::Peekable<std::str::Chars<'static>>;

impl Token {
    pub fn get_next(cursor: &mut Cursor) -> Result<Option<Self>, String> {
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
                        '=' => Token::Equal,
                        '(' => Token::LeftParen,
                        ')' => Token::RightParen,
                        ':' => Token::Colon,
                        '\t' => Token::Tab,
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
                            while let Some(&next) = cursor.peek() {
                                if next == '"' {
                                    cursor.next(); // Consume the closing quote
                                    break;
                                } else {
                                    string_literal.push(next);
                                    cursor.next();
                                }
                            }
                            Token::StringLiteral(string_literal)
                        }
                        c if c.is_ascii_alphabetic() || c == '_' => {
                            let mut word = String::from(c);
                            while let Some(&next) = cursor.peek() {
                                if next.is_ascii_alphanumeric() || next == '_' {
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
                        _ => return Ok(None), // Unrecognized character
                    };
                    return Ok(Some(token));
                }
                None => return Ok(None), // End of input
            }
        }
    }
}
