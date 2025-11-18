use crate::hgnc::error::HGNCError;
use thiserror::Error;
use crate::hgvs_variant::HgvsVariant;

#[derive(Debug, Error)]
pub enum PivotError {
    #[error(transparent)]
    HgncError(#[from] HGNCError),
    #[error("Provided {id_type} {gene} does not match with HGVS variant {variant:?}")]
    IncorrectGeneData{id_type: String, gene: String, variant: HgvsVariant},
    #[error("Invalid quantity of genes '{n_genes}' and HGVS variants '{n_variants}'. Could not interpret as PathogenicGeneVariantData.")]
    InvalidGeneVariantConfiguration{n_genes: usize, n_variants: usize},
}
