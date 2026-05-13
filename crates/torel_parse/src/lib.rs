use torel_ast::{
    BindingKind, Block, Expr, Item, Param, ProcDecl, SourceFile, Stmt, TypeRef, UnitDecl,
    Visibility,
};
use torel_lexer::{Keyword, Span, Token, TokenKind};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParseError {
    pub message: String,
    pub span: Option<Span>,
}

impl std::fmt::Display for ParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.span {
            Some(span) => write!(f, "{} at {}..{}", self.message, span.start, span.end),
            None => f.write_str(&self.message),
        }
    }
}

impl std::error::Error for ParseError {}

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

        let mut items = Vec::new();

        while !self.at_eof() {
            items.push(self.item()?);
        }

        self.expect(TokenKind::Eof)?;

        Ok(SourceFile { unit, items })
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

    fn item(&mut self) -> Result<Item, ParseError> {
        let visibility = if self.eat_keyword(Keyword::Export) {
            Visibility::Export
        } else {
            Visibility::Private
        };

        if self.at_keyword(Keyword::Proc) {
            Ok(Item::Proc(self.proc_decl(visibility)?))
        } else {
            Err(self.error_current("expected top-level item"))
        }
    }

    fn proc_decl(&mut self, visibility: Visibility) -> Result<ProcDecl, ParseError> {
        self.expect_keyword(Keyword::Proc)?;
        let name = self.expect_ident()?;
        let params = self.param_list()?;
        self.expect(TokenKind::Arrow)?;
        let return_type = self.type_ref()?;
        let body = self.block()?;

        Ok(ProcDecl {
            visibility,
            name,
            params,
            return_type,
            body,
        })
    }

    fn param_list(&mut self) -> Result<Vec<Param>, ParseError> {
        self.expect(TokenKind::LParen)?;

        if self.eat(TokenKind::RParen) {
            return Ok(Vec::new());
        }

        let mut params = Vec::new();

        loop {
            let name = self.expect_ident()?;
            self.expect(TokenKind::Colon)?;
            let ty = self.type_ref()?;
            params.push(Param { name, ty });

            if !self.eat(TokenKind::Comma) {
                break;
            }
        }

        self.expect(TokenKind::RParen)?;
        Ok(params)
    }

    fn type_ref(&mut self) -> Result<TypeRef, ParseError> {
        Ok(TypeRef { path: self.path()? })
    }

    fn block(&mut self) -> Result<Block, ParseError> {
        self.expect(TokenKind::LBrace)?;
        let mut stmts = Vec::new();

        while !self.eat(TokenKind::RBrace) {
            if self.at_eof() {
                return Err(self.error_current("expected statement or `}`"));
            }

            stmts.push(self.stmt()?);
        }

        Ok(Block { stmts })
    }

    fn stmt(&mut self) -> Result<Stmt, ParseError> {
        if self.eat_keyword(Keyword::Fix) {
            self.local_stmt(BindingKind::Fix)
        } else if self.eat_keyword(Keyword::Slot) {
            self.local_stmt(BindingKind::Slot)
        } else if self.eat_keyword(Keyword::Return) {
            let expr = self.expr()?;
            self.expect(TokenKind::Semicolon)?;
            Ok(Stmt::Return(expr))
        } else {
            let target = self.path()?;
            self.expect(TokenKind::Equal)?;
            let value = self.expr()?;
            self.expect(TokenKind::Semicolon)?;
            Ok(Stmt::Assign { target, value })
        }
    }

    fn local_stmt(&mut self, kind: BindingKind) -> Result<Stmt, ParseError> {
        let name = self.expect_ident()?;
        self.expect(TokenKind::Colon)?;
        let ty = self.type_ref()?;
        self.expect(TokenKind::Equal)?;
        let value = self.expr()?;
        self.expect(TokenKind::Semicolon)?;

        Ok(Stmt::Local {
            kind,
            name,
            ty,
            value,
        })
    }

    fn expr(&mut self) -> Result<Expr, ParseError> {
        match self.peek() {
            Some(TokenKind::Int(value)) => {
                let value = (*value).to_owned();
                self.cursor += 1;
                return Ok(Expr::Int(value));
            }
            Some(TokenKind::Text(value)) => {
                let value = (*value).to_owned();
                self.cursor += 1;
                return Ok(Expr::Text(value));
            }
            Some(TokenKind::Keyword(Keyword::True)) => {
                self.cursor += 1;
                return Ok(Expr::Bool(true));
            }
            Some(TokenKind::Keyword(Keyword::False)) => {
                self.cursor += 1;
                return Ok(Expr::Bool(false));
            }
            _ => {}
        }

        let path = self.path()?;
        if self.eat(TokenKind::LParen) {
            Ok(Expr::Call {
                callee: path,
                args: self.arg_list()?,
            })
        } else {
            Ok(Expr::Path(path))
        }
    }

    fn arg_list(&mut self) -> Result<Vec<Expr>, ParseError> {
        if self.eat(TokenKind::RParen) {
            return Ok(Vec::new());
        }

        let mut args = Vec::new();

        loop {
            args.push(self.expr()?);

            if !self.eat(TokenKind::Comma) {
                break;
            }
        }

        self.expect(TokenKind::RParen)?;
        Ok(args)
    }

    fn path(&mut self) -> Result<Vec<String>, ParseError> {
        let mut path = vec![self.expect_ident()?];

        while self.eat(TokenKind::Dot) {
            path.push(self.expect_ident()?);
        }

        Ok(path)
    }

    fn at_keyword(&self, keyword: Keyword) -> bool {
        matches!(self.peek(), Some(TokenKind::Keyword(actual)) if *actual == keyword)
    }

    fn eat_keyword(&mut self, keyword: Keyword) -> bool {
        if self.at_keyword(keyword) {
            self.cursor += 1;
            true
        } else {
            false
        }
    }

    fn expect_keyword(&mut self, keyword: Keyword) -> Result<(), ParseError> {
        if self.eat_keyword(keyword) {
            Ok(())
        } else {
            Err(self.error_current(format!("expected keyword `{keyword:?}`")))
        }
    }

    fn expect_ident(&mut self) -> Result<String, ParseError> {
        match self.peek() {
            Some(TokenKind::Ident(name)) => {
                let name = (*name).to_owned();
                self.cursor += 1;
                Ok(name)
            }
            _ => Err(self.error_current("expected identifier")),
        }
    }

    fn expect(&mut self, kind: TokenKind<'_>) -> Result<(), ParseError> {
        if self.eat(kind.clone()) {
            Ok(())
        } else {
            Err(self.error_current(format!("expected token `{kind:?}`")))
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

    fn at_eof(&self) -> bool {
        matches!(self.peek(), Some(TokenKind::Eof))
    }

    fn current_span(&self) -> Option<Span> {
        self.tokens.get(self.cursor).map(|token| token.span)
    }

    fn error_current(&self, message: impl Into<String>) -> ParseError {
        ParseError {
            message: message.into(),
            span: self.current_span(),
        }
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

    #[test]
    fn parses_exported_proc() {
        let tokens = lex(r#"
            unit app.server;

            export proc main() -> Exit {
                return Exit.ok;
            }
            "#);
        let file = parse_source_file(&tokens).expect("source file should parse");

        assert_eq!(file.items.len(), 1);
        let Item::Proc(proc) = &file.items[0];
        assert_eq!(proc.visibility, Visibility::Export);
        assert_eq!(proc.name, "main");
        assert_eq!(proc.return_type.path, vec!["Exit".to_owned()]);
        assert_eq!(proc.body.stmts.len(), 1);
    }

    #[test]
    fn parses_call_expression() {
        let tokens = lex(r#"
            unit app.calls;

            proc make_exit() -> Exit {
                return Exit.ok;
            }

            export proc main() -> Exit {
                return make_exit();
            }
            "#);
        let file = parse_source_file(&tokens).expect("source file should parse");

        assert_eq!(file.items.len(), 2);
        let Item::Proc(proc) = &file.items[1];
        let Stmt::Return(Expr::Call { callee, args }) = &proc.body.stmts[0] else {
            panic!("main should return a call expression");
        };

        assert_eq!(callee, &vec!["make_exit".to_owned()]);
        assert!(args.is_empty());
    }

    #[test]
    fn parses_fix_statement_and_literals() {
        let tokens = lex(r#"
            unit app.locals;

            export proc main() -> Int32 {
                fix answer: Int32 = 42;
                return answer;
            }
            "#);
        let file = parse_source_file(&tokens).expect("source file should parse");

        let Item::Proc(proc) = &file.items[0];
        let Stmt::Local {
            kind,
            name,
            ty,
            value,
        } = &proc.body.stmts[0]
        else {
            panic!("first statement should be fix");
        };

        assert_eq!(kind, &BindingKind::Fix);
        assert_eq!(name, "answer");
        assert_eq!(ty.path, vec!["Int32".to_owned()]);
        assert_eq!(value, &Expr::Int("42".to_owned()));
    }

    #[test]
    fn parses_slot_and_assignment() {
        let tokens = lex(r#"
            unit app.slots;

            export proc main() -> Int32 {
                slot answer: Int32 = 40;
                answer = 42;
                return answer;
            }
            "#);
        let file = parse_source_file(&tokens).expect("source file should parse");

        let Item::Proc(proc) = &file.items[0];
        let Stmt::Local { kind, name, .. } = &proc.body.stmts[0] else {
            panic!("first statement should be slot");
        };
        let Stmt::Assign { target, value } = &proc.body.stmts[1] else {
            panic!("second statement should be assignment");
        };

        assert_eq!(kind, &BindingKind::Slot);
        assert_eq!(name, "answer");
        assert_eq!(target, &vec!["answer".to_owned()]);
        assert_eq!(value, &Expr::Int("42".to_owned()));
    }

    #[test]
    fn rejects_trailing_junk() {
        let tokens = lex("unit app.server; ???");
        let err = parse_source_file(&tokens).expect_err("trailing junk should fail");

        assert_eq!(err.message, "expected top-level item");
        assert_eq!(err.span, Some(Span { start: 17, end: 18 }));
    }
}
