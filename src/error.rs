use crate::hgnc::error::HGNCError;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum PivotError {
    #[error(transparent)]
    HgncError(#[from] HGNCError),
    #[error("")]
    IncorrectGeneData(String),
    #[error("")]
    InvalidGeneVariantConfiguration(String),
}
