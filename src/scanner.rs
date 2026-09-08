use crate::token::Token;

pub fn scan_line(line: String) -> Result<Vec<Token>, String> {
    let mut tokens = Vec::new();
    let static_line: &'static str = Box::leak(line.into_boxed_str());
    let mut cursor = static_line.chars().peekable();

    while let Some(token) = Token::get_next(&mut cursor)? {
        tokens.push(token);
    }
    tokens.push(Token::NewLine);
    Ok(tokens)
}
