pub use torel_diagnostics::Span;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Token<'src> {
    pub kind: TokenKind<'src>,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TokenKind<'src> {
    Ident(&'src str),
    Keyword(Keyword),
    Int(&'src str),
    Text(&'src str),
    Dot,
    Semicolon,
    Comma,
    Colon,
    Equal,
    LBrace,
    RBrace,
    LParen,
    RParen,
    Arrow,
    Unknown(&'src str),
    Eof,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Keyword {
    Unit,
    Bring,
    Export,
    Proc,
    Type,
    Choice,
    Contract,
    Implement,
    Fix,
    Slot,
    Own,
    Arena,
    Does,
    Fails,
    Return,
    If,
    Else,
    True,
    False,
}

pub fn lex(source: &str) -> Vec<Token<'_>> {
    let mut lexer = Lexer { source, offset: 0 };
    let mut tokens = Vec::new();

    loop {
        let token = lexer.next_token();
        let done = token.kind == TokenKind::Eof;
        tokens.push(token);

        if done {
            return tokens;
        }
    }
}

struct Lexer<'src> {
    source: &'src str,
    offset: usize,
}

impl<'src> Lexer<'src> {
    fn next_token(&mut self) -> Token<'src> {
        self.skip_whitespace_and_comments();

        let start = self.offset;
        let Some(ch) = self.peek_char() else {
            return Token {
                kind: TokenKind::Eof,
                span: Span { start, end: start },
            };
        };

        let kind = match ch {
            '.' => self.single(TokenKind::Dot),
            ';' => self.single(TokenKind::Semicolon),
            ',' => self.single(TokenKind::Comma),
            ':' => self.single(TokenKind::Colon),
            '=' => self.single(TokenKind::Equal),
            '{' => self.single(TokenKind::LBrace),
            '}' => self.single(TokenKind::RBrace),
            '(' => self.single(TokenKind::LParen),
            ')' => self.single(TokenKind::RParen),
            '-' if self.peek_next_char() == Some('>') => {
                self.bump_char();
                self.bump_char();
                TokenKind::Arrow
            }
            '"' => self.text_literal(),
            '0'..='9' => self.number(),
            ch if is_ident_start(ch) => self.ident_or_keyword(),
            _ => {
                self.bump_char();
                TokenKind::Unknown(&self.source[start..self.offset])
            }
        };

        Token {
            kind,
            span: Span {
                start,
                end: self.offset,
            },
        }
    }

    fn single(&mut self, kind: TokenKind<'src>) -> TokenKind<'src> {
        self.bump_char();
        kind
    }

    fn text_literal(&mut self) -> TokenKind<'src> {
        let start = self.offset;
        self.bump_char();

        while let Some(ch) = self.peek_char() {
            self.bump_char();

            if ch == '"' {
                break;
            }
        }

        TokenKind::Text(&self.source[start..self.offset])
    }

    fn number(&mut self) -> TokenKind<'src> {
        let start = self.offset;

        while matches!(self.peek_char(), Some('0'..='9')) {
            self.bump_char();
        }

        TokenKind::Int(&self.source[start..self.offset])
    }

    fn ident_or_keyword(&mut self) -> TokenKind<'src> {
        let start = self.offset;

        while matches!(self.peek_char(), Some(ch) if is_ident_continue(ch)) {
            self.bump_char();
        }

        let text = &self.source[start..self.offset];

        match text {
            "unit" => TokenKind::Keyword(Keyword::Unit),
            "bring" => TokenKind::Keyword(Keyword::Bring),
            "export" => TokenKind::Keyword(Keyword::Export),
            "proc" => TokenKind::Keyword(Keyword::Proc),
            "type" => TokenKind::Keyword(Keyword::Type),
            "choice" => TokenKind::Keyword(Keyword::Choice),
            "contract" => TokenKind::Keyword(Keyword::Contract),
            "implement" => TokenKind::Keyword(Keyword::Implement),
            "fix" => TokenKind::Keyword(Keyword::Fix),
            "slot" => TokenKind::Keyword(Keyword::Slot),
            "own" => TokenKind::Keyword(Keyword::Own),
            "arena" => TokenKind::Keyword(Keyword::Arena),
            "does" => TokenKind::Keyword(Keyword::Does),
            "fails" => TokenKind::Keyword(Keyword::Fails),
            "return" => TokenKind::Keyword(Keyword::Return),
            "if" => TokenKind::Keyword(Keyword::If),
            "else" => TokenKind::Keyword(Keyword::Else),
            "true" => TokenKind::Keyword(Keyword::True),
            "false" => TokenKind::Keyword(Keyword::False),
            _ => TokenKind::Ident(text),
        }
    }

    fn skip_whitespace_and_comments(&mut self) {
        loop {
            while matches!(self.peek_char(), Some(ch) if ch.is_whitespace()) {
                self.bump_char();
            }

            if self.rest().starts_with("//") {
                while !matches!(self.peek_char(), None | Some('\n')) {
                    self.bump_char();
                }
                continue;
            }

            if self.rest().starts_with("/*") {
                self.bump_char();
                self.bump_char();

                while !self.rest().starts_with("*/") && self.peek_char().is_some() {
                    self.bump_char();
                }

                if self.rest().starts_with("*/") {
                    self.bump_char();
                    self.bump_char();
                }
                continue;
            }

            break;
        }
    }

    fn rest(&self) -> &'src str {
        &self.source[self.offset..]
    }

    fn peek_char(&self) -> Option<char> {
        self.rest().chars().next()
    }

    fn peek_next_char(&self) -> Option<char> {
        let mut chars = self.rest().chars();
        chars.next()?;
        chars.next()
    }

    fn bump_char(&mut self) -> Option<char> {
        let ch = self.peek_char()?;
        self.offset += ch.len_utf8();
        Some(ch)
    }
}

fn is_ident_start(ch: char) -> bool {
    ch == '_' || ch.is_ascii_alphabetic()
}

fn is_ident_continue(ch: char) -> bool {
    is_ident_start(ch) || ch.is_ascii_digit()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn lexes_unit_declaration() {
        let tokens = lex("unit app.server;");

        assert_eq!(tokens[0].kind, TokenKind::Keyword(Keyword::Unit));
        assert_eq!(tokens[1].kind, TokenKind::Ident("app"));
        assert_eq!(tokens[2].kind, TokenKind::Dot);
        assert_eq!(tokens[3].kind, TokenKind::Ident("server"));
        assert_eq!(tokens[4].kind, TokenKind::Semicolon);
    }
}
