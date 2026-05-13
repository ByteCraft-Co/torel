use std::collections::HashMap;

use thiserror::Error;
use torel_ast::{BinaryOp, UnaryOp};
use torel_diagnostics::{Diagnostic, Label, Span};
use torel_ir::{
    HirBindingKind, HirBlock, HirExpr, HirExprKind, HirModule, HirParam, HirPath, HirProc, HirStmt,
    HirStmtKind, HirTypeRef, IntOverflowMode, LocalId, Mutability, ProcId, ResolvedValue,
    SymbolCounts, TypeId, TypedBinaryOp, TypedBlock, TypedExpr, TypedExprKind, TypedModule,
    TypedParam, TypedProc, TypedStmt, TypedStmtKind, TypedTypeRef, TypedUnaryOp, ValueId,
};

#[derive(Debug, Error)]
pub enum TypeckError {
    #[error("duplicate procedure `{name}`")]
    DuplicateProc { name: String, span: Span },

    #[error("duplicate parameter `{name}`")]
    DuplicateParam { name: String, span: Span },

    #[error("unknown type `{name}`")]
    UnknownType { name: String, span: Span },

    #[error("unknown value path `{path}`")]
    UnknownValuePath { path: String, span: Span },

    #[error("unknown local `{name}`")]
    UnknownLocal { name: String, span: Span },

    #[error("duplicate local `{name}`")]
    DuplicateLocal { name: String, span: Span },

    #[error("local `{name}` type mismatch: expected `{expected}`, found `{found}`")]
    LocalTypeMismatch {
        name: String,
        expected: String,
        found: String,
        span: Span,
    },

    #[error("cannot assign to immutable local `{name}`")]
    CannotAssignToImmutable { name: String, span: Span },

    #[error("assignment to `{name}` type mismatch: expected `{expected}`, found `{found}`")]
    AssignTypeMismatch {
        name: String,
        expected: String,
        found: String,
        span: Span,
    },

    #[error("invalid assignment target `{path}`")]
    InvalidAssignmentTarget { path: String, span: Span },

    #[error("if condition type mismatch: expected `Bool`, found `{found}`")]
    IfConditionTypeMismatch { found: String, span: Span },

    #[error("operator `{op}` cannot be applied to `{found}`")]
    UnaryOperatorTypeMismatch {
        op: &'static str,
        found: String,
        span: Span,
    },

    #[error("operator `{op}` cannot be applied to `{left}` and `{right}`")]
    BinaryOperatorTypeMismatch {
        op: &'static str,
        left: String,
        right: String,
        span: Span,
    },

    #[error("unknown procedure `{name}`")]
    UnknownProc { name: String, span: Span },

    #[error("`{path}` is not callable")]
    NotCallable { path: String, span: Span },

    #[error("procedure `{name}` used as value")]
    ProcedureUsedAsValue { name: String, span: Span },

    #[error("argument count mismatch for `{proc}`: expected {expected}, found {found}")]
    ArgCountMismatch {
        proc: String,
        expected: usize,
        found: usize,
        span: Span,
    },

    #[error(
        "argument type mismatch for `{proc}` argument {index}: expected `{expected}`, found `{found}`"
    )]
    ArgTypeMismatch {
        proc: String,
        index: usize,
        expected: String,
        found: String,
        span: Span,
    },

    #[error("return type mismatch: expected `{expected}`, found `{found}`")]
    ReturnTypeMismatch {
        expected: String,
        found: String,
        span: Span,
    },

    #[error("missing return from procedure `{proc}`: expected `{expected}`")]
    MissingReturn {
        proc: String,
        expected: String,
        span: Span,
    },
}

