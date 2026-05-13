use thiserror::Error;
use torel_ir::TypedModule;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CodegenTarget {
    CheckOnly,
    LlvmIr,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CodegenOutput {
    pub target: CodegenTarget,
    pub text: String,
}

#[derive(Debug, Error)]
pub enum CodegenError {
    #[error("LLVM code generation is not implemented yet")]
    LlvmNotImplemented,
}

pub fn codegen(module: &TypedModule, target: CodegenTarget) -> Result<CodegenOutput, CodegenError> {
    match target {
        CodegenTarget::CheckOnly => Ok(CodegenOutput {
            target,
            text: format!(
                "check-only module unit={} items={}",
                module
                    .unit_path
                    .as_ref()
                    .map(|path| path.join("."))
                    .unwrap_or_else(|| "<anonymous>".to_owned()),
                module.item_count
            ),
        }),
        CodegenTarget::LlvmIr => Err(CodegenError::LlvmNotImplemented),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn emits_check_only_summary() {
        let module = TypedModule {
            unit_path: Some(vec!["examples".to_owned(), "hello".to_owned()]),
            item_count: 0,
        };

        let output = codegen(&module, CodegenTarget::CheckOnly).expect("check-only codegen");

        assert_eq!(output.target, CodegenTarget::CheckOnly);
        assert!(output.text.contains("examples.hello"));
    }
}
