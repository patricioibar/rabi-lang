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
    Null,

    // Operators
    Plus,
    Minus,
    Asterisk,
    Slash,
    Percent,
    And,
    Or,
    Equal,
    Bang,
    BangEqual,
    EqualEqual,
    Less,
    LessEqual,
    Greater,
    GreaterEqual,

    // Punctuation
    LeftParen,
    RightParen,
    LeftBracket,
    RightBracket,
    Colon,
    Comma,

    // Keywords
    If,
    Else,
    While,
    For,
    In,
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

impl Token {
    pub fn is_literal(&self) -> bool {
        matches!(
            self,
            Token::Integer(_)
                | Token::Decimal(_)
                | Token::StringLiteral(_)
                | Token::True
                | Token::False
                | Token::Null
        )
    }
}

impl std::fmt::Display for Token {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Token::Identifier(s) => write!(f, "identifier '{}'", s),
            Token::Integer(i) => write!(f, "integer '{}'", i),
            Token::Decimal(d) => write!(f, "decimal '{}'", d),
            Token::StringLiteral(s) => write!(f, "string literal '{}'", s),
            Token::True => write!(f, "'true'"),
            Token::False => write!(f, "'false'"),
            Token::LeftParen => write!(f, "(",),
            Token::RightParen => write!(f, ")",),
            Token::LeftBracket => write!(f, "[",),
            Token::RightBracket => write!(f, "]",),
            Token::Colon => write!(f, ":",),
            Token::Comma => write!(f, ",",),
            Token::Equal => write!(f, "=",),
            Token::Else => write!(f, "else",),
            Token::NewLine => write!(f, "a newline"),
            Token::Indent => write!(f, "an indented block"),
            Token::Dedent => write!(f, "a dedent"),
            Token::Eof => write!(f, "end of input"),
            Token::Plus => write!(f, "+"),
            Token::Minus => write!(f, "-"),
            Token::Asterisk => write!(f, "*"),
            Token::Slash => write!(f, "/"),
            Token::Percent => write!(f, "%"),
            Token::And => write!(f, "and"),
            Token::Or => write!(f, "or"),
            Token::Bang => write!(f, "!"),
            Token::BangEqual => write!(f, "!="),
            Token::EqualEqual => write!(f, "=="),
            Token::Less => write!(f, "<"),
            Token::LessEqual => write!(f, "<="),
            Token::Greater => write!(f, ">"),
            Token::GreaterEqual => write!(f, ">="),
            Token::If => write!(f, "if"),
            Token::While => write!(f, "while"),
            Token::For => write!(f, "for"),
            Token::In => write!(f, "in"),
            Token::Break => write!(f, "break"),
            Token::Continue => write!(f, "continue"),
            Token::Function => write!(f, "function"),
            Token::Return => write!(f, "return"),
            Token::Let => write!(f, "let"),
            Token::Null => write!(f, "null"),
        }
    }
}
