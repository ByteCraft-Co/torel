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
    pub path: Vec<String>,
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
    pub name: String,
    pub params: Vec<Param>,
    pub return_type: TypeRef,
    pub body: Block,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Param {
    pub name: String,
    pub ty: TypeRef,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TypeRef {
    pub path: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Block {
    pub stmts: Vec<Stmt>,
    pub tail: Option<Expr>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BindingKind {
    Fix,
    Slot,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Stmt {
    Local {
        kind: BindingKind,
        name: String,
        ty: TypeRef,
        value: Expr,
    },
    Assign {
        target: Vec<String>,
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
pub enum Expr {
    Path(Vec<String>),
    Int(String),
    Text(String),
    Bool(bool),
    Call {
        callee: Vec<String>,
        args: Vec<Expr>,
    },
}
