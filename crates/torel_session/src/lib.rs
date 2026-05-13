use std::path::PathBuf;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CompileSession {
    pub input: PathBuf,
    pub emit: EmitKind,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EmitKind {
    Check,
    LlvmIr,
    Object,
    Binary,
}

impl CompileSession {
    #[must_use]
    pub fn check(input: impl Into<PathBuf>) -> Self {
        Self {
            input: input.into(),
            emit: EmitKind::Check,
        }
    }
}
