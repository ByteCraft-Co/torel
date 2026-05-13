use torel_diagnostics::Span;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SourceFile {
    pub unit: Option<UnitDecl>,
    pub items: Vec<Item>,
}

impl SourceFile {
    #[must_use]
    pub const fn empty() -> Self {
        Self {
            unit: None,
            items: Vec::new(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UnitDecl {
    pub path: Path,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Item {
    Proc(ProcDecl),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Visibility {
    Private,
    Export,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProcDecl {
    pub visibility: Visibility,
    pub visibility_span: Option<Span>,
    pub name: String,
    pub name_span: Span,
    pub params: Vec<Param>,
    pub return_type: TypeRef,
    pub body: Block,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Param {
    pub name: String,
    pub name_span: Span,
    pub ty: TypeRef,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TypeRef {
    pub path: Path,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Path {
    pub segments: Vec<String>,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Block {
    pub stmts: Vec<Stmt>,
    pub tail: Option<Expr>,
    pub span: Span,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BindingKind {
    Fix,
    Slot,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UnaryOp {
    Not,
    Neg,
}

impl UnaryOp {
    #[must_use]
    pub const fn symbol(self) -> &'static str {
        match self {
            Self::Not => "!",
            Self::Neg => "-",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BinaryOp {
    Add,
    Sub,
    Mul,
    Div,
    Rem,
    Eq,
    NotEq,
    Lt,
    LtEq,
    Gt,
    GtEq,
    And,
    Or,
}

impl BinaryOp {
    #[must_use]
    pub const fn symbol(self) -> &'static str {
        match self {
            Self::Add => "+",
            Self::Sub => "-",
            Self::Mul => "*",
            Self::Div => "/",
            Self::Rem => "%",
            Self::Eq => "==",
            Self::NotEq => "!=",
            Self::Lt => "<",
            Self::LtEq => "<=",
            Self::Gt => ">",
            Self::GtEq => ">=",
            Self::And => "&&",
            Self::Or => "||",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Stmt {
    pub kind: StmtKind,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum StmtKind {
    Local {
        kind: BindingKind,
        name: String,
        name_span: Span,
        ty: TypeRef,
        value: Expr,
    },
    Assign {
        target: Path,
        value: Expr,
    },
    If {
        condition: Expr,
        then_block: Block,
        else_block: Option<Block>,
    },
    Return(Expr),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Expr {
    pub kind: ExprKind,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ExprKind {
    Path(Path),
    Int(String),
    Text(String),
    Bool(bool),
    Call {
        callee: Path,
        args: Vec<Expr>,
    },
    Unary {
        op: UnaryOp,
        op_span: Span,
        expr: Box<Expr>,
    },
    Binary {
        op: BinaryOp,
        op_span: Span,
        lhs: Box<Expr>,
        rhs: Box<Expr>,
    },
}
