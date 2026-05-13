use std::collections::HashMap;

use thiserror::Error;
use torel_ir::{
    HirBindingKind, HirBlock, HirExpr, HirModule, HirParam, HirProc, HirStmt, HirTypeRef, LocalId,
    Mutability, ProcId, ResolvedValue, SymbolCounts, TypeId, TypedBlock, TypedExpr, TypedModule,
    TypedParam, TypedProc, TypedStmt, TypedTypeRef, ValueId,
};

#[derive(Debug, Error)]
pub enum TypeckError {
    #[error("duplicate procedure `{name}`")]
    DuplicateProc { name: String },

    #[error("duplicate parameter `{name}`")]
    DuplicateParam { name: String },

    #[error("unknown type `{name}`")]
    UnknownType { name: String },

    #[error("unknown value path `{path}`")]
    UnknownValuePath { path: String },

    #[error("unknown local `{name}`")]
    UnknownLocal { name: String },

    #[error("duplicate local `{name}`")]
    DuplicateLocal { name: String },

    #[error("local `{name}` type mismatch: expected `{expected}`, found `{found}`")]
    LocalTypeMismatch {
        name: String,
        expected: String,
        found: String,
    },

    #[error("cannot assign to immutable local `{name}`")]
    CannotAssignToImmutable { name: String },

    #[error("assignment to `{name}` type mismatch: expected `{expected}`, found `{found}`")]
    AssignTypeMismatch {
        name: String,
        expected: String,
        found: String,
    },

    #[error("invalid assignment target `{path}`")]
    InvalidAssignmentTarget { path: String },

    #[error("if condition type mismatch: expected `Bool`, found `{found}`")]
    IfConditionTypeMismatch { found: String },

    #[error("unknown procedure `{name}`")]
    UnknownProc { name: String },

    #[error("`{path}` is not callable")]
    NotCallable { path: String },

    #[error("procedure `{name}` used as value")]
    ProcedureUsedAsValue { name: String },

    #[error("argument count mismatch for `{proc}`: expected {expected}, found {found}")]
    ArgCountMismatch {
        proc: String,
        expected: usize,
        found: usize,
    },

