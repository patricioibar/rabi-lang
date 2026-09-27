use std::io::BufRead;

use crate::token::Token;

mod tokenizer;

#[cfg(test)]
mod tests;

pub fn scan(source: impl BufRead) -> Result<Vec<Token>, String> {
    let mut tokens = Vec::new();
    let mut open_blocks = 0;

    for line in source.lines() {
        let line = line.map_err(|e| format!("Error reading line from source: {}", e))?;
        let (depth, rest) = split_indentation(&line);

        let mut line_tokens = tokenizer::tokenize(rest)?;
        line_tokens.push(Token::NewLine);

        // blank lines dont affect identation
        if line_tokens == [Token::NewLine] {
            tokens.push(Token::NewLine);
            continue;
        }

        while depth > open_blocks {
            tokens.push(Token::Indent);
            open_blocks += 1;
        }
        while depth < open_blocks {
            tokens.push(Token::Dedent);
            open_blocks -= 1;
        }

        tokens.append(&mut line_tokens);
    }

    while open_blocks > 0 {
        tokens.push(Token::Dedent);
        open_blocks -= 1;
    }

    tokens.push(Token::Eof);

    Ok(tokens)
}

fn split_indentation(line: &str) -> (usize, &str) {
    let depth = line.chars().take_while(|c| *c == '\t').count();
    (depth, &line[depth..])
}
