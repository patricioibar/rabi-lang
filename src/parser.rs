use crate::{statement::Statement, token::Token};

pub fn parse(tokens: Vec<Token>) -> Result<Vec<Statement>, String> {
    let mut cursor = tokens.into_iter().peekable();
    let mut statements = Vec::new();
    loop {
        Statement::skip_blank_lines(&mut cursor);
        match cursor.peek() {
            Some(Token::Eof) | None => return Ok(statements),
            _ => statements.push(Statement::get_next(&mut cursor)?),
        }
    }
}
