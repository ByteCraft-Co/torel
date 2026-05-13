use torel_backend::{
    ArtifactKind, Backend, BackendArtifact, BackendError, BackendErrorKind, BackendTarget,
    validate_before_backend,
};
use torel_mir::MirModule;

#[derive(Debug, Default, Clone, Copy)]
pub struct CDebugBackend;

impl Backend for CDebugBackend {
    fn name(&self) -> &'static str {
        "c-debug"
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
            BackendTarget::C => Err(BackendError::new(
                BackendErrorKind::UnsupportedTarget,
                "C debug backend is reserved until MIR lowering exposes a supported primitive subset",
            )),
            BackendTarget::LlvmIr | BackendTarget::Object | BackendTarget::Executable => {
                Err(BackendError::new(
                    BackendErrorKind::UnsupportedTarget,
                    "C debug backend cannot emit LLVM IR, objects, or executables",
                ))
            }
        }
    }
}
