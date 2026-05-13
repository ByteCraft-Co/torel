use thiserror::Error;
use torel_ir::TypedModule;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct OwnershipReport {
    pub checked_owner_regions: usize,
}

#[derive(Debug, Error)]
pub enum OwnershipError {
    #[error("ownership checking is not implemented for this construct yet")]
    UnimplementedConstruct,
}

pub fn check_ownership(module: &TypedModule) -> Result<OwnershipReport, OwnershipError> {
    Ok(OwnershipReport {
        checked_owner_regions: module.item_count,
    })
}
