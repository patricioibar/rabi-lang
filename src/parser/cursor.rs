use crate::token::Token;

pub(super) struct Cursor {
    tokens: std::iter::Peekable<std::vec::IntoIter<Token>>,
}

impl Cursor {
    pub(super) fn new(tokens: Vec<Token>) -> Self {
        Cursor {
            tokens: tokens.into_iter().peekable(),
        }
    }

    pub(super) fn peek(&mut self) -> Option<&Token> {
        self.tokens.peek()
    }

    pub(super) fn peek_cloned(&mut self) -> Option<Token> {
        self.tokens.peek().cloned()
    }

    pub(super) fn advance(&mut self) -> Option<Token> {
        self.tokens.next()
    }

    pub(super) fn advance_or_err(&mut self) -> Result<Token, String> {
        self.tokens
            .next()
            .ok_or_else(|| "Unexpected end of input".to_string())
    }

    pub(super) fn peek_is(&mut self, token: &Token) -> bool {
        self.tokens.peek() == Some(token)
    }

    pub(super) fn peek_any(&mut self, tokens: &[Token]) -> bool {
        self.tokens.peek().is_some_and(|next| tokens.contains(next))
    }

    pub(super) fn accept(&mut self, token: &Token) -> bool {
        if self.peek_is(token) {
            self.tokens.next();
            true
        } else {
            false
        }
    }

    pub(super) fn expect(&mut self, token: &Token, context: &str) -> Result<(), String> {
        if self.accept(token) {
            Ok(())
        } else {
            Err(format!("Expected '{}' {}", token, context))
        }
    }

    pub(super) fn skip_blank_lines(&mut self) {
        while self.peek_is(&Token::NewLine) {
            self.tokens.next();
        }
    }

    pub(super) fn at_end(&mut self) -> bool {
        matches!(self.tokens.peek(), Some(Token::Eof) | None)
    }

    pub(super) fn at_statement_terminator(&mut self) -> bool {
        matches!(
            self.tokens.peek(),
            Some(Token::NewLine)
                | Some(Token::Eof)
                | Some(Token::Dedent)
                | Some(Token::Else)
                | None
        )
    }

    pub(super) fn end_statement(&mut self) -> Result<(), String> {
        if !self.at_statement_terminator() {
            return Err(format!(
                "Expected newline or end of file after statement, found {:?}",
                self.tokens.peek()
            ));
        }
        self.accept(&Token::NewLine);
        Ok(())
    }

    #[cfg(test)]
    pub(super) fn remaining(self) -> Vec<Token> {
        self.tokens.collect()
    }
}
