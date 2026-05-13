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
                "C++ LLVM bridge is reserved but not built in this toolchain; install LLVM and build the bridge before emitting verified LLVM IR",
            )),
            BackendTarget::Object | BackendTarget::Executable => Err(BackendError::new(
                BackendErrorKind::UnsupportedTarget,
                "object and executable emission require the verified C++ LLVM bridge",
            )),
            BackendTarget::C => Err(BackendError::new(
                BackendErrorKind::UnsupportedTarget,
                "LLVM backend does not emit C source",
            )),
        }
    }
}