impl TypeckError {
    #[must_use]
    pub fn to_diagnostic(&self) -> Diagnostic {
        let label_message = match self {
            Self::UnknownType { .. } => "unknown type",
            Self::UnknownValuePath { .. } => "unknown value path",
            Self::UnknownLocal { .. } => "unknown local",
            Self::DuplicateProc { .. } => "duplicate procedure",
            Self::DuplicateParam { .. } => "duplicate parameter",
            Self::DuplicateLocal { .. } => "duplicate local",
            Self::LocalTypeMismatch { .. } => "initializer has this type",
            Self::CannotAssignToImmutable { .. } => "immutable local",
            Self::AssignTypeMismatch { .. } => "assigned value has this type",
            Self::InvalidAssignmentTarget { .. } => "invalid assignment target",
            Self::IfConditionTypeMismatch { .. } => "condition has this type",
            Self::UnaryOperatorTypeMismatch { .. } | Self::BinaryOperatorTypeMismatch { .. } => {
                "operator cannot be applied here"
            }
            Self::UnknownProc { .. } => "unknown procedure",
            Self::NotCallable { .. } => "not callable",
            Self::ProcedureUsedAsValue { .. } => "procedure used as value",
            Self::ArgCountMismatch { .. } => "wrong number of arguments",
            Self::ArgTypeMismatch { .. } => "argument has this type",
            Self::ReturnTypeMismatch { .. } => "value has this type",
            Self::MissingReturn { .. } => "procedure body may continue without returning",
        };

        Diagnostic::error(self.to_string()).with_label(Label::primary(self.span(), label_message))
    }

    fn span(&self) -> Span {
        match self {
            Self::DuplicateProc { span, .. }
            | Self::DuplicateParam { span, .. }
            | Self::UnknownType { span, .. }
            | Self::UnknownValuePath { span, .. }
            | Self::UnknownLocal { span, .. }
            | Self::DuplicateLocal { span, .. }
            | Self::LocalTypeMismatch { span, .. }
            | Self::CannotAssignToImmutable { span, .. }
            | Self::AssignTypeMismatch { span, .. }
            | Self::InvalidAssignmentTarget { span, .. }
            | Self::IfConditionTypeMismatch { span, .. }
            | Self::UnaryOperatorTypeMismatch { span, .. }
            | Self::BinaryOperatorTypeMismatch { span, .. }
            | Self::UnknownProc { span, .. }
            | Self::NotCallable { span, .. }
            | Self::ProcedureUsedAsValue { span, .. }
            | Self::ArgCountMismatch { span, .. }
            | Self::ArgTypeMismatch { span, .. }
            | Self::ReturnTypeMismatch { span, .. }
            | Self::MissingReturn { span, .. } => *span,
        }
    }
}

#[derive(Debug, Clone)]
pub struct SymbolTable {
    types: HashMap<String, TypeId>,
    values: HashMap<String, ValueSymbol>,
    procs: HashMap<String, ProcSymbol>,
    type_names: Vec<String>,
}

#[derive(Debug, Clone, Copy)]
struct ValueSymbol {
    id: ValueId,
    ty: TypeId,
}

#[derive(Debug, Clone)]
struct ProcSymbol {
    id: ProcId,
    params: Vec<TypeId>,
    return_type: TypeId,
}

#[derive(Debug, Clone, Copy)]
struct LocalSymbol {
    id: LocalId,
    ty: TypeId,
    mutability: Mutability,
}

struct LocalDecl<'hir> {
    kind: HirBindingKind,
    name: &'hir str,
    name_span: Span,
    ty: &'hir HirTypeRef,
    value: &'hir HirExpr,
    stmt_span: Span,
}

struct IfStmt<'hir> {
    condition: &'hir HirExpr,
    then_block: &'hir HirBlock,
    else_block: Option<&'hir HirBlock>,
    stmt_span: Span,
}

impl SymbolTable {
    #[must_use]
    pub fn with_builtins() -> Self {
        let mut table = Self {
            types: HashMap::new(),
            values: HashMap::new(),
            procs: HashMap::new(),
            type_names: Vec::new(),
        };

        let exit = table.insert_type("Exit");
        table.insert_type("Void");
        table.insert_type("Bool");
        table.insert_type("Int32");
        table.insert_type("UInt64");
        table.insert_type("Text");
        table.insert_type("Never");
        table.insert_value("Exit.ok", exit);

        table
    }

    fn insert_type(&mut self, name: &str) -> TypeId {
        let id = TypeId(self.type_names.len() as u32);
        self.types.insert(name.to_owned(), id);
        self.type_names.push(name.to_owned());
        id
    }

    fn insert_value(&mut self, path: &str, ty: TypeId) -> ValueId {
        let id = ValueId(self.values.len() as u32);
        self.values.insert(path.to_owned(), ValueSymbol { id, ty });
        id
    }

    fn insert_proc(
        &mut self,
        name: &str,
        span: Span,
        params: Vec<TypeId>,
        return_type: TypeId,
    ) -> Result<ProcId, TypeckError> {
        if self.procs.contains_key(name) {
            return Err(TypeckError::DuplicateProc {
                name: name.to_owned(),
                span,
            });
        }

        let id = ProcId(self.procs.len() as u32);
        self.procs.insert(
            name.to_owned(),
            ProcSymbol {
                id,
                params,
                return_type,
            },
        );
        Ok(id)
    }

