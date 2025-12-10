#![allow(unused)]
use crate::hgvs::enums::{AlleleCount, ChromosomalSex};
use crate::hgvs::error::HGVSError;
use crate::hgvs::utils::{is_c_hgvs, is_n_hgvs};
use crate::utils::is_hgnc_id;
use phenopackets::ga4gh::vrsatile::v1::{
    Expression, GeneDescriptor, MoleculeContext, VariationDescriptor, VcfRecord,
};
use phenopackets::schema::v2::core::{
    AcmgPathogenicityClassification, OntologyClass, TherapeuticActionability, VariantInterpretation,
};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize, Default, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct HgvsVariant {
    /// Genome build, e.g., hg38
    assembly: String,
    /// Chromosome, e.g., "17"
    chr: String,
    /// Position on the chromosome
    position: u32,
    /// Reference allele
    ref_allele: String,
    /// Alternate allele
    alt_allele: String,
    /// Gene symbol, e.g., FBN1
    symbol: String,
    /// HUGO Gene Nomenclature Committee identifier, e.g., HGNC:3603
    hgnc_id: String,
    /// Transcript, e.g., NM_000138.5
    transcript: String,
    /// HGVS Nomenclature, e.g., c.8242G>T
    allele: String,
    /// Coding or Non-coding (RNA) gene HGVS nomenclature, e.g., NM_000138.5:c.8242G>T or NR_002196.1:n.601G>T
    transcript_hgvs: String,
    /// Genomic HGVS nomenclature, e.g., NC_000015.10:g.48411364C>A
    g_hgvs: String,
    /// Protein level HGVS, if available
    p_hgvs: Option<String>,
}

