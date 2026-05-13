use thiserror::Error;
use torel_ir::{HirModule, TypedModule};

#[derive(Debug, Error)]
pub enum TypeckError {
    #[error("type checking is not implemented for this construct yet")]
    UnimplementedConstruct,
}

pub fn check_types(hir: &HirModule) -> Result<TypedModule, TypeckError> {
    Ok(TypedModule {
        unit_path: hir.unit_path.clone(),
        item_count: hir.item_count,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn produces_typed_module_from_hir() {
        let hir = HirModule {
            unit_path: Some(vec!["examples".to_owned(), "hello".to_owned()]),
            item_count: 0,
        };

        let typed = check_types(&hir).expect("type checking should pass");

        assert_eq!(typed.unit_path, hir.unit_path);
    }
}
