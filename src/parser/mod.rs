use crate::{statement::Statement, token::Token};

mod cursor;
mod expressions;
mod statements;

#[cfg(test)]
mod statements_test;

#[cfg(test)]
mod expressions_test;

use cursor::Cursor;

/// Parses a token stream from [`crate::scanner::scan`] into statements.
///
/// The stream's invariants are the Scanner's to keep; this module relies on
/// them, in particular that indentation arrives as `Indent`/`Dedent` and that
/// the stream ends with `Eof`.
pub fn parse(tokens: Vec<Token>) -> Result<Vec<Statement>, String> {
    let mut cursor = Cursor::new(tokens);
    let mut statements = Vec::new();
    loop {
        cursor.skip_blank_lines();
        if cursor.at_end() {
            return Ok(statements);
        }
        statements.push(statements::statement(&mut cursor)?);
    }
}