impl HgvsVariant {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        assembly: impl Into<String>,
        chr: impl Into<String>,
        position: u32,
        ref_allele: impl Into<String>,
        alt_allele: impl Into<String>,
        symbol: impl Into<String>,
        hgnc_id: impl Into<String>,
        transcript: impl Into<String>,
        allele: impl Into<String>,
        transcript_hgvs: impl Into<String>,
        g_hgvs: impl Into<String>,
        p_hgvs: Option<impl Into<String>>,
    ) -> Self {
        HgvsVariant {
            assembly: assembly.into(),
            chr: chr.into(),
            position,
            ref_allele: ref_allele.into(),
            alt_allele: alt_allele.into(),
            symbol: symbol.into(),
            hgnc_id: hgnc_id.into(),
            transcript: transcript.into(),
            allele: allele.into(),
            transcript_hgvs: transcript_hgvs.into(),
            g_hgvs: g_hgvs.into(),
            p_hgvs: p_hgvs.map(|s| s.into()),
        }
    }

    pub fn assembly(&self) -> &str {
        self.assembly.as_ref()
    }

    pub fn chr(&self) -> &str {
        self.chr.as_ref()
    }

    pub fn position(&self) -> u32 {
        self.position
    }

    pub fn ref_allele(&self) -> &str {
        self.ref_allele.as_ref()
    }

    pub fn alt_allele(&self) -> &str {
        self.alt_allele.as_ref()
    }

    pub fn hgnc_id(&self) -> &str {
        self.hgnc_id.as_ref()
    }

    pub fn gene_symbol(&self) -> &str {
        &self.symbol
    }

    pub fn transcript(&self) -> &str {
        self.transcript.as_ref()
    }

    pub fn allele(&self) -> &str {
        self.allele.as_ref()
    }

    pub fn transcript_hgvs(&self) -> &str {
        self.transcript_hgvs.as_ref()
    }

    pub fn g_hgvs(&self) -> &str {
        self.g_hgvs.as_ref()
    }

    pub fn p_hgvs(&self) -> Option<String> {
        self.p_hgvs.as_ref().map(|phgvs| phgvs.to_string())
    }

    pub fn is_x_chromosomal(&self) -> bool {
        self.chr.contains("X")
    }

    pub fn is_y_chromosomal(&self) -> bool {
        self.chr.contains("Y")
    }

    /// Create Phenopacket VariantInterpretation from a ValidatedHgvs and an allele count.
    /// Throws an error if the allele count is not 1 or 2.
    pub fn create_variant_interpretation(
        &self,
        allele_count: AlleleCount,
        sex: ChromosomalSex,
    ) -> Result<VariantInterpretation, HGVSError> {
        let gene_context = GeneDescriptor {
            value_id: self.hgnc_id().to_string(),
            symbol: self.gene_symbol().to_string(),
            ..Default::default()
        };

        let mut expressions = vec![];

        if is_c_hgvs(self.allele()) {
            let hgvs_c = Expression {
                syntax: "hgvs.c".to_string(),
                value: self.transcript_hgvs().to_string(),
                version: String::default(),
            };
            expressions.push(hgvs_c);
        }

        if is_n_hgvs(self.allele()) {
            let hgvs_n = Expression {
                syntax: "hgvs.n".to_string(),
                value: self.transcript_hgvs().to_string(),
                version: String::default(),
            };
            expressions.push(hgvs_n);
        }

        expressions.push(Expression {
            syntax: "hgvs.g".to_string(),
            value: self.g_hgvs().to_string(),
            version: String::default(),
        });

        if let Some(hgvs_p) = &self.p_hgvs() {
            let hgvs_p = Expression {
                syntax: "hgvs.p".to_string(),
                value: hgvs_p.clone(),
                version: String::default(),
            };
            expressions.push(hgvs_p);
        }

        let vcf_record = VcfRecord {
            genome_assembly: self.assembly().to_string(),
            chrom: self.chr().to_string(),
            pos: self.position() as u64,
            r#ref: self.ref_allele().to_string(),
            alt: self.alt_allele().to_string(),
            ..Default::default()
        };

        let allelic_state = Self::get_allele_term(
            sex,
            allele_count,
            self.is_x_chromosomal(),
            self.is_y_chromosomal(),
        )?;

        let variation_descriptor = VariationDescriptor {
            id: self.g_hgvs().to_string(), // I'm not entirely happy with this
            gene_context: Some(gene_context),
            expressions,
            vcf_record: Some(vcf_record),
            molecule_context: MoleculeContext::Genomic.into(),
            allelic_state: Some(allelic_state),
            ..Default::default()
        };
        Ok(VariantInterpretation {
            acmg_pathogenicity_classification: AcmgPathogenicityClassification::Pathogenic.into(),
            therapeutic_actionability: TherapeuticActionability::UnknownActionability.into(),
            variation_descriptor: Some(variation_descriptor),
        })
    }

    fn get_allele_term(
        chromosomal_sex: ChromosomalSex,
        allele_count: AlleleCount,
        is_x: bool,
        is_y: bool,
    ) -> Result<OntologyClass, HGVSError> {
        match (&chromosomal_sex, &allele_count, is_x, is_y) {
            // variants on non-sex chromosomes
            (_, AlleleCount::Double, false, false) => Ok(OntologyClass {
                id: "GENO:0000136".to_string(),
                label: "homozygous".to_string(),
            }),
            (_, AlleleCount::Single, false, false) => Ok(OntologyClass {
                id: "GENO:0000135".to_string(),
                label: "heterozygous".to_string(),
            }),
            // variants on x-chromosome
            (
                ChromosomalSex::XX
                | ChromosomalSex::XXY
                | ChromosomalSex::XXX
                | ChromosomalSex::Unknown,
                AlleleCount::Double,
                true,
                false,
            ) => Ok(OntologyClass {
                id: "GENO:0000136".to_string(),
                label: "homozygous".to_string(),
            }),
            (
                ChromosomalSex::XX | ChromosomalSex::XXY | ChromosomalSex::XXX,
                AlleleCount::Single,
                true,
                false,
            ) => Ok(OntologyClass {
                id: "GENO:0000135".to_string(),
                label: "heterozygous".to_string(),
            }),
            (
                ChromosomalSex::X | ChromosomalSex::XY | ChromosomalSex::XYY,
                AlleleCount::Single,
                true,
                false,
            ) => Ok(OntologyClass {
                id: "GENO:0000134".to_string(),
                label: "hemizygous".to_string(),
            }),
            (ChromosomalSex::Unknown, AlleleCount::Single, true, false) => Ok(OntologyClass {
                id: "GENO:0000137".to_string(),
                label: "unspecified zygosity".to_string(),
            }),
            // variants on y-chromosome
            (ChromosomalSex::XYY | ChromosomalSex::Unknown, AlleleCount::Double, false, true) => {
                Ok(OntologyClass {
                    id: "GENO:0000136".to_string(),
                    label: "homozygous".to_string(),
                })
            }
            (ChromosomalSex::XYY, AlleleCount::Single, false, true) => Ok(OntologyClass {
                id: "GENO:0000135".to_string(),
                label: "heterozygous".to_string(),
            }),
            (ChromosomalSex::XY | ChromosomalSex::XXY, AlleleCount::Single, false, true) => {
                Ok(OntologyClass {
                    id: "GENO:0000134".to_string(),
                    label: "hemizygous".to_string(),
                })
            }
            (ChromosomalSex::Unknown, AlleleCount::Single, false, true) => Ok(OntologyClass {
                id: "GENO:0000137".to_string(),
                label: "unspecified zygosity".to_string(),
            }),
            // nothing else makes sense
            _ => Err(HGVSError::ContradictoryAllelicData {
                chromosomal_sex,
                allele_count,
                is_x,
                is_y,
            }),
        }
    }

    pub fn validate_against_gene(&self, gene: &str) -> Result<(), HGVSError> {
        let (expected, id_type) = if is_hgnc_id(gene) {
            (self.hgnc_id.as_str(), "HGNC ID")
        } else {
            (self.symbol.as_str(), "gene symbol")
        };

        if gene == expected {
            Ok(())
        } else {
            Err(HGVSError::MismatchingGeneData {
                id_type: id_type.to_string(),
                inputted_gene: gene.to_string(),
                hgvs: self.transcript_hgvs.clone(),
                actual_gene: self.hgnc_id.clone(),
            })
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::hgvs::enums::{AlleleCount, ChromosomalSex};
    use crate::hgvs::hgvs_client::HGVSClient;
    use crate::hgvs::hgvs_variant::HgvsVariant;
    use crate::hgvs::traits::HGVSData;
    use phenopackets::ga4gh::vrsatile::v1::Expression;
    use rstest::{fixture, rstest};

    #[fixture]
    fn validated_c_hgvs() -> HgvsVariant {
        HgvsVariant::new(
            "hg38",
            "chr12",
            38332495,
            "G",
            "A",
            "KIF21A",
            "HGNC:19349",
            "NM_001173464.1",
            "c.2860C>T",
            "NM_001173464.1:c.2860C>T",
            "NC_000012.12:g.39332405G>A",
            Some("NP_001166935.1:p.(Arg954Trp)"),
        )
    }

    #[fixture]
    fn validated_n_hgvs() -> HgvsVariant {
        HgvsVariant::new(
            "hg38",
            "chr11",
            1997235,
            "C",
            "A",
            "H19",
            "HGNC:4713",
            "NR_002196.1",
            "n.601G>T",
            "NR_002196.1:n.601G>T",
            "NC_000011.10:g.1997235C>A",
            None::<&str>,
        )
    }

    #[rstest]
    fn test_validate_against_gene() {
        validated_c_hgvs().validate_against_gene("KIF21A").unwrap();
        validated_c_hgvs()
            .validate_against_gene("HGNC:19349")
            .unwrap();
    }

    #[rstest]
    fn test_validate_against_gene_err() {
        assert!(validated_c_hgvs().validate_against_gene("CLOCK").is_err());
        assert!(
            validated_c_hgvs()
                .validate_against_gene("HGNC:1234")
                .is_err()
        );
    }

    #[rstest]
    fn test_get_allele_term_heterozygous() {
        let allele_term =
            HgvsVariant::get_allele_term(ChromosomalSex::XX, AlleleCount::Single, false, false)
                .unwrap();
        assert_eq!(allele_term.label, "heterozygous");
    }

    #[rstest]
    fn test_get_allele_term_heterozygous_on_x() {
        let allele_term =
            HgvsVariant::get_allele_term(ChromosomalSex::XX, AlleleCount::Single, true, false)
                .unwrap();
        assert_eq!(allele_term.label, "heterozygous");
    }

    #[rstest]
    fn test_get_allele_term_homozygous() {
        let allele_term = HgvsVariant::get_allele_term(
            ChromosomalSex::Unknown,
            AlleleCount::Double,
            false,
            false,
        )
        .unwrap();
        assert_eq!(allele_term.label, "homozygous");
    }

    #[rstest]
    fn test_get_allele_term_hemizygous_on_x() {
        let allele_term =
            HgvsVariant::get_allele_term(ChromosomalSex::XYY, AlleleCount::Single, true, false)
                .unwrap();
        assert_eq!(allele_term.label, "hemizygous");
    }

    #[rstest]
    fn test_get_allele_term_hemizygous_on_y() {
        let allele_term =
            HgvsVariant::get_allele_term(ChromosomalSex::XXY, AlleleCount::Single, false, true)
                .unwrap();
        assert_eq!(allele_term.label, "hemizygous");
    }

    #[rstest]
    fn test_get_allele_term_unknown_on_x() {
        let allele_term =
            HgvsVariant::get_allele_term(ChromosomalSex::Unknown, AlleleCount::Single, true, false)
                .unwrap();
        assert_eq!(allele_term.label, "unspecified zygosity");
    }

    #[rstest]
    fn test_get_allele_term_unknown_on_y() {
        let allele_term =
            HgvsVariant::get_allele_term(ChromosomalSex::Unknown, AlleleCount::Single, false, true)
                .unwrap();
        assert_eq!(allele_term.label, "unspecified zygosity");
    }

    #[rstest]
    fn test_get_allele_term_unknown_not_on_x_or_y() {
        let allele_term = HgvsVariant::get_allele_term(
            ChromosomalSex::Unknown,
            AlleleCount::Single,
            false,
            false,
        )
        .unwrap();
        assert_eq!(allele_term.label, "heterozygous");
    }

    #[rstest]
    fn test_get_allele_term_on_x_and_y() {
        let result =
            HgvsVariant::get_allele_term(ChromosomalSex::Unknown, AlleleCount::Single, true, true);
        assert!(result.is_err());
    }

    #[rstest]
    fn test_get_allele_term_not_enough_x_chromosomes() {
        let result =
            HgvsVariant::get_allele_term(ChromosomalSex::XY, AlleleCount::Double, true, false);
        assert!(result.is_err());
    }

    #[rstest]
    fn test_create_variant_interpretation_c_hgvs() {
        let vi = validated_c_hgvs()
            .create_variant_interpretation(AlleleCount::Single, ChromosomalSex::Unknown)
            .unwrap();

        let vi_allelic_state = vi
            .variation_descriptor
            .clone()
            .unwrap()
            .allelic_state
            .unwrap()
            .label;
        assert_eq!(vi_allelic_state, "heterozygous");

        let vi_expressions = vi.variation_descriptor.clone().unwrap().expressions;
        assert_eq!(vi_expressions.len(), 3);
        let c_hgvs_expressions = vi_expressions
            .iter()
            .filter(|exp| exp.syntax == "hgvs.c")
            .collect::<Vec<&Expression>>();
        let c_hgvs_expression = c_hgvs_expressions.first().unwrap();
        assert_eq!(c_hgvs_expression.value, validated_c_hgvs().transcript_hgvs);
    }

    #[rstest]
    fn test_create_variant_interpretation_n_hgvs() {
        let vi = validated_n_hgvs()
            .create_variant_interpretation(AlleleCount::Double, ChromosomalSex::Unknown)
            .unwrap();

        let vi_allelic_state = vi
            .variation_descriptor
            .clone()
            .unwrap()
            .allelic_state
            .unwrap()
            .label;
        assert_eq!(vi_allelic_state, "homozygous");

        let vi_expressions = vi.variation_descriptor.clone().unwrap().expressions;
        assert_eq!(vi_expressions.len(), 2);
        let n_hgvs_expressions = vi_expressions
            .iter()
            .filter(|exp| exp.syntax == "hgvs.n")
            .collect::<Vec<&Expression>>();
        let n_hgvs_expression = n_hgvs_expressions.first().unwrap();
        assert_eq!(n_hgvs_expression.value, validated_n_hgvs().transcript_hgvs);
    }
}
