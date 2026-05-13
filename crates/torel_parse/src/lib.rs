use torel_ast::BinaryOp;
use torel_ast::{
    BindingKind, Block, Expr, ExprKind, Item, Param, Path, ProcDecl, SourceFile, Stmt, StmtKind,
    TypeRef, UnaryOp, UnitDecl, Visibility,
};
use torel_diagnostics::{Diagnostic, Label};
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

impl ParseError {
    #[must_use]
    pub fn into_diagnostic(self) -> Diagnostic {
        let mut diagnostic = Diagnostic::error(self.message.clone());

        if let Some(span) = self.span {
            diagnostic = diagnostic.with_label(Label::primary(span, self.message));
        }

        diagnostic
    }
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

        let mut items = Vec::new();

        while !self.at_eof() {
            items.push(self.item()?);
        }

        self.expect(TokenKind::Eof)?;

        Ok(SourceFile { unit, items })
    }

    fn unit_decl(&mut self) -> Result<UnitDecl, ParseError> {
        let start = self.expect_keyword(Keyword::Unit)?;
        let path = self.path()?;
        let end = self.expect(TokenKind::Semicolon)?;

        Ok(UnitDecl {
            path,
            span: start.join(end),
        })
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
        let start = self.expect_keyword(Keyword::Proc)?;
        let (name, name_span) = self.expect_ident_with_span()?;
        let params = self.param_list()?;
        self.expect(TokenKind::Arrow)?;
        let return_type = self.type_ref()?;
        let body = self.block()?;
        let span = start.join(body.span);

        Ok(ProcDecl {
            visibility,
            visibility_span: None,
            name,
            name_span,
            params,
            return_type,
            body,
            span,
        })
    }

    fn param_list(&mut self) -> Result<Vec<Param>, ParseError> {
        self.expect(TokenKind::LParen)?;

        if self.eat(TokenKind::RParen) {
            return Ok(Vec::new());
        }

        let mut params = Vec::new();

        loop {
            let (name, name_span) = self.expect_ident_with_span()?;
            self.expect(TokenKind::Colon)?;
            let ty = self.type_ref()?;
            params.push(Param {
                name,
                name_span,
                ty,
            });

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
        let start = self.expect(TokenKind::LBrace)?;
        let mut stmts = Vec::new();
        let mut tail = None;

        loop {
            if self.eat(TokenKind::RBrace) {
                let end = self.previous_span().unwrap_or(start);
                return Ok(Block {
                    stmts,
                    tail,
                    span: start.join(end),
                });
            }

            if self.at_eof() {
                return Err(self.error_current("expected statement, final expression, or `}`"));
            }

            if self.at_stmt_start() {
                stmts.push(self.stmt()?);
                continue;
            }

            if self.at_expr_start() {
                tail = Some(self.expr()?);
                let end = self.expect(TokenKind::RBrace)?;
                return Ok(Block {
                    stmts,
                    tail,
                    span: start.join(end),
                });
            }

            return Err(self.error_current("expected statement, final expression, or `}`"));
        }
    }

    fn stmt(&mut self) -> Result<Stmt, ParseError> {
        if self.at_keyword(Keyword::Fix) {
            let start = self.expect_keyword(Keyword::Fix)?;
            self.local_stmt(BindingKind::Fix, start)
        } else if self.at_keyword(Keyword::Slot) {
            let start = self.expect_keyword(Keyword::Slot)?;
            self.local_stmt(BindingKind::Slot, start)
        } else if self.at_keyword(Keyword::If) {
            let start = self.expect_keyword(Keyword::If)?;
            self.if_stmt(start)
        } else if self.at_keyword(Keyword::Return) {
            let start = self.expect_keyword(Keyword::Return)?;
            let expr = self.expr()?;
            let end = self.expect(TokenKind::Semicolon)?;
            Ok(Stmt {
                kind: StmtKind::Return(expr),
                span: start.join(end),
            })
        } else {
            let target = self.path()?;
            let start = target.span;
            self.expect(TokenKind::Equal)?;
            let value = self.expr()?;
            let end = self.expect(TokenKind::Semicolon)?;
            Ok(Stmt {
                kind: StmtKind::Assign { target, value },
                span: start.join(end),
            })
        }
    }

    fn local_stmt(&mut self, kind: BindingKind, start: Span) -> Result<Stmt, ParseError> {
        let (name, name_span) = self.expect_ident_with_span()?;
        self.expect(TokenKind::Colon)?;
        let ty = self.type_ref()?;
        self.expect(TokenKind::Equal)?;
        let value = self.expr()?;
        let end = self.expect(TokenKind::Semicolon)?;

        Ok(Stmt {
            kind: StmtKind::Local {
                kind,
                name,
                name_span,
                ty,
                value,
            },
            span: start.join(end),
        })
    }

    fn if_stmt(&mut self, start: Span) -> Result<Stmt, ParseError> {
        let condition = self.expr()?;
        let then_block = self.block()?;
        let else_block = if self.eat_keyword(Keyword::Else) {
            Some(self.block()?)
        } else {
            None
        };
        let end = else_block
            .as_ref()
            .map(|block| block.span)
            .unwrap_or(then_block.span);

        Ok(Stmt {
            kind: StmtKind::If {
                condition,
                then_block,
                else_block,
            },
            span: start.join(end),
        })
    }

    fn expr(&mut self) -> Result<Expr, ParseError> {
        self.expr_bp(0)
    }

    fn expr_bp(&mut self, min_bp: u8) -> Result<Expr, ParseError> {
        let mut lhs = self.prefix_expr()?;

        while let Some((op, left_bp, right_bp)) = self.current_binary_op() {
            if left_bp < min_bp {
                break;
            }

            let op_span = self.current_span().expect("operator should have span");
            self.cursor += 1;
            let rhs = self.expr_bp(right_bp)?;
            let span = lhs.span.join(rhs.span);

            lhs = Expr {
                kind: ExprKind::Binary {
                    op,
                    op_span,
                    lhs: Box::new(lhs),
                    rhs: Box::new(rhs),
                },
                span,
            };
        }

        Ok(lhs)
    }

    fn prefix_expr(&mut self) -> Result<Expr, ParseError> {
        if let Some(op) = self.current_unary_op() {
            let op_span = self.current_span().expect("operator should have span");
            self.cursor += 1;
            let expr = self.expr_bp(13)?;
            let span = op_span.join(expr.span);

            return Ok(Expr {
                kind: ExprKind::Unary {
                    op,
                    op_span,
                    expr: Box::new(expr),
                },
                span,
            });
        }

        self.primary_expr()
    }

    fn primary_expr(&mut self) -> Result<Expr, ParseError> {
        match self.peek() {
            Some(TokenKind::Int(value)) => {
                let value = (*value).to_owned();
                let span = self.current_span().expect("current token should have span");
                self.cursor += 1;
                return Ok(Expr {
                    kind: ExprKind::Int(value),
                    span,
                });
            }
            Some(TokenKind::Text(value)) => {
                let value = (*value).to_owned();
                let span = self.current_span().expect("current token should have span");
                self.cursor += 1;
                return Ok(Expr {
                    kind: ExprKind::Text(value),
                    span,
                });
            }
            Some(TokenKind::Keyword(Keyword::True)) => {
                let span = self.current_span().expect("current token should have span");
                self.cursor += 1;
                return Ok(Expr {
                    kind: ExprKind::Bool(true),
                    span,
                });
            }
            Some(TokenKind::Keyword(Keyword::False)) => {
                let span = self.current_span().expect("current token should have span");
                self.cursor += 1;
                return Ok(Expr {
                    kind: ExprKind::Bool(false),
                    span,
                });
            }
            Some(TokenKind::LParen) => {
                let start = self.expect(TokenKind::LParen)?;
                let mut expr = self.expr()?;
                let end = self.expect(TokenKind::RParen)?;
                expr.span = start.join(end);
                return Ok(expr);
            }
            _ => {}
        }

        let path = self.path()?;
        if self.eat(TokenKind::LParen) {
            let (args, end) = self.arg_list()?;
            let span = path.span.join(end);
            Ok(Expr {
                kind: ExprKind::Call { callee: path, args },
                span,
            })
        } else {
            Ok(Expr {
                span: path.span,
                kind: ExprKind::Path(path),
            })
        }
    }

    fn current_unary_op(&self) -> Option<UnaryOp> {
        match self.peek()? {
            TokenKind::Bang => Some(UnaryOp::Not),
            TokenKind::Minus => Some(UnaryOp::Neg),
            _ => None,
        }
    }

    fn current_binary_op(&self) -> Option<(BinaryOp, u8, u8)> {
        match self.peek()? {
            TokenKind::PipePipe => Some((BinaryOp::Or, 1, 2)),
            TokenKind::AmpAmp => Some((BinaryOp::And, 3, 4)),
            TokenKind::EqualEqual => Some((BinaryOp::Eq, 5, 6)),
            TokenKind::BangEqual => Some((BinaryOp::NotEq, 5, 6)),
            TokenKind::Less => Some((BinaryOp::Lt, 7, 8)),
            TokenKind::LessEqual => Some((BinaryOp::LtEq, 7, 8)),
            TokenKind::Greater => Some((BinaryOp::Gt, 7, 8)),
            TokenKind::GreaterEqual => Some((BinaryOp::GtEq, 7, 8)),
            TokenKind::Plus => Some((BinaryOp::Add, 9, 10)),
            TokenKind::Minus => Some((BinaryOp::Sub, 9, 10)),
            TokenKind::Star => Some((BinaryOp::Mul, 11, 12)),
            TokenKind::Slash => Some((BinaryOp::Div, 11, 12)),
            TokenKind::Percent => Some((BinaryOp::Rem, 11, 12)),
            _ => None,
        }
    }

    fn arg_list(&mut self) -> Result<(Vec<Expr>, Span), ParseError> {
        if self.eat(TokenKind::RParen) {
            let end = self.previous_span().expect("right paren should have span");
            return Ok((Vec::new(), end));
        }

        let mut args = Vec::new();

        loop {
            args.push(self.expr()?);

            if !self.eat(TokenKind::Comma) {
                break;
            }
        }

        let end = self.expect(TokenKind::RParen)?;
        Ok((args, end))
    }

    fn path(&mut self) -> Result<Path, ParseError> {
        let (first, start) = self.expect_ident_with_span()?;
        let mut segments = vec![first];
        let mut end = start;

        while self.eat(TokenKind::Dot) {
            let (segment, span) = self.expect_ident_with_span()?;
            segments.push(segment);
            end = span;
        }

        Ok(Path {
            segments,
            span: start.join(end),
        })
    }

    fn at_stmt_start(&self) -> bool {
        self.at_keyword(Keyword::Fix)
            || self.at_keyword(Keyword::Slot)
            || self.at_keyword(Keyword::If)
            || self.at_keyword(Keyword::Return)
            || self.path_is_followed_by_equal()
    }

    fn at_expr_start(&self) -> bool {
        matches!(
            self.peek(),
            Some(
                TokenKind::Ident(_)
                    | TokenKind::Int(_)
                    | TokenKind::Text(_)
                    | TokenKind::LParen
                    | TokenKind::Bang
                    | TokenKind::Minus
                    | TokenKind::Keyword(Keyword::True | Keyword::False)
            )
        )
    }

    fn path_is_followed_by_equal(&self) -> bool {
        let mut cursor = self.cursor;

        if !matches!(
            self.tokens.get(cursor).map(|token| &token.kind),
            Some(TokenKind::Ident(_))
        ) {
            return false;
        }

        cursor += 1;

        while matches!(
            self.tokens.get(cursor).map(|token| &token.kind),
            Some(TokenKind::Dot)
        ) {
            cursor += 1;

            if !matches!(
                self.tokens.get(cursor).map(|token| &token.kind),
                Some(TokenKind::Ident(_))
            ) {
                return false;
            }

            cursor += 1;
        }

        matches!(
            self.tokens.get(cursor).map(|token| &token.kind),
            Some(TokenKind::Equal)
        )
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

    fn expect_keyword(&mut self, keyword: Keyword) -> Result<Span, ParseError> {
        if self.eat_keyword(keyword) {
            Ok(self
                .previous_span()
                .expect("expected keyword should have span"))
        } else {
            Err(self.error_current(format!("expected keyword `{keyword:?}`")))
        }
    }

    fn expect_ident_with_span(&mut self) -> Result<(String, Span), ParseError> {
        match self.peek() {
            Some(TokenKind::Ident(name)) => {
                let name = (*name).to_owned();
                let span = self.current_span().expect("current token should have span");
                self.cursor += 1;
                Ok((name, span))
            }
            _ => Err(self.error_current("expected identifier")),
        }
    }

    fn expect(&mut self, kind: TokenKind<'_>) -> Result<Span, ParseError> {
        if self.eat(kind.clone()) {
            Ok(self
                .previous_span()
                .expect("expected token should have span"))
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

    fn previous_span(&self) -> Option<Span> {
        self.cursor
            .checked_sub(1)
            .and_then(|cursor| self.tokens.get(cursor))
            .map(|token| token.span)
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
            file.unit.expect("unit decl").path.segments,
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
        assert_eq!(proc.return_type.path.segments, vec!["Exit".to_owned()]);
        assert_eq!(proc.body.stmts.len(), 1);
        assert_eq!(proc.body.tail, None);
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
        let StmtKind::Return(Expr {
            kind: ExprKind::Call { callee, args },
            ..
        }) = &proc.body.stmts[0].kind
        else {
            panic!("main should return a call expression");
        };

        assert_eq!(callee.segments, vec!["make_exit".to_owned()]);
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
        let StmtKind::Local {
            kind,
            name,
            ty,
            value,
            ..
        } = &proc.body.stmts[0].kind
        else {
            panic!("first statement should be fix");
        };

        assert_eq!(kind, &BindingKind::Fix);
        assert_eq!(name, "answer");
        assert_eq!(ty.path.segments, vec!["Int32".to_owned()]);
        assert_eq!(value.kind, ExprKind::Int("42".to_owned()));
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
        let StmtKind::Local { kind, name, .. } = &proc.body.stmts[0].kind else {
            panic!("first statement should be slot");
        };
        let StmtKind::Assign { target, value } = &proc.body.stmts[1].kind else {
            panic!("second statement should be assignment");
        };

        assert_eq!(kind, &BindingKind::Slot);
        assert_eq!(name, "answer");
        assert_eq!(target.segments, vec!["answer".to_owned()]);
        assert_eq!(value.kind, ExprKind::Int("42".to_owned()));
    }

    #[test]
    fn parses_if_else_statement() {
        let tokens = lex(r#"
            unit app.branching;

            export proc main() -> Int32 {
                if true {
                    return 1;
                } else {
                    return 2;
                }
            }
            "#);
        let file = parse_source_file(&tokens).expect("source file should parse");

        let Item::Proc(proc) = &file.items[0];
        let StmtKind::If {
            condition,
            then_block,
            else_block,
        } = &proc.body.stmts[0].kind
        else {
            panic!("first statement should be if");
        };

        assert_eq!(condition.kind, ExprKind::Bool(true));
        assert_eq!(then_block.stmts.len(), 1);
        assert_eq!(else_block.as_ref().expect("else block").stmts.len(), 1);
    }

    #[test]
    fn parses_final_expression() {
        let tokens = lex(r#"
            unit app.tail;

            export proc main() -> Int32 {
                fix answer: Int32 = 42;
                answer
            }
            "#);
        let file = parse_source_file(&tokens).expect("source file should parse");

        let Item::Proc(proc) = &file.items[0];

        assert_eq!(proc.body.stmts.len(), 1);
        let tail = proc.body.tail.as_ref().expect("tail expression");
        let ExprKind::Path(path) = &tail.kind else {
            panic!("tail should be a path");
        };
        assert_eq!(path.segments, vec!["answer".to_owned()]);
    }

    #[test]
    fn parses_binary_precedence() {
        let expr = parse_tail_expr("1 + 2 * 3");
        let ExprKind::Binary {
            op: BinaryOp::Add,
            lhs,
            rhs,
            ..
        } = &expr.kind
        else {
            panic!("tail should be addition");
        };

        assert_eq!(lhs.kind, ExprKind::Int("1".to_owned()));
        assert!(matches!(
            rhs.kind,
            ExprKind::Binary {
                op: BinaryOp::Mul,
                ..
            }
        ));
    }

    #[test]
    fn parses_parenthesized_precedence() {
        let expr = parse_tail_expr("(1 + 2) * 3");
        let ExprKind::Binary {
            op: BinaryOp::Mul,
            lhs,
            rhs,
            ..
        } = &expr.kind
        else {
            panic!("tail should be multiplication");
        };

        assert!(matches!(
            lhs.kind,
            ExprKind::Binary {
                op: BinaryOp::Add,
                ..
            }
        ));
        assert_eq!(rhs.kind, ExprKind::Int("3".to_owned()));
    }

    #[test]
    fn parses_unary_expressions() {
        assert!(matches!(
            parse_tail_expr("!false").kind,
            ExprKind::Unary {
                op: UnaryOp::Not,
                ..
            }
        ));
        assert!(matches!(
            parse_tail_expr("-42").kind,
            ExprKind::Unary {
                op: UnaryOp::Neg,
                ..
            }
        ));
    }

    #[test]
    fn parses_operator_args_and_if_condition() {
        let call = parse_tail_expr("foo(1 + 2, true && false)");
        let ExprKind::Call { args, .. } = &call.kind else {
            panic!("tail should be a call");
        };
        assert!(matches!(
            args[0].kind,
            ExprKind::Binary {
                op: BinaryOp::Add,
                ..
            }
        ));
        assert!(matches!(
            args[1].kind,
            ExprKind::Binary {
                op: BinaryOp::And,
                ..
            }
        ));

        let tokens = lex(r#"
            unit app.branching;

            export proc main() -> Int32 {
                if 1 < 2 {
                    10
                } else {
                    20
                }
            }
            "#);
        let file = parse_source_file(&tokens).expect("source file should parse");
        let Item::Proc(proc) = &file.items[0];
        let StmtKind::If { condition, .. } = &proc.body.stmts[0].kind else {
            panic!("first statement should be if");
        };

        assert!(matches!(
            condition.kind,
            ExprKind::Binary {
                op: BinaryOp::Lt,
                ..
            }
        ));
    }

    fn parse_tail_expr(source: &str) -> Expr {
        let source = format!("unit app.expr; export proc main() -> Int32 {{ {source} }}");
        let tokens = lex(&source);
        let file = parse_source_file(&tokens).expect("source file should parse");
        let Item::Proc(proc) = &file.items[0];
        proc.body.tail.clone().expect("tail expression")
    }

    #[test]
    fn rejects_trailing_junk() {
        let tokens = lex("unit app.server; ???");
        let err = parse_source_file(&tokens).expect_err("trailing junk should fail");

        assert_eq!(err.message, "expected top-level item");
        assert_eq!(err.span, Some(Span { start: 17, end: 18 }));
    }
}
