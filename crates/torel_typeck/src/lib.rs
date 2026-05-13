use std::collections::HashMap;

use thiserror::Error;
use torel_ir::{
    HirBlock, HirExpr, HirModule, HirParam, HirProc, HirStmt, HirTypeRef, LocalId, ProcId,
    ResolvedValue, SymbolCounts, TypeId, TypedBlock, TypedExpr, TypedModule, TypedParam, TypedProc,
    TypedStmt, TypedTypeRef, ValueId,
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

    #[error("return type mismatch: expected `{expected}`, found `{found}`")]
    ReturnTypeMismatch { expected: String, found: String },

    #[error("missing return from procedure `{proc}`: expected `{expected}`")]
    MissingReturn { proc: String, expected: String },
}

#[derive(Debug, Clone)]
pub struct SymbolTable {
    types: HashMap<String, TypeId>,
    values: HashMap<String, ValueSymbol>,
    procs: HashMap<String, ProcId>,
    type_names: Vec<String>,
}

#[derive(Debug, Clone, Copy)]
struct ValueSymbol {
    id: ValueId,
    ty: TypeId,
}

#[derive(Debug, Clone, Copy)]
struct LocalSymbol {
    id: LocalId,
    ty: TypeId,
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

    fn insert_proc(&mut self, name: &str) -> Result<ProcId, TypeckError> {
        if self.procs.contains_key(name) {
            return Err(TypeckError::DuplicateProc {
                name: name.to_owned(),
            });
        }

        let id = ProcId(self.procs.len() as u32);
        self.procs.insert(name.to_owned(), id);
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
        symbols.insert_proc(&proc.name)?;
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
        .copied()
        .expect("procedure should be predeclared");
    let params = check_params(symbols, &proc.params)?;
    let locals = local_map(&params)?;
    let return_type = symbols.resolve_type(&proc.return_type)?;
    let body = check_block(symbols, &locals, &proc.body)?;

    check_returns(symbols, proc, &return_type, &body)?;

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
    locals: &HashMap<String, LocalSymbol>,
    block: &HirBlock,
) -> Result<TypedBlock, TypeckError> {
    Ok(TypedBlock {
        stmts: block
            .stmts
            .iter()
            .map(|stmt| check_stmt(symbols, locals, stmt))
            .collect::<Result<Vec<_>, _>>()?,
    })
}

fn check_stmt(
    symbols: &SymbolTable,
    locals: &HashMap<String, LocalSymbol>,
    stmt: &HirStmt,
) -> Result<TypedStmt, TypeckError> {
    match stmt {
        HirStmt::Return(expr) => Ok(TypedStmt::Return(check_expr(symbols, locals, expr)?)),
    }
}

fn check_expr(
    symbols: &SymbolTable,
    locals: &HashMap<String, LocalSymbol>,
    expr: &HirExpr,
) -> Result<TypedExpr, TypeckError> {
    match expr {
        HirExpr::Path(path) => check_path_expr(symbols, locals, path),
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

        if let Some(proc) = symbols.procs.get(&joined) {
            return Ok(TypedExpr::Path {
                path: path.to_vec(),
                ty: TypeId(6),
                resolved: ResolvedValue::Proc(*proc),
            });
        }
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

fn check_returns(
    symbols: &SymbolTable,
    proc: &HirProc,
    expected: &TypedTypeRef,
    body: &TypedBlock,
) -> Result<(), TypeckError> {
    let mut has_return = false;

    for stmt in &body.stmts {
        let TypedStmt::Return(expr) = stmt;
        has_return = true;
        let found = expr_type(expr);

        if found != expected.id {
            return Err(TypeckError::ReturnTypeMismatch {
                expected: expected.display_name.clone(),
                found: symbols.type_name(found),
            });
        }
    }

    if !has_return && expected.display_name != "Void" {
        return Err(TypeckError::MissingReturn {
            proc: proc.name.clone(),
            expected: expected.display_name.clone(),
        });
    }

    Ok(())
}

fn expr_type(expr: &TypedExpr) -> TypeId {
    match expr {
        TypedExpr::Path { ty, .. } => *ty,
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
