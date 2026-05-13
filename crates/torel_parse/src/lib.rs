use torel_ast::{SourceFile, UnitDecl};
use torel_lexer::{Keyword, Token, TokenKind};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParseError {
    pub message: String,
}

pub fn parse_source_file(tokens: &[Token<'_>]) -> Result<SourceFile, ParseError> {
    let mut parser = Parser { tokens, cursor: 0 };
    parser.source_file()
}

struct Parser<'tok, 'src> {
    tokens: &'tok [Token<'src>],
    cursor: usize,
}

impl Parser<'_, '_> {
    fn source_file(&mut self) -> Result<SourceFile, ParseError> {
        let unit = if self.at_keyword(Keyword::Unit) {
            Some(self.unit_decl()?)
        } else {
            None
        };

        Ok(SourceFile {
            unit,
            items: Vec::new(),
        })
    }

    fn unit_decl(&mut self) -> Result<UnitDecl, ParseError> {
        self.expect_keyword(Keyword::Unit)?;
        let mut path = vec![self.expect_ident()?];

        while self.eat(TokenKind::Dot) {
            path.push(self.expect_ident()?);
        }

        self.expect(TokenKind::Semicolon)?;

        Ok(UnitDecl { path })
    }

    fn at_keyword(&self, keyword: Keyword) -> bool {
        matches!(self.peek(), Some(TokenKind::Keyword(actual)) if *actual == keyword)
    }

    fn expect_keyword(&mut self, keyword: Keyword) -> Result<(), ParseError> {
        if self.at_keyword(keyword) {
            self.cursor += 1;
            Ok(())
        } else {
            Err(ParseError {
                message: format!("expected keyword `{keyword:?}`"),
            })
        }
    }

    fn expect_ident(&mut self) -> Result<String, ParseError> {
        match self.peek() {
            Some(TokenKind::Ident(name)) => {
                let name = (*name).to_owned();
                self.cursor += 1;
                Ok(name)
            }
            _ => Err(ParseError {
                message: "expected identifier".to_owned(),
            }),
        }
    }

    fn expect(&mut self, kind: TokenKind<'_>) -> Result<(), ParseError> {
        if self.eat(kind.clone()) {
            Ok(())
        } else {
            Err(ParseError {
                message: format!("expected token `{kind:?}`"),
            })
        }
    }

    fn eat(&mut self, kind: TokenKind<'_>) -> bool {
        if self.peek() == Some(&kind) {
            self.cursor += 1;
            true
        } else {
            false
        }
    }

    fn peek(&self) -> Option<&TokenKind<'_>> {
        self.tokens.get(self.cursor).map(|token| &token.kind)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use torel_lexer::lex;

    #[test]
    fn parses_unit_declaration() {
        let tokens = lex("unit app.server;");
        let file = parse_source_file(&tokens).expect("source file should parse");

        assert_eq!(
            file.unit.expect("unit decl").path,
            vec!["app".to_owned(), "server".to_owned()]
        );
    }
}
