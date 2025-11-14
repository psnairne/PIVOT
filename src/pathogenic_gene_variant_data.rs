use crate::error::PivotError;

/// This enum carries the gene-variant configurations that we allow in our data
/// as a causative Genomic Interpretation for a disease.
///
/// gene should be either the gene symbol or the HGNC ID
/// the hgvs entries should be valid hgvs e.g. NM_001173464.1:c.2860C>T
#[derive(PartialEq, Debug)]
pub enum PathogenicGeneVariantData<'a> {
    None,
    CausativeGene(&'a str),
    HeterozygousVariant {
        gene: Option<&'a str>,
        hgvs: &'a str,
    },
    HomozygousVariant {
        gene: Option<&'a str>,
        hgvs: &'a str,
    },
    CompoundHeterozygousVariantPair {
        gene: Option<&'a str>,
        hgvs1: &'a str,
        hgvs2: &'a str,
    },
}

impl<'a> PathogenicGeneVariantData<'a> {
    /// Constructs a [`PathogenicGeneVariantData`] value from lists of genes and variants.
    ///
    /// # Valid Configurations
    ///
    /// - **No genes or variants** → `None`
    /// - **A single gene with no variants** → `CausativeGene`
    /// - **At most one gene with a single heterozygous variant** → `HeterozygousVariant`
    /// - **At most one gene with a pair of identical variants** → `HomozygousVariant`
    /// - **At most one gene with a pair of distinct, heterozygous variants** → `CompoundHeterozygousVariantPair`
    ///
    /// All other configurations are considered **invalid**.
    ///
    /// # Errors
    ///
    /// Returns an `Err` containing a descriptive message if the provided genes and variants
    /// do not match any valid configuration.
    pub fn from_genes_and_hgvs(
        genes: Vec<&'a str>,
        hgvs_strings: Vec<&'a str>,
    ) -> Result<PathogenicGeneVariantData<'a>, PivotError> {
        match (genes.len(), hgvs_strings.len()) {
            (0, 0) => Ok(PathogenicGeneVariantData::None),
            (1, 0) => Ok(PathogenicGeneVariantData::CausativeGene(genes[0])),
            (0, 1) | (1, 1) => Ok(PathogenicGeneVariantData::HeterozygousVariant {
                gene: genes.first().copied(),
                hgvs: hgvs_strings[0],
            }),
            (0, 2) | (1, 2) => {
                if hgvs_strings[0] == hgvs_strings[1] {
                    Ok(PathogenicGeneVariantData::HomozygousVariant {
                        gene: genes.first().copied(),
                        hgvs: hgvs_strings[0],
                    })
                } else {
                    Ok(PathogenicGeneVariantData::CompoundHeterozygousVariantPair {
                        gene: genes.first().copied(),
                        hgvs1: hgvs_strings[0],
                        hgvs2: hgvs_strings[1],
                    })
                }
            }
            _ => Err(PivotError::InvalidGeneVariantConfiguration(format!(
                "Invalid quantity of genes {} and HGVS variants {}. Could not interpret as PathogenicGeneVariantData.",
                genes.len(),
                hgvs_strings.len()
            ))),
        }
    }

    pub fn get_allelic_count(&self) -> usize {
        match self {
            PathogenicGeneVariantData::None => 0,
            PathogenicGeneVariantData::CausativeGene(_) => 0,
            PathogenicGeneVariantData::HeterozygousVariant { .. } => 1,
            PathogenicGeneVariantData::HomozygousVariant { .. } => 2,
            PathogenicGeneVariantData::CompoundHeterozygousVariantPair { .. } => 1,
        }
    }

    pub fn get_gene(&self) -> Option<&str> {
        match self {
            PathogenicGeneVariantData::None => None,
            PathogenicGeneVariantData::CausativeGene(gene) => Some(gene),
            PathogenicGeneVariantData::HeterozygousVariant { gene, .. }
            | PathogenicGeneVariantData::HomozygousVariant { gene, .. }
            | PathogenicGeneVariantData::CompoundHeterozygousVariantPair { gene, .. } => {
                gene.as_deref()
            }
        }
    }

    pub fn get_vars(&self) -> Vec<&str> {
        match self {
            PathogenicGeneVariantData::None | PathogenicGeneVariantData::CausativeGene(_) => vec![],
            PathogenicGeneVariantData::HomozygousVariant { hgvs, .. }
            | PathogenicGeneVariantData::HeterozygousVariant { hgvs, .. } => vec![hgvs],
            PathogenicGeneVariantData::CompoundHeterozygousVariantPair { hgvs1, hgvs2, .. } => {
                vec![hgvs1, hgvs2]
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::pathogenic_gene_variant_data::PathogenicGeneVariantData;
    use rstest::rstest;

    #[rstest]
    fn test_from_genes_and_variants() {
        assert!(matches!(
            PathogenicGeneVariantData::from_genes_and_hgvs(vec![], vec![]).unwrap(),
            PathogenicGeneVariantData::None
        ));
        assert!(matches!(
            PathogenicGeneVariantData::from_genes_and_hgvs(vec!["KIF21A"], vec![]).unwrap(),
            PathogenicGeneVariantData::CausativeGene(_)
        ));
        assert!(matches!(
            PathogenicGeneVariantData::from_genes_and_hgvs(
                vec![],
                vec!["NM_001173464.1:c.2860C>T"]
            )
            .unwrap(),
            PathogenicGeneVariantData::HeterozygousVariant { .. }
        ));
        assert!(matches!(
            PathogenicGeneVariantData::from_genes_and_hgvs(
                vec!["KIF21A"],
                vec!["NM_001173464.1:c.2860C>T"]
            )
            .unwrap(),
            PathogenicGeneVariantData::HeterozygousVariant { .. }
        ));
        assert!(matches!(
            PathogenicGeneVariantData::from_genes_and_hgvs(
                vec![],
                vec!["NM_001173464.1:c.2860C>T", "NM_001173464.1:c.2860C>T"]
            )
            .unwrap(),
            PathogenicGeneVariantData::HomozygousVariant { .. }
        ));
        assert!(matches!(
            PathogenicGeneVariantData::from_genes_and_hgvs(
                vec!["KIF21A"],
                vec!["NM_001173464.1:c.2860C>T", "NM_001173464.1:c.2860C>T"]
            )
            .unwrap(),
            PathogenicGeneVariantData::HomozygousVariant { .. }
        ));
        assert!(matches!(
            PathogenicGeneVariantData::from_genes_and_hgvs(
                vec![],
                vec!["NM_001173464.1:c.2860C>T", "NM_015120.4:c.11031_11032delGA"]
            )
            .unwrap(),
            PathogenicGeneVariantData::CompoundHeterozygousVariantPair { .. }
        ));
        assert!(matches!(
            PathogenicGeneVariantData::from_genes_and_hgvs(
                vec!["KIF21A"],
                vec!["NM_001173464.1:c.2860C>T", "NM_015120.4:c.11031_11032delGA"]
            )
            .unwrap(),
            PathogenicGeneVariantData::CompoundHeterozygousVariantPair { .. }
        ));
    }

    #[rstest]
    fn test_from_genes_and_variants_invalid_configuration() {
        // multiple genes
        assert!(
            PathogenicGeneVariantData::from_genes_and_hgvs(
                vec!["KIF21A", "CLOCK"],
                vec!["NM_001173464.1:c.2860C>T"]
            )
            .is_err()
        );
        // too many variants
        assert!(
            PathogenicGeneVariantData::from_genes_and_hgvs(
                vec!["KIF21A"],
                vec![
                    "NM_001173464.1:c.2860C>T",
                    "NM_001173464.1:c.2860C>T",
                    "NM_001173464.1:c.2860C>T"
                ]
            )
            .is_err()
        );
    }
}
