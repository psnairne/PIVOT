use thiserror::Error;

#[derive(Debug, Error)]
pub enum HGVSError {
    #[error("Hgvs string {hgvs} not accepted due to format problem: {problem}.")]
    HgvsFormatNotAccepted { hgvs: String, problem: String },
    #[error(
        "VariantValidator response for {hgvs} did not have flag gene_variant. The flag was {flag} instead. "
    )]
    NonGeneVariant { hgvs: String, flag: String },
    #[error(
        "VariantValidator response for {hgvs} did not have genome_assembly {desired_assembly}. The following assemblies were found instead: {found_assemblies:?}"
    )]
    GenomeAssemblyNotFound {
        hgvs: String,
        desired_assembly: String,
        found_assemblies: Vec<String>,
    },
    #[error(
        "VariantValidator response for {hgvs} has element {element} with following problem: {problem}"
    )]
    InvalidVariantValidatorResponseElement {
        hgvs: String,
        element: String,
        problem: String,
    },
    #[error(
        "Expected {expected} variant_info entries in VariantValidator response but {found} were found."
    )]
    WrongNumberOfVariantInfos { expected: usize, found: usize },
    #[error(
        "VariantValidator response for {hgvs} could not be deserialized to schema. {hgvs} may be invalid. Error: {err}."
    )]
    DeserializeVariantValidatorResponseToSchema { hgvs: String, err: String },
    #[error("VariantValidator fetch request for {hgvs} failed. Error: {err}.")]
    FetchRequest { hgvs: String, err: String },
}
