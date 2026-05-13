use torel_ast::{Block, Expr, Item, ProcDecl, SourceFile, Stmt, TypeRef, Visibility};

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
    pub params: Vec<HirParam>,
    pub return_type: HirTypeRef,
    pub body: HirBlock,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HirParam {
    pub name: String,
    pub ty: HirTypeRef,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HirTypeRef {
    pub path: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HirBlock {
    pub stmts: Vec<HirStmt>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum HirStmt {
    Fix {
        name: String,
        ty: HirTypeRef,
        value: HirExpr,
    },
    Return(HirExpr),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum HirExpr {
    Path(Vec<String>),
    Int(String),
    Text(String),
    Bool(bool),
    Call {
        callee: Vec<String>,
        args: Vec<HirExpr>,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HirVisibility {
    Private,
    Export,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TypedProc {
    pub id: ProcId,
    pub visibility: HirVisibility,
    pub name: String,
    pub params: Vec<TypedParam>,
    pub return_type: TypedTypeRef,
    pub body: TypedBlock,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TypedParam {
    pub id: LocalId,
    pub name: String,
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
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TypedStmt {
    Fix {
        id: LocalId,
        name: String,
        ty: TypedTypeRef,
        value: TypedExpr,
    },
    Return(TypedExpr),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TypedExpr {
    Path {
        path: Vec<String>,
        ty: TypeId,
        resolved: ResolvedValue,
    },
    Int {
        value: String,
        ty: TypeId,
    },
    Text {
        value: String,
        ty: TypeId,
    },
    Bool {
        value: bool,
        ty: TypeId,
    },
    Call {
        callee: ProcId,
        args: Vec<TypedExpr>,
        ty: TypeId,
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
        unit_path: source_file.unit.as_ref().map(|unit| unit.path.clone()),
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
        params: proc
            .params
            .iter()
            .map(|param| HirParam {
                name: param.name.clone(),
                ty: lower_type_ref(&param.ty),
            })
            .collect(),
        return_type: lower_type_ref(&proc.return_type),
        body: lower_block(&proc.body),
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
        path: ty.path.clone(),
    }
}

fn lower_block(block: &Block) -> HirBlock {
    HirBlock {
        stmts: block.stmts.iter().map(lower_stmt).collect(),
    }
}

fn lower_stmt(stmt: &Stmt) -> HirStmt {
    match stmt {
        Stmt::Fix { name, ty, value } => HirStmt::Fix {
            name: name.clone(),
            ty: lower_type_ref(ty),
            value: lower_expr(value),
        },
        Stmt::Return(expr) => HirStmt::Return(lower_expr(expr)),
    }
}

fn lower_expr(expr: &Expr) -> HirExpr {
    match expr {
        Expr::Path(path) => HirExpr::Path(path.clone()),
        Expr::Int(value) => HirExpr::Int(value.clone()),
        Expr::Text(value) => HirExpr::Text(value.clone()),
        Expr::Bool(value) => HirExpr::Bool(*value),
        Expr::Call { callee, args } => HirExpr::Call {
            callee: callee.clone(),
            args: args.iter().map(lower_expr).collect(),
        },
    }
}
