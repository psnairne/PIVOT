#[derive(Debug)]
pub enum PivotError {
    HgncError(String),
    IncorrectGeneData(String),
    InvalidGeneVariantConfiguration(String),
}
