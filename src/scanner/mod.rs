use std::{io::BufRead, mem::take};

use crate::token::Token;

mod tokenizer;

#[cfg(test)]
mod tests;

#[derive(Default)]
pub struct Scanner {
    tokens: Vec<Token>,
    open_blocks: usize,
    expects_block: bool,
}

impl Scanner {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn scan_one_line(&mut self, line: &str) -> Result<(), String> {
        let (depth, rest) = split_indentation(line)?;

        let mut line_tokens = tokenizer::tokenize(rest)?;

        // blank lines dont affect identation
        if line_tokens.is_empty() {
            self.tokens.push(Token::NewLine);
            return Ok(());
        }

        while depth > self.open_blocks {
            self.tokens.push(Token::Indent);
            self.open_blocks += 1;
        }
        while depth < self.open_blocks {
            self.tokens.push(Token::Dedent);
            self.open_blocks -= 1;
        }

        self.expects_block = line_tokens.last() == Some(&Token::Colon);

        line_tokens.push(Token::NewLine);
        self.tokens.append(&mut line_tokens);

        Ok(())
    }

    pub fn is_block_open(&self) -> bool {
        self.open_blocks > 0 || self.expects_block
    }

    /// Closes every open block and returns the tokens scanned so far
    pub fn finish(&mut self) -> Vec<Token> {
        while self.open_blocks > 0 {
            self.tokens.push(Token::Dedent);
            self.open_blocks -= 1;
        }
        self.tokens.push(Token::Eof);
        self.expects_block = false;

        take(&mut self.tokens)
    }
}

pub fn scan(source: impl BufRead) -> Result<Vec<Token>, String> {
    let mut scanner = Scanner::new();

    for line in source.lines() {
        let line = line.map_err(|e| format!("Error reading line from source: {}", e))?;
        scanner.scan_one_line(&line)?;
    }

    Ok(scanner.finish())
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
