use thiserror::Error;
use torel_ir::{
    HirBlock, HirExpr, HirModule, HirParam, HirProc, HirStmt, HirTypeRef, TypedBlock, TypedExpr,
    TypedModule, TypedParam, TypedProc, TypedStmt, TypedTypeRef,
};

#[derive(Debug, Error)]
pub enum TypeckError {
    #[error("type checking is not implemented for this construct yet")]
    UnimplementedConstruct,
}

pub fn check_types(hir: &HirModule) -> Result<TypedModule, TypeckError> {
    Ok(TypedModule {
        unit_path: hir.unit_path.clone(),
        procs: hir.procs.iter().map(check_proc).collect(),
    })
}

fn check_proc(proc: &HirProc) -> TypedProc {
    TypedProc {
        visibility: proc.visibility,
        name: proc.name.clone(),
        params: proc.params.iter().map(check_param).collect(),
        return_type: check_type_ref(&proc.return_type),
        body: check_block(&proc.body),
    }
}

fn check_param(param: &HirParam) -> TypedParam {
    TypedParam {
        name: param.name.clone(),
        ty: check_type_ref(&param.ty),
    }
}

fn check_type_ref(ty: &HirTypeRef) -> TypedTypeRef {
    TypedTypeRef {
        path: ty.path.clone(),
    }
}

fn check_block(block: &HirBlock) -> TypedBlock {
    TypedBlock {
        stmts: block.stmts.iter().map(check_stmt).collect(),
    }
}

fn check_stmt(stmt: &HirStmt) -> TypedStmt {
    match stmt {
        HirStmt::Return(expr) => TypedStmt::Return(check_expr(expr)),
    }
}

fn check_expr(expr: &HirExpr) -> TypedExpr {
    match expr {
        HirExpr::Path(path) => TypedExpr::Path(path.clone()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn produces_typed_module_from_hir() {
        let hir = HirModule {
            unit_path: Some(vec!["examples".to_owned(), "hello".to_owned()]),
            procs: Vec::new(),
        };

        let typed = check_types(&hir).expect("type checking should pass");

        assert_eq!(typed.unit_path, hir.unit_path);
        assert!(typed.procs.is_empty());
    }
}
