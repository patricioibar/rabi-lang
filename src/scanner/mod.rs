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
        let (depth, rest) = split_indentation(&line)?;

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

/// Number of spaces that stand for one indentation level.
const SPACES_PER_INDENT: usize = 4;

fn split_indentation(line: &str) -> Result<(usize, &str), String> {
    let indent = line
        .find(|c: char| c != '\t' && c != ' ')
        .unwrap_or(line.len());
    let (indent, rest) = line.split_at(indent);

    let tabs = indent.chars().filter(|c| *c == '\t').count();
    let spaces = indent.len() - tabs;

    if tabs > 0 && spaces > 0 {
        return Err(format!(
            "Inconsistent indentation: line mixes tabs and spaces: {:?}",
            line
        ));
    }

    if spaces % SPACES_PER_INDENT != 0 {
        return Err(format!(
            "Inconsistent indentation: {} spaces is not a multiple of {}: {:?}",
            spaces, SPACES_PER_INDENT, line
        ));
    }

    Ok((tabs + spaces / SPACES_PER_INDENT, rest))
}