    fn resolve_type(&self, ty: &HirTypeRef) -> Result<TypedTypeRef, TypeckError> {
        let name = join_path(&ty.path.segments);
        let id = self
            .types
            .get(&name)
            .copied()
            .ok_or_else(|| TypeckError::UnknownType {
                name: name.clone(),
                span: ty.path.span,
            })?;

        Ok(TypedTypeRef {
            id,
            display_name: name,
        })
    }

    fn type_name(&self, id: TypeId) -> String {
        self.type_names
            .get(id.0 as usize)
            .cloned()
            .unwrap_or_else(|| format!("<type {}>", id.0))
    }

    fn builtin_type(&self, name: &str) -> TypeId {
        self.types
            .get(name)
            .copied()
            .unwrap_or_else(|| panic!("builtin type `{name}` should exist"))
    }

    fn symbol_counts(&self) -> SymbolCounts {
        SymbolCounts {
            types: self.types.len(),
            values: self.values.len(),
            procs: self.procs.len(),
        }
    }
}

pub fn check_types(hir: &HirModule) -> Result<TypedModule, TypeckError> {
    let mut symbols = SymbolTable::with_builtins();

    for proc in &hir.procs {
        let params = proc
            .params
            .iter()
            .map(|param| symbols.resolve_type(&param.ty).map(|ty| ty.id))
            .collect::<Result<Vec<_>, _>>()?;
        let return_type = symbols.resolve_type(&proc.return_type)?.id;

        symbols.insert_proc(&proc.name, proc.name_span, params, return_type)?;
    }

    let procs = hir
        .procs
        .iter()
        .map(|proc| check_proc(&symbols, proc))
        .collect::<Result<Vec<_>, _>>()?;

    Ok(TypedModule {
        unit_path: hir.unit_path.clone(),
        procs,
        symbols: symbols.symbol_counts(),
    })
}

fn check_proc(symbols: &SymbolTable, proc: &HirProc) -> Result<TypedProc, TypeckError> {
    let id = symbols
        .procs
        .get(&proc.name)
        .map(|proc| proc.id)
        .expect("procedure should be predeclared");
    let params = check_params(symbols, &proc.params)?;
    let mut locals = local_map(&params)?;
    let mut next_local_id = params.len() as u32;
    let return_type = symbols.resolve_type(&proc.return_type)?;
    let body = check_block(
        symbols,
        &mut locals,
        &mut next_local_id,
        &proc.body,
        &return_type,
    )?;

    check_return_flow(proc, &return_type, &body)?;

    Ok(TypedProc {
        id,
        visibility: proc.visibility,
        name: proc.name.clone(),
        name_span: proc.name_span,
        params,
        return_type,
        body,
        span: proc.span,
    })
}

fn check_params(
    symbols: &SymbolTable,
    params: &[HirParam],
) -> Result<Vec<TypedParam>, TypeckError> {
    params
        .iter()
        .enumerate()
        .map(|(index, param)| {
            Ok(TypedParam {
                id: LocalId(index as u32),
                name: param.name.clone(),
                name_span: param.name_span,
                ty: symbols.resolve_type(&param.ty)?,
            })
        })
        .collect()
}

fn local_map(params: &[TypedParam]) -> Result<HashMap<String, LocalSymbol>, TypeckError> {
    let mut locals = HashMap::new();

    for param in params {
        if locals
            .insert(
                param.name.clone(),
                LocalSymbol {
                    id: param.id,
                    ty: param.ty.id,
                    mutability: Mutability::Immutable,
                },
            )
            .is_some()
        {
            return Err(TypeckError::DuplicateParam {
                name: param.name.clone(),
                span: param.name_span,
            });
        }
    }

    Ok(locals)
}

fn check_block(
    symbols: &SymbolTable,
    locals: &mut HashMap<String, LocalSymbol>,
    next_local_id: &mut u32,
    block: &HirBlock,
    expected_return: &TypedTypeRef,
) -> Result<TypedBlock, TypeckError> {
    let mut stmts = Vec::new();

    for stmt in &block.stmts {
        stmts.push(check_stmt(
            symbols,
            locals,
            next_local_id,
            stmt,
            expected_return,
        )?);
    }

    let tail = block
        .tail
        .as_ref()
        .map(|expr| check_tail_expr(symbols, locals, expected_return, expr))
        .transpose()?;

    Ok(TypedBlock {
        stmts,
        tail,
        span: block.span,
    })
}

