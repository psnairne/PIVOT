use thiserror::Error;

#[derive(Debug, Error)]
pub enum HGVSError {
    #[error("Problem {problem} in found HGVS: {hgvs} ")]
    IncorrectHGVSFormat { hgvs: String, problem: String },
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
        "The provided {id_type} {expected_gene} does not match with the {hgvs_gene} that of HGVS variant {hgvs}"
    )]
    MismatchingGeneData {
        id_type: String,
        expected_gene: String,
        hgvs: String,
        hgvs_gene: String,
    },
    #[error(
        "VariantValidator response for {hgvs} has element {element} with following problem: {problem}"
    )]
    InvalidVariantValidatorResponseElement {
        hgvs: String,
        element: String,
        problem: String,
    },
}
