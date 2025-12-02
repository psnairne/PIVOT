#![allow(unused)]
use crate::hgvs::error::HGVSError;
use crate::utils::{get_allele_term, is_hgnc_id};
use phenopackets::ga4gh::vrsatile::v1::{
    Expression, GeneDescriptor, MoleculeContext, VariationDescriptor, VcfRecord,
};
use phenopackets::schema::v2::core::{
    AcmgPathogenicityClassification, TherapeuticActionability, VariantInterpretation,
};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ValidatedCHgvs {
    /// Genome build, e.g., hg38
    assembly: String,
    /// Chromosome, e.g., "17"
    chr: String,
    /// Position on the chromosome
    position: u64,
    /// Reference allele
    ref_allele: String,
    /// Alternate allele
    alt_allele: String,
    /// HUGO Gene Nomenclature Committee identifier, e.g., HGNC:3603
    hgnc_id: String,
    /// Gene symbol, e.g., FBN1
    gene_symbol: String,
    /// Transcript, e.g., NM_000138.5
    transcript: String,
    /// HGVS Nomenclature, e.g., c.8242G>T
    allele: String,
    /// Coding HGVS nomenclature, e.g., NM_000138.5:c.8242G>T
    c_hgvs: String,
    /// Genomic HGVS nomenclature, e.g., NC_000015.10:g.48411364C>A
    g_hgvs: String,
    /// Protein level HGVS, if available
    p_hgvs: Option<String>,
}

impl ValidatedCHgvs {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        assembly: String,
        chr: String,
        position: u64,
        ref_allele: String,
        alt_allele: String,
        hgnc_id: String,
        gene_symbol: String,
        transcript: String,
        allele: String,
        c_hgvs: String,
        g_hgvs: String,
        p_hgvs: Option<String>,
    ) -> Self {
        ValidatedCHgvs {
            assembly,
            chr,
            position,
            ref_allele,
            alt_allele,
            hgnc_id,
            gene_symbol,
            transcript,
            allele,
            c_hgvs,
            g_hgvs,
            p_hgvs,
        }
    }

    pub fn assembly(&self) -> &str {
        self.assembly.as_ref()
    }

    pub fn chr(&self) -> &str {
        self.chr.as_ref()
    }

    pub fn position(&self) -> u64 {
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
        &self.gene_symbol
    }

    pub fn transcript(&self) -> &str {
        self.transcript.as_ref()
    }

    pub fn allele(&self) -> &str {
        self.allele.as_ref()
    }

    pub fn c_hgvs(&self) -> &str {
        self.c_hgvs.as_ref()
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

    pub fn validate_against_gene(&self, gene: &str) -> Result<(), HGVSError> {
        let (expected, id_type) = if is_hgnc_id(gene) {
            (self.hgnc_id.as_str(), "HGNC ID")
        } else {
            (self.gene_symbol.as_str(), "gene symbol")
        };

        if gene == expected {
            Ok(())
        } else {
            Err(HGVSError::MismatchingGeneData {
                id_type: id_type.to_string(),
                expected_gene: gene.to_string(),
                hgvs: self.g_hgvs.to_string(),
                hgvs_gene: self.hgnc_id.to_string(),
            })
        }
    }

    /// Create Phenopacket VariantInterpretation from a ValidatedHgvs and an allele count.
    pub fn get_hgvs_variant_interpretation(&self, allele_count: usize) -> VariantInterpretation {
        let gene_context = GeneDescriptor {
            value_id: self.hgnc_id().to_string(),
            symbol: self.gene_symbol().to_string(),
            ..Default::default()
        };

        let hgvs_c = Expression {
            syntax: "hgvs.c".to_string(),
            value: self.c_hgvs().to_string(),
            version: String::default(),
        };
        let hgvs_g = Expression {
            syntax: "hgvs.g".to_string(),
            value: self.g_hgvs().to_string(),
            version: String::default(),
        };
        let expressions = if let Some(hgvs_p) = &self.p_hgvs() {
            let hgvs_p = Expression {
                syntax: "hgvs.p".to_string(),
                value: hgvs_p.clone(),
                version: String::default(),
            };
            vec![hgvs_c, hgvs_g, hgvs_p]
        } else {
            vec![hgvs_c, hgvs_g]
        };

        let vcf_record = VcfRecord {
            genome_assembly: self.assembly().to_string(),
            chrom: self.chr().to_string(),
            pos: self.position(),
            r#ref: self.ref_allele().to_string(),
            alt: self.alt_allele().to_string(),
            ..Default::default()
        };

        let allelic_state = get_allele_term(allele_count, self.is_x_chromosomal());

        let variation_descriptor = VariationDescriptor {
            id: self.g_hgvs().to_string(), // I'm not entirely happy with this
            gene_context: Some(gene_context),
            expressions,
            vcf_record: Some(vcf_record),
            molecule_context: MoleculeContext::Genomic.into(),
            allelic_state: Some(allelic_state),
            ..Default::default()
        };
        VariantInterpretation {
            acmg_pathogenicity_classification: AcmgPathogenicityClassification::Pathogenic.into(),
            therapeutic_actionability: TherapeuticActionability::UnknownActionability.into(),
            variation_descriptor: Some(variation_descriptor),
        }
    }
}