fn check_stmt(
    symbols: &SymbolTable,
    locals: &mut HashMap<String, LocalSymbol>,
    next_local_id: &mut u32,
    stmt: &HirStmt,
    expected_return: &TypedTypeRef,
) -> Result<TypedStmt, TypeckError> {
    match &stmt.kind {
        HirStmtKind::Local {
            kind,
            name,
            name_span,
            ty,
            value,
        } => check_local_stmt(
            symbols,
            locals,
            next_local_id,
            LocalDecl {
                kind: *kind,
                name,
                name_span: *name_span,
                ty,
                value,
                stmt_span: stmt.span,
            },
        ),
        HirStmtKind::Assign { target, value } => {
            check_assign_stmt(symbols, locals, target, value, stmt.span)
        }
        HirStmtKind::If {
            condition,
            then_block,
            else_block,
        } => check_if_stmt(
            symbols,
            locals,
            next_local_id,
            expected_return,
            IfStmt {
                condition,
                then_block,
                else_block: else_block.as_ref(),
                stmt_span: stmt.span,
            },
        ),
        HirStmtKind::Return(expr) => {
            check_return_stmt(symbols, locals, expected_return, expr, stmt.span)
        }
    }
}

fn check_local_stmt(
    symbols: &SymbolTable,
    locals: &mut HashMap<String, LocalSymbol>,
    next_local_id: &mut u32,
    decl: LocalDecl<'_>,
) -> Result<TypedStmt, TypeckError> {
    if locals.contains_key(decl.name) {
        return Err(TypeckError::DuplicateLocal {
            name: decl.name.to_owned(),
            span: decl.name_span,
        });
    }

    let ty = symbols.resolve_type(decl.ty)?;
    let value = check_expr(symbols, locals, decl.value)?;
    let found = expr_type(&value);

    if found != ty.id {
        return Err(TypeckError::LocalTypeMismatch {
            name: decl.name.to_owned(),
            expected: ty.display_name.clone(),
            found: symbols.type_name(found),
            span: value.span,
        });
    }

    let id = LocalId(*next_local_id);
    *next_local_id += 1;
    let mutability = local_mutability(decl.kind);
    locals.insert(
        decl.name.to_owned(),
        LocalSymbol {
            id,
            ty: ty.id,
            mutability,
        },
    );

    Ok(TypedStmt {
        kind: TypedStmtKind::Local {
            id,
            mutability,
            name: decl.name.to_owned(),
            ty,
            value,
        },
        span: decl.stmt_span,
    })
}

fn check_assign_stmt(
    symbols: &SymbolTable,
    locals: &mut HashMap<String, LocalSymbol>,
    target: &HirPath,
    value: &HirExpr,
    stmt_span: Span,
) -> Result<TypedStmt, TypeckError> {
    if target.segments.len() != 1 {
        return Err(TypeckError::InvalidAssignmentTarget {
            path: join_path(&target.segments),
            span: target.span,
        });
    }

    let name = &target.segments[0];
    let Some(local) = locals.get(name).copied() else {
        return Err(TypeckError::UnknownLocal {
            name: name.clone(),
            span: target.span,
        });
    };

    if local.mutability == Mutability::Immutable {
        return Err(TypeckError::CannotAssignToImmutable {
            name: name.clone(),
            span: target.span,
        });
    }

    let value = check_expr(symbols, locals, value)?;
    let found = expr_type(&value);

    if found != local.ty {
        return Err(TypeckError::AssignTypeMismatch {
            name: name.clone(),
            expected: symbols.type_name(local.ty),
            found: symbols.type_name(found),
            span: value.span,
        });
    }

    Ok(TypedStmt {
        kind: TypedStmtKind::Assign {
            target: local.id,
            value,
        },
        span: stmt_span,
    })
}

