use std::{
    fs::File,
    io::{BufRead, BufReader},
};

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

pub fn scan_file(file_reader: BufReader<File>) -> Result<Vec<Token>, String> {
    let mut tokens = Vec::new();
    let mut last_indentation_level = 0;
    for line in file_reader.lines() {
        let line = line.map_err(|e| format!("Error reading line from file: {}", e))?;
        let mut line_tokens = scan_line(line)?;

        let this_indentation_level = line_tokens.iter().take_while(|t| **t == Token::Tab).count();
        line_tokens.drain(0..this_indentation_level); // Remove the indentation tokens

        // A line with no tokens of its own (blank, or only a comment) carries no
        // indentation information, so it must not open or close a block.
        if line_tokens == [Token::NewLine] {
            tokens.append(&mut line_tokens);
            continue;
        }

        while this_indentation_level > last_indentation_level {
            tokens.push(Token::Indent);
            last_indentation_level += 1;
        }
        while this_indentation_level < last_indentation_level {
            tokens.push(Token::Dedent);
            last_indentation_level -= 1;
        }

        tokens.append(&mut line_tokens);
    }

    while last_indentation_level > 0 {
        tokens.push(Token::Dedent);
        last_indentation_level -= 1;
    }

    tokens.push(Token::Eof);

    Ok(tokens)
}
