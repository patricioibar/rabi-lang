use std::io::Cursor;

use crate::{expression::Expression, token::Token};

pub fn parse(tokens: Vec<Token>) -> Result<Vec<Expression>, String> {
    let mut cursor = Cursor::new(tokens).into_inner().into_iter().peekable();
    let mut expressions = Vec::new();
    while cursor.peek().is_some() {
        let expression = Expression::get_next(&mut cursor)?;
        expressions.push(expression);
    }
    Ok(expressions)
}