fn check_if_stmt(
    symbols: &SymbolTable,
    locals: &mut HashMap<String, LocalSymbol>,
    next_local_id: &mut u32,
    expected_return: &TypedTypeRef,
    stmt: IfStmt<'_>,
) -> Result<TypedStmt, TypeckError> {
    let condition = check_expr(symbols, locals, stmt.condition)?;
    let condition_type = expr_type(&condition);
    let bool_type = symbols.builtin_type("Bool");

    if condition_type != bool_type {
        return Err(TypeckError::IfConditionTypeMismatch {
            found: symbols.type_name(condition_type),
            span: condition.span,
        });
    }

    let mut then_locals = locals.clone();
    let then_block = check_block(
        symbols,
        &mut then_locals,
        next_local_id,
        stmt.then_block,
        expected_return,
    )?;
    let else_block = stmt
        .else_block
        .map(|else_block| {
            let mut else_locals = locals.clone();
            check_block(
                symbols,
                &mut else_locals,
                next_local_id,
                else_block,
                expected_return,
            )
        })
        .transpose()?;

    Ok(TypedStmt {
        kind: TypedStmtKind::If {
            condition,
            then_block,
            else_block,
        },
        span: stmt.stmt_span,
    })
}

fn check_return_stmt(
    symbols: &SymbolTable,
    locals: &HashMap<String, LocalSymbol>,
    expected: &TypedTypeRef,
    expr: &HirExpr,
    stmt_span: Span,
) -> Result<TypedStmt, TypeckError> {
    let expr = check_expr(symbols, locals, expr)?;
    check_return_type(symbols, expected, &expr)?;

    Ok(TypedStmt {
        kind: TypedStmtKind::Return(expr),
        span: stmt_span,
    })
}

fn check_tail_expr(
    symbols: &SymbolTable,
    locals: &HashMap<String, LocalSymbol>,
    expected: &TypedTypeRef,
    expr: &HirExpr,
) -> Result<TypedExpr, TypeckError> {
    let expr = check_expr(symbols, locals, expr)?;
    check_return_type(symbols, expected, &expr)?;
    Ok(expr)
}

fn check_return_type(
    symbols: &SymbolTable,
    expected: &TypedTypeRef,
    expr: &TypedExpr,
) -> Result<(), TypeckError> {
    let found = expr_type(expr);

    if found != expected.id {
        return Err(TypeckError::ReturnTypeMismatch {
            expected: expected.display_name.clone(),
            found: symbols.type_name(found),
            span: expr.span,
        });
    }

    Ok(())
}

fn local_mutability(kind: HirBindingKind) -> Mutability {
    match kind {
        HirBindingKind::Fix => Mutability::Immutable,
        HirBindingKind::Slot => Mutability::Mutable,
    }
}

fn check_expr(
    symbols: &SymbolTable,
    locals: &HashMap<String, LocalSymbol>,
    expr: &HirExpr,
) -> Result<TypedExpr, TypeckError> {
    match &expr.kind {
        HirExprKind::Path(path) => check_path_expr(symbols, locals, path, expr.span),
        HirExprKind::Int(value) => Ok(TypedExpr {
            kind: TypedExprKind::Int {
                value: value.clone(),
            },
            ty: symbols.builtin_type("Int32"),
            span: expr.span,
        }),
        HirExprKind::Text(value) => Ok(TypedExpr {
            kind: TypedExprKind::Text {
                value: value.clone(),
            },
            ty: symbols.builtin_type("Text"),
            span: expr.span,
        }),
        HirExprKind::Bool(value) => Ok(TypedExpr {
            kind: TypedExprKind::Bool { value: *value },
            ty: symbols.builtin_type("Bool"),
            span: expr.span,
        }),
        HirExprKind::Call { callee, args } => {
            check_call_expr(symbols, locals, callee, args, expr.span)
        }
        HirExprKind::Unary { op, op_span, expr } => {
            check_unary_expr(symbols, locals, *op, *op_span, expr, expr.span)
        }
        HirExprKind::Binary {
            op,
            op_span,
            lhs,
            rhs,
        } => check_binary_expr(symbols, locals, *op, *op_span, lhs, rhs, expr.span),
    }
}

fn check_unary_expr(
    symbols: &SymbolTable,
    locals: &HashMap<String, LocalSymbol>,
    op: UnaryOp,
    op_span: Span,
    expr: &HirExpr,
    span: Span,
) -> Result<TypedExpr, TypeckError> {
    let expr = check_expr(symbols, locals, expr)?;
    let found = expr.ty;
    let int32 = symbols.builtin_type("Int32");
    let bool_type = symbols.builtin_type("Bool");

    let (typed_op, ty) = match (op, found) {
        (UnaryOp::Not, found) if found == bool_type => (TypedUnaryOp::BoolNot, bool_type),
        (UnaryOp::Neg, found) if found == int32 => (
            TypedUnaryOp::IntNeg {
                overflow: IntOverflowMode::Checked,
            },
            int32,
        ),
        _ => {
            return Err(TypeckError::UnaryOperatorTypeMismatch {
                op: op.symbol(),
                found: symbols.type_name(found),
                span: op_span,
            });
        }
    };

    Ok(TypedExpr {
        kind: TypedExprKind::Unary {
            op: typed_op,
            expr: Box::new(expr),
        },
        ty,
        span,
    })
}

