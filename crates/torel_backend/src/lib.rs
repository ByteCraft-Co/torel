use thiserror::Error;
use torel_diagnostics::Span;
use torel_mir::{MirModule, validate_module};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BackendTarget {
    MirText,
    C,
    LlvmIr,
    Object,
    Executable,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ArtifactKind {
    MirText,
    CSource,
    LlvmIr,
    Object,
    Executable,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BackendArtifact {
    pub kind: ArtifactKind,
    pub text: Option<String>,
    pub bytes: Vec<u8>,
}

impl BackendArtifact {
    #[must_use]
    pub fn text(kind: ArtifactKind, text: String) -> Self {
        Self {
            kind,
            text: Some(text),
            bytes: Vec::new(),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BackendErrorKind {
    InvalidMir,
    UnsupportedTarget,
    UnsupportedFeature,
    BridgeFailure,
    VerificationFailure,
}

#[derive(Debug, Error, Clone, PartialEq, Eq)]
#[error("{kind:?}: {message}")]
pub struct BackendError {
    pub kind: BackendErrorKind,
    pub message: String,
    pub span: Option<Span>,
}

impl BackendError {
    #[must_use]
    pub fn new(kind: BackendErrorKind, message: impl Into<String>) -> Self {
        Self {
            kind,
            message: message.into(),
            span: None,
        }
    }

    #[must_use]
    pub fn with_span(mut self, span: Span) -> Self {
        self.span = Some(span);
        self
    }
}

pub trait Backend {
    fn name(&self) -> &'static str;

    fn emit(
        &self,
        module: &MirModule,
        target: BackendTarget,
    ) -> Result<BackendArtifact, BackendError>;
}

pub fn validate_before_backend(module: &MirModule) -> Result<(), BackendError> {
    validate_module(module).map_err(|err| {
        BackendError::new(
            BackendErrorKind::InvalidMir,
            format!("validated backend input required: {err}"),
        )
    })
}