    #[error(
        "argument type mismatch for `{proc}` argument {index}: expected `{expected}`, found `{found}`"
    )]
    ArgTypeMismatch {
        proc: String,
        index: usize,
        expected: String,
        found: String,
    },

    #[error("return type mismatch: expected `{expected}`, found `{found}`")]
    ReturnTypeMismatch { expected: String, found: String },

    #[error("missing return from procedure `{proc}`: expected `{expected}`")]
    MissingReturn { proc: String, expected: String },
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
        params: Vec<TypeId>,
        return_type: TypeId,
    ) -> Result<ProcId, TypeckError> {
        if self.procs.contains_key(name) {
            return Err(TypeckError::DuplicateProc {
                name: name.to_owned(),
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
        let name = join_path(&ty.path);
        let id = self
            .types
            .get(&name)
            .copied()
            .ok_or_else(|| TypeckError::UnknownType { name: name.clone() })?;

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

        symbols.insert_proc(&proc.name, params, return_type)?;
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
        params,
        return_type,
        body,
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

    Ok(TypedBlock { stmts, tail })
}

fn check_stmt(
    symbols: &SymbolTable,
    locals: &mut HashMap<String, LocalSymbol>,
    next_local_id: &mut u32,
    stmt: &HirStmt,
    expected_return: &TypedTypeRef,
) -> Result<TypedStmt, TypeckError> {
    match stmt {
        HirStmt::Local {
            kind,
            name,
            ty,
            value,
        } => check_local_stmt(symbols, locals, next_local_id, *kind, name, ty, value),
        HirStmt::Assign { target, value } => check_assign_stmt(symbols, locals, target, value),
        HirStmt::If {
            condition,
            then_block,
            else_block,
        } => check_if_stmt(
            symbols,
            locals,
            next_local_id,
            condition,
            then_block,
            else_block.as_ref(),
            expected_return,
        ),
        HirStmt::Return(expr) => check_return_stmt(symbols, locals, expected_return, expr),
    }
}

fn check_local_stmt(
    symbols: &SymbolTable,
    locals: &mut HashMap<String, LocalSymbol>,
    next_local_id: &mut u32,
    kind: HirBindingKind,
    name: &str,
    ty: &HirTypeRef,
    value: &HirExpr,
) -> Result<TypedStmt, TypeckError> {
    if locals.contains_key(name) {
        return Err(TypeckError::DuplicateLocal {
            name: name.to_owned(),
        });
    }

    let ty = symbols.resolve_type(ty)?;
    let value = check_expr(symbols, locals, value)?;
    let found = expr_type(&value);

    if found != ty.id {
        return Err(TypeckError::LocalTypeMismatch {
            name: name.to_owned(),
            expected: ty.display_name.clone(),
            found: symbols.type_name(found),
        });
    }

    let id = LocalId(*next_local_id);
    *next_local_id += 1;
    let mutability = local_mutability(kind);
    locals.insert(
        name.to_owned(),
        LocalSymbol {
            id,
            ty: ty.id,
            mutability,
        },
    );

    Ok(TypedStmt::Local {
        id,
        mutability,
        name: name.to_owned(),
        ty,
        value,
    })
}

fn check_assign_stmt(
    symbols: &SymbolTable,
    locals: &mut HashMap<String, LocalSymbol>,
    target: &[String],
    value: &HirExpr,
) -> Result<TypedStmt, TypeckError> {
    if target.len() != 1 {
        return Err(TypeckError::InvalidAssignmentTarget {
            path: join_path(target),
        });
    }

    let name = &target[0];
    let Some(local) = locals.get(name).copied() else {
        return Err(TypeckError::UnknownLocal { name: name.clone() });
    };

    if local.mutability == Mutability::Immutable {
        return Err(TypeckError::CannotAssignToImmutable { name: name.clone() });
    }

    let value = check_expr(symbols, locals, value)?;
    let found = expr_type(&value);

    if found != local.ty {
        return Err(TypeckError::AssignTypeMismatch {
            name: name.clone(),
            expected: symbols.type_name(local.ty),
            found: symbols.type_name(found),
        });
    }

    Ok(TypedStmt::Assign {
        target: local.id,
        value,
    })
}

fn check_if_stmt(
    symbols: &SymbolTable,
    locals: &mut HashMap<String, LocalSymbol>,
    next_local_id: &mut u32,
    condition: &HirExpr,
    then_block: &HirBlock,
    else_block: Option<&HirBlock>,
    expected_return: &TypedTypeRef,
) -> Result<TypedStmt, TypeckError> {
    let condition = check_expr(symbols, locals, condition)?;
    let condition_type = expr_type(&condition);
    let bool_type = symbols.builtin_type("Bool");

    if condition_type != bool_type {
        return Err(TypeckError::IfConditionTypeMismatch {
            found: symbols.type_name(condition_type),
        });
    }

    let mut then_locals = locals.clone();
    let then_block = check_block(
        symbols,
        &mut then_locals,
        next_local_id,
        then_block,
        expected_return,
    )?;
    let else_block = else_block
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

    Ok(TypedStmt::If {
        condition,
        then_block,
        else_block,
    })
}

fn check_return_stmt(
    symbols: &SymbolTable,
    locals: &HashMap<String, LocalSymbol>,
    expected: &TypedTypeRef,
    expr: &HirExpr,
) -> Result<TypedStmt, TypeckError> {
    let expr = check_expr(symbols, locals, expr)?;
    check_return_type(symbols, expected, &expr)?;

    Ok(TypedStmt::Return(expr))
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
    match expr {
        HirExpr::Path(path) => check_path_expr(symbols, locals, path),
        HirExpr::Int(value) => Ok(TypedExpr::Int {
            value: value.clone(),
            ty: symbols.builtin_type("Int32"),
        }),
        HirExpr::Text(value) => Ok(TypedExpr::Text {
            value: value.clone(),
            ty: symbols.builtin_type("Text"),
        }),
        HirExpr::Bool(value) => Ok(TypedExpr::Bool {
            value: *value,
            ty: symbols.builtin_type("Bool"),
        }),
        HirExpr::Call { callee, args } => check_call_expr(symbols, locals, callee, args),
    }
}

fn check_path_expr(
    symbols: &SymbolTable,
    locals: &HashMap<String, LocalSymbol>,
    path: &[String],
) -> Result<TypedExpr, TypeckError> {
    let joined = join_path(path);

    if path.len() == 1 {
        if let Some(local) = locals.get(&joined) {
            return Ok(TypedExpr::Path {
                path: path.to_vec(),
                ty: local.ty,
                resolved: ResolvedValue::Local(local.id),
            });
        }

        if symbols.procs.contains_key(&joined) {
            return Err(TypeckError::ProcedureUsedAsValue { name: joined });
        }

        return Err(TypeckError::UnknownLocal { name: joined });
    }

    if let Some(value) = symbols.values.get(&joined) {
        return Ok(TypedExpr::Path {
            path: path.to_vec(),
            ty: value.ty,
            resolved: ResolvedValue::BuiltinValue(value.id),
        });
    }

    Err(TypeckError::UnknownValuePath { path: joined })
}

fn check_call_expr(
    symbols: &SymbolTable,
    locals: &HashMap<String, LocalSymbol>,
    callee: &[String],
    args: &[HirExpr],
) -> Result<TypedExpr, TypeckError> {
    let name = join_path(callee);

    if symbols.values.contains_key(&name) || locals.contains_key(&name) {
        return Err(TypeckError::NotCallable { path: name });
    }

    let Some(proc) = symbols.procs.get(&name) else {
        return Err(TypeckError::UnknownProc { name });
    };

    if proc.params.len() != args.len() {
        return Err(TypeckError::ArgCountMismatch {
            proc: name,
            expected: proc.params.len(),
            found: args.len(),
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
            });
        }

        typed_args.push(typed);
    }

    Ok(TypedExpr::Call {
        callee: proc.id,
        args: typed_args,
        ty: proc.return_type,
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
    match stmt {
        TypedStmt::Return(_) => Flow::AlwaysReturns,
        TypedStmt::If {
            then_block,
            else_block: Some(else_block),
            ..
        } => combine_branch_flow(block_flow(then_block), block_flow(else_block)),
        TypedStmt::Local { .. } | TypedStmt::Assign { .. } | TypedStmt::If { .. } => {
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
    match expr {
        TypedExpr::Path { ty, .. } => *ty,
        TypedExpr::Int { ty, .. } => *ty,
        TypedExpr::Text { ty, .. } => *ty,
        TypedExpr::Bool { ty, .. } => *ty,
        TypedExpr::Call { ty, .. } => *ty,
    }
}

fn join_path(path: &[String]) -> String {
    path.join(".")
}

#[cfg(test)]
mod tests {
    use super::*;
    use torel_ir::{HirBlock, HirExpr, HirProc, HirStmt, HirTypeRef, HirVisibility};

    #[test]
    fn resolves_builtin_exit_value() {
        let hir = HirModule {
            unit_path: Some(vec!["examples".to_owned(), "hello".to_owned()]),
            procs: vec![HirProc {
                visibility: HirVisibility::Export,
                name: "main".to_owned(),
                params: Vec::new(),
                return_type: HirTypeRef {
                    path: vec!["Exit".to_owned()],
                },
                body: HirBlock {
                    stmts: vec![HirStmt::Return(HirExpr::Path(vec![
                        "Exit".to_owned(),
                        "ok".to_owned(),
                    ]))],
                    tail: None,
                },
            }],
        };

        let typed = check_types(&hir).expect("type checking should pass");

        assert_eq!(typed.unit_path, hir.unit_path);
        assert_eq!(typed.symbols.types, 7);
        assert_eq!(typed.symbols.values, 1);
        assert_eq!(typed.symbols.procs, 1);
        assert_eq!(typed.procs[0].return_type.display_name, "Exit");
    }
}