fn check_binary_expr(
    symbols: &SymbolTable,
    locals: &HashMap<String, LocalSymbol>,
    op: BinaryOp,
    op_span: Span,
    lhs: &HirExpr,
    rhs: &HirExpr,
    span: Span,
) -> Result<TypedExpr, TypeckError> {
    let lhs = check_expr(symbols, locals, lhs)?;
    let rhs = check_expr(symbols, locals, rhs)?;
    let left = lhs.ty;
    let right = rhs.ty;
    let int32 = symbols.builtin_type("Int32");
    let bool_type = symbols.builtin_type("Bool");

    let (typed_op, ty) = match op {
        BinaryOp::Add if left == int32 && right == int32 => (
            TypedBinaryOp::IntAdd {
                overflow: IntOverflowMode::Checked,
            },
            int32,
        ),
        BinaryOp::Sub if left == int32 && right == int32 => (
            TypedBinaryOp::IntSub {
                overflow: IntOverflowMode::Checked,
            },
            int32,
        ),
        BinaryOp::Mul if left == int32 && right == int32 => (
            TypedBinaryOp::IntMul {
                overflow: IntOverflowMode::Checked,
            },
            int32,
        ),
        BinaryOp::Div if left == int32 && right == int32 => (TypedBinaryOp::IntDiv, int32),
        BinaryOp::Rem if left == int32 && right == int32 => (TypedBinaryOp::IntRem, int32),
        BinaryOp::Lt if left == int32 && right == int32 => (TypedBinaryOp::IntLt, bool_type),
        BinaryOp::LtEq if left == int32 && right == int32 => (TypedBinaryOp::IntLtEq, bool_type),
        BinaryOp::Gt if left == int32 && right == int32 => (TypedBinaryOp::IntGt, bool_type),
        BinaryOp::GtEq if left == int32 && right == int32 => (TypedBinaryOp::IntGtEq, bool_type),
        BinaryOp::Eq if left == int32 && right == int32 => (TypedBinaryOp::IntEq, bool_type),
        BinaryOp::NotEq if left == int32 && right == int32 => (TypedBinaryOp::IntNotEq, bool_type),
        BinaryOp::Eq if left == right => (TypedBinaryOp::SameTypeEq, bool_type),
        BinaryOp::NotEq if left == right => (TypedBinaryOp::SameTypeNotEq, bool_type),
        BinaryOp::And if left == bool_type && right == bool_type => {
            (TypedBinaryOp::BoolAnd, bool_type)
        }
        BinaryOp::Or if left == bool_type && right == bool_type => {
            (TypedBinaryOp::BoolOr, bool_type)
        }
        _ => {
            return Err(TypeckError::BinaryOperatorTypeMismatch {
                op: op.symbol(),
                left: symbols.type_name(left),
                right: symbols.type_name(right),
                span: op_span,
            });
        }
    };

    Ok(TypedExpr {
        kind: TypedExprKind::Binary {
            op: typed_op,
            lhs: Box::new(lhs),
            rhs: Box::new(rhs),
        },
        ty,
        span,
    })
}

fn check_path_expr(
    symbols: &SymbolTable,
    locals: &HashMap<String, LocalSymbol>,
    path: &HirPath,
    span: Span,
) -> Result<TypedExpr, TypeckError> {
    let joined = join_path(&path.segments);

    if path.segments.len() == 1 {
        if let Some(local) = locals.get(&joined) {
            return Ok(TypedExpr {
                kind: TypedExprKind::Path {
                    path: path.segments.clone(),
                    resolved: ResolvedValue::Local(local.id),
                },
                ty: local.ty,
                span,
            });
        }

        if symbols.procs.contains_key(&joined) {
            return Err(TypeckError::ProcedureUsedAsValue {
                name: joined,
                span: path.span,
            });
        }

        return Err(TypeckError::UnknownLocal {
            name: joined,
            span: path.span,
        });
    }

    if let Some(value) = symbols.values.get(&joined) {
        return Ok(TypedExpr {
            kind: TypedExprKind::Path {
                path: path.segments.clone(),
                resolved: ResolvedValue::BuiltinValue(value.id),
            },
            ty: value.ty,
            span,
        });
    }

    Err(TypeckError::UnknownValuePath {
        path: joined,
        span: path.span,
    })
}

