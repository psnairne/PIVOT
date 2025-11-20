use crate::hgnc::error::HGNCError;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum PivotError {
    #[error(transparent)]
    HgncError(#[from] HGNCError),
    #[error("Provided {id_type} {gene} does not match with HGVS variant {hgnc_id}")]
    IncorrectGeneData {
        id_type: String,
        gene: String,
        hgnc_id: String,
    },
    #[error(
        "Invalid quantity of genes '{n_genes}' and HGVS variants '{n_variants}'. Could not interpret as PathogenicGeneVariantData."
    )]
    InvalidGeneVariantConfiguration { n_genes: usize, n_variants: usize },
    #[error("None ASCII character found in HGVS string: {0}")]
    NonAsciiCharacter(String),
    #[error("Problem {problem} in found HGVS: {hgvs} ")]
    IncorrectHGVSFormat { hgvs: String, problem: String },
    #[error("Temporary Error")]
    TemporaryError,
}
