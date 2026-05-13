use thiserror::Error;
use torel_ir::TypedModule;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct EffectReport {
    pub checked_effect_sets: usize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FailureReport {
    pub checked_failure_sets: usize,
}

#[derive(Debug, Error)]
pub enum EffectError {
    #[error("effect checking is not implemented for this construct yet")]
    UnimplementedConstruct,
}

#[derive(Debug, Error)]
pub enum FailureError {
    #[error("failure checking is not implemented for this construct yet")]
    UnimplementedConstruct,
}

pub fn check_effects(module: &TypedModule) -> Result<EffectReport, EffectError> {
    Ok(EffectReport {
        checked_effect_sets: module.procs.len(),
    })
}

pub fn check_failures(module: &TypedModule) -> Result<FailureReport, FailureError> {
    Ok(FailureReport {
        checked_failure_sets: module.procs.len(),
    })
}