fn check_call_expr(
    symbols: &SymbolTable,
    locals: &HashMap<String, LocalSymbol>,
    callee: &HirPath,
    args: &[HirExpr],
    span: Span,
) -> Result<TypedExpr, TypeckError> {
    let name = join_path(&callee.segments);

    if symbols.values.contains_key(&name) || locals.contains_key(&name) {
        return Err(TypeckError::NotCallable {
            path: name,
            span: callee.span,
        });
    }

    let Some(proc) = symbols.procs.get(&name) else {
        return Err(TypeckError::UnknownProc {
            name,
            span: callee.span,
        });
    };

    if proc.params.len() != args.len() {
        return Err(TypeckError::ArgCountMismatch {
            proc: name,
            expected: proc.params.len(),
            found: args.len(),
            span,
        });
    }

    let mut typed_args = Vec::new();

    for (index, (arg, expected)) in args.iter().zip(&proc.params).enumerate() {
        let typed = check_expr(symbols, locals, arg)?;
        let found = expr_type(&typed);

        if found != *expected {
            return Err(TypeckError::ArgTypeMismatch {
                proc: name,
                index: index + 1,
                expected: symbols.type_name(*expected),
                found: symbols.type_name(found),
                span: arg.span,
            });
        }

        typed_args.push(typed);
    }

    Ok(TypedExpr {
        kind: TypedExprKind::Call {
            callee: proc.id,
            args: typed_args,
        },
        ty: proc.return_type,
        span,
    })
}

