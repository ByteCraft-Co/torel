use torel_backend::{
    ArtifactKind, Backend, BackendArtifact, BackendError, BackendErrorKind, BackendTarget,
    validate_before_backend,
};
use torel_mir::MirModule;

#[derive(Debug, Default, Clone, Copy)]
pub struct LlvmBackend;

impl Backend for LlvmBackend {
    fn name(&self) -> &'static str {
        "llvm-cpp"
    }

    fn emit(
        &self,
        module: &MirModule,
        target: BackendTarget,
    ) -> Result<BackendArtifact, BackendError> {
        validate_before_backend(module)?;

        match target {
            BackendTarget::MirText => Ok(BackendArtifact::text(
                ArtifactKind::MirText,
                module.pretty(),
            )),
            BackendTarget::LlvmIr => Err(BackendError::new(
                BackendErrorKind::UnsupportedTarget,
                "C++ LLVM bridge is reserved; textual LLVM emission lands after MIR lowering is wired",
            )),
            BackendTarget::Object | BackendTarget::Executable => Err(BackendError::new(
                BackendErrorKind::UnsupportedTarget,
                "object and executable emission wait for verified LLVM IR",
            )),
            BackendTarget::C => Err(BackendError::new(
                BackendErrorKind::UnsupportedTarget,
                "LLVM backend does not emit C source",
            )),
        }
    }
}
