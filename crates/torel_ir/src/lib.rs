use torel_ast::{
    BindingKind, Block, Expr, ExprKind, Item, Path, ProcDecl, SourceFile, Stmt, StmtKind, TypeRef,
    Visibility,
};
use torel_diagnostics::Span;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HirModule {
    pub unit_path: Option<Vec<String>>,
    pub procs: Vec<HirProc>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TypedModule {
    pub unit_path: Option<Vec<String>>,
    pub procs: Vec<TypedProc>,
    pub symbols: SymbolCounts,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SymbolCounts {
    pub types: usize,
    pub values: usize,
    pub procs: usize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct TypeId(pub u32);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ProcId(pub u32);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ValueId(pub u32);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct LocalId(pub u32);

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HirProc {
    pub visibility: HirVisibility,
    pub name: String,
    pub name_span: Span,
    pub params: Vec<HirParam>,
    pub return_type: HirTypeRef,
    pub body: HirBlock,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HirParam {
    pub name: String,
    pub name_span: Span,
    pub ty: HirTypeRef,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HirTypeRef {
    pub path: HirPath,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HirPath {
    pub segments: Vec<String>,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HirBlock {
    pub stmts: Vec<HirStmt>,
    pub tail: Option<HirExpr>,
    pub span: Span,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HirBindingKind {
    Fix,
    Slot,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HirStmt {
    pub kind: HirStmtKind,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum HirStmtKind {
    Local {
        kind: HirBindingKind,
        name: String,
        name_span: Span,
        ty: HirTypeRef,
        value: HirExpr,
    },
    Assign {
        target: HirPath,
        value: HirExpr,
    },
    If {
        condition: HirExpr,
        then_block: HirBlock,
        else_block: Option<HirBlock>,
    },
    Return(HirExpr),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HirExpr {
    pub kind: HirExprKind,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum HirExprKind {
    Path(HirPath),
    Int(String),
    Text(String),
    Bool(bool),
    Call { callee: HirPath, args: Vec<HirExpr> },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HirVisibility {
    Private,
    Export,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Mutability {
    Immutable,
    Mutable,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TypedProc {
    pub id: ProcId,
    pub visibility: HirVisibility,
    pub name: String,
    pub name_span: Span,
    pub params: Vec<TypedParam>,
    pub return_type: TypedTypeRef,
    pub body: TypedBlock,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TypedParam {
    pub id: LocalId,
    pub name: String,
    pub name_span: Span,
    pub ty: TypedTypeRef,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TypedTypeRef {
    pub id: TypeId,
    pub display_name: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TypedBlock {
    pub stmts: Vec<TypedStmt>,
    pub tail: Option<TypedExpr>,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TypedStmt {
    pub kind: TypedStmtKind,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TypedStmtKind {
    Local {
        id: LocalId,
        mutability: Mutability,
        name: String,
        ty: TypedTypeRef,
        value: TypedExpr,
    },
    Assign {
        target: LocalId,
        value: TypedExpr,
    },
    If {
        condition: TypedExpr,
        then_block: TypedBlock,
        else_block: Option<TypedBlock>,
    },
    Return(TypedExpr),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TypedExpr {
    pub kind: TypedExprKind,
    pub ty: TypeId,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TypedExprKind {
    Path {
        path: Vec<String>,
        resolved: ResolvedValue,
    },
    Int {
        value: String,
    },
    Text {
        value: String,
    },
    Bool {
        value: bool,
    },
    Call {
        callee: ProcId,
        args: Vec<TypedExpr>,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ResolvedValue {
    BuiltinValue(ValueId),
    Proc(ProcId),
    Local(LocalId),
}

#[must_use]
pub fn lower_ast(source_file: &SourceFile) -> HirModule {
    HirModule {
        unit_path: source_file
            .unit
            .as_ref()
            .map(|unit| unit.path.segments.clone()),
        procs: source_file.items.iter().map(lower_item).collect(),
    }
}

fn lower_item(item: &Item) -> HirProc {
    match item {
        Item::Proc(proc) => lower_proc(proc),
    }
}

fn lower_proc(proc: &ProcDecl) -> HirProc {
    HirProc {
        visibility: lower_visibility(proc.visibility),
        name: proc.name.clone(),
        name_span: proc.name_span,
        params: proc
            .params
            .iter()
            .map(|param| HirParam {
                name: param.name.clone(),
                name_span: param.name_span,
                ty: lower_type_ref(&param.ty),
            })
            .collect(),
        return_type: lower_type_ref(&proc.return_type),
        body: lower_block(&proc.body),
        span: proc.span,
    }
}

fn lower_visibility(visibility: Visibility) -> HirVisibility {
    match visibility {
        Visibility::Private => HirVisibility::Private,
        Visibility::Export => HirVisibility::Export,
    }
}

fn lower_type_ref(ty: &TypeRef) -> HirTypeRef {
    HirTypeRef {
        path: lower_path(&ty.path),
    }
}

fn lower_path(path: &Path) -> HirPath {
    HirPath {
        segments: path.segments.clone(),
        span: path.span,
    }
}

fn lower_block(block: &Block) -> HirBlock {
    HirBlock {
        stmts: block.stmts.iter().map(lower_stmt).collect(),
        tail: block.tail.as_ref().map(lower_expr),
        span: block.span,
    }
}

fn lower_stmt(stmt: &Stmt) -> HirStmt {
    let kind = match &stmt.kind {
        StmtKind::Local {
            kind,
            name,
            name_span,
            ty,
            value,
        } => HirStmtKind::Local {
            kind: lower_binding_kind(*kind),
            name: name.clone(),
            name_span: *name_span,
            ty: lower_type_ref(ty),
            value: lower_expr(value),
        },
        StmtKind::Assign { target, value } => HirStmtKind::Assign {
            target: lower_path(target),
            value: lower_expr(value),
        },
        StmtKind::If {
            condition,
            then_block,
            else_block,
        } => HirStmtKind::If {
            condition: lower_expr(condition),
            then_block: lower_block(then_block),
            else_block: else_block.as_ref().map(lower_block),
        },
        StmtKind::Return(expr) => HirStmtKind::Return(lower_expr(expr)),
    };

    HirStmt {
        kind,
        span: stmt.span,
    }
}

fn lower_binding_kind(kind: BindingKind) -> HirBindingKind {
    match kind {
        BindingKind::Fix => HirBindingKind::Fix,
        BindingKind::Slot => HirBindingKind::Slot,
    }
}

fn lower_expr(expr: &Expr) -> HirExpr {
    let kind = match &expr.kind {
        ExprKind::Path(path) => HirExprKind::Path(lower_path(path)),
        ExprKind::Int(value) => HirExprKind::Int(value.clone()),
        ExprKind::Text(value) => HirExprKind::Text(value.clone()),
        ExprKind::Bool(value) => HirExprKind::Bool(*value),
        ExprKind::Call { callee, args } => HirExprKind::Call {
            callee: lower_path(callee),
            args: args.iter().map(lower_expr).collect(),
        },
    };

    HirExpr {
        kind,
        span: expr.span,
    }
}