fn check_return_flow(
    proc: &HirProc,
    expected: &TypedTypeRef,
    body: &TypedBlock,
) -> Result<(), TypeckError> {
    if expected.display_name != "Void" && !block_flow(body).satisfies_return(expected.id) {
        return Err(TypeckError::MissingReturn {
            proc: proc.name.clone(),
            expected: expected.display_name.clone(),
            span: proc.name_span,
        });
    }

    Ok(())
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Flow {
    MayContinue,
    AlwaysReturns,
    CompletesWithValue(TypeId),
}

impl Flow {
    fn satisfies_return(self, expected: TypeId) -> bool {
        match self {
            Self::MayContinue => false,
            Self::AlwaysReturns => true,
            Self::CompletesWithValue(found) => found == expected,
        }
    }
}

fn block_flow(block: &TypedBlock) -> Flow {
    for (index, stmt) in block.stmts.iter().enumerate() {
        match stmt_flow(stmt) {
            Flow::MayContinue => {}
            Flow::AlwaysReturns => return Flow::AlwaysReturns,
            flow @ Flow::CompletesWithValue(_) if index + 1 == block.stmts.len() => return flow,
            Flow::CompletesWithValue(_) => {}
        }
    }

    if let Some(tail) = &block.tail {
        return Flow::CompletesWithValue(expr_type(tail));
    }

    Flow::MayContinue
}

fn stmt_flow(stmt: &TypedStmt) -> Flow {
    match &stmt.kind {
        TypedStmtKind::Return(_) => Flow::AlwaysReturns,
        TypedStmtKind::If {
            then_block,
            else_block: Some(else_block),
            ..
        } => combine_branch_flow(block_flow(then_block), block_flow(else_block)),
        TypedStmtKind::Local { .. } | TypedStmtKind::Assign { .. } | TypedStmtKind::If { .. } => {
            Flow::MayContinue
        }
    }
}

fn combine_branch_flow(then_flow: Flow, else_flow: Flow) -> Flow {
    match (then_flow, else_flow) {
        (Flow::AlwaysReturns, Flow::AlwaysReturns) => Flow::AlwaysReturns,
        (Flow::CompletesWithValue(then_ty), Flow::CompletesWithValue(else_ty))
            if then_ty == else_ty =>
        {
            Flow::CompletesWithValue(then_ty)
        }
        (Flow::AlwaysReturns, Flow::CompletesWithValue(ty))
        | (Flow::CompletesWithValue(ty), Flow::AlwaysReturns) => Flow::CompletesWithValue(ty),
        _ => Flow::MayContinue,
    }
}

fn expr_type(expr: &TypedExpr) -> TypeId {
    expr.ty
}

fn join_path(path: &[String]) -> String {
    path.join(".")
}

#[cfg(test)]
mod tests {
    use super::*;
    use torel_ir::{
        HirBlock, HirExpr, HirExprKind, HirPath, HirProc, HirStmt, HirStmtKind, HirTypeRef,
        HirVisibility, lower_ast,
    };

    #[test]
    fn resolves_builtin_exit_value() {
        let span = Span::new(0, 1);
        let hir = HirModule {
            unit_path: Some(vec!["examples".to_owned(), "hello".to_owned()]),
            procs: vec![HirProc {
                visibility: HirVisibility::Export,
                name: "main".to_owned(),
                name_span: span,
                params: Vec::new(),
                return_type: HirTypeRef {
                    path: HirPath {
                        segments: vec!["Exit".to_owned()],
                        span,
                    },
                },
                body: HirBlock {
                    stmts: vec![HirStmt {
                        kind: HirStmtKind::Return(HirExpr {
                            kind: HirExprKind::Path(HirPath {
                                segments: vec!["Exit".to_owned(), "ok".to_owned()],
                                span,
                            }),
                            span,
                        }),
                        span,
                    }],
                    tail: None,
                    span,
                },
                span,
            }],
        };

        let typed = check_types(&hir).expect("type checking should pass");

        assert_eq!(typed.unit_path, hir.unit_path);
        assert_eq!(typed.symbols.types, 7);
        assert_eq!(typed.symbols.values, 1);
        assert_eq!(typed.symbols.procs, 1);
        assert_eq!(typed.procs[0].return_type.display_name, "Exit");
    }

    #[test]
    fn checks_int_add_type() {
        let expr = checked_tail("Int32", "1 + 2");
        assert_eq!(expr.ty, SymbolTable::with_builtins().builtin_type("Int32"));
        assert!(matches!(
            expr.kind,
            TypedExprKind::Binary {
                op: TypedBinaryOp::IntAdd { .. },
                ..
            }
        ));
    }

    #[test]
    fn checks_int_comparison_type() {
        let expr = checked_tail("Bool", "1 < 2");
        assert_eq!(expr.ty, SymbolTable::with_builtins().builtin_type("Bool"));
        assert!(matches!(
            expr.kind,
            TypedExprKind::Binary {
                op: TypedBinaryOp::IntLt,
                ..
            }
        ));
    }

    #[test]
    fn checks_bool_and_type() {
        let expr = checked_tail("Bool", "true && false");
        assert_eq!(expr.ty, SymbolTable::with_builtins().builtin_type("Bool"));
        assert!(matches!(
            expr.kind,
            TypedExprKind::Binary {
                op: TypedBinaryOp::BoolAnd,
                ..
            }
        ));
    }

    #[test]
    fn checks_unary_types() {
        let not_expr = checked_tail("Bool", "!false");
        assert!(matches!(
            not_expr.kind,
            TypedExprKind::Unary {
                op: TypedUnaryOp::BoolNot,
                ..
            }
        ));

        let neg_expr = checked_tail("Int32", "-42");
        assert!(matches!(
            neg_expr.kind,
            TypedExprKind::Unary {
                op: TypedUnaryOp::IntNeg { .. },
                ..
            }
        ));
    }

    #[test]
    fn rejects_mismatched_binary_operands() {
        let err = check_source("Bool", "1 == true").expect_err("mismatch should fail");

        assert!(matches!(
            err,
            TypeckError::BinaryOperatorTypeMismatch {
                op: "==",
                left,
                right,
                ..
            } if left == "Int32" && right == "Bool"
        ));
    }

    fn checked_tail(return_type: &str, body: &str) -> TypedExpr {
        let typed = check_source(return_type, body).expect("type checking should pass");
        typed.procs[0]
            .body
            .tail
            .clone()
            .expect("tail expression should exist")
    }

    fn check_source(return_type: &str, body: &str) -> Result<TypedModule, TypeckError> {
        let source = format!("unit tests.ops; export proc main() -> {return_type} {{ {body} }}");
        let tokens = torel_lexer::lex(&source);
        let ast = torel_parse::parse_source_file(&tokens).expect("source should parse");
        let hir = lower_ast(&ast);
        check_types(&hir)
    }
}
