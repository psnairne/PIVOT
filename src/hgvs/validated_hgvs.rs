use crate::error::PivotError;
use crate::hgvs::unvalidated_hgvs::UnvalidatedHgvs;
use crate::hgvs::vcf_var::VcfVar;
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
pub struct ValidatedHgvs {
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
    /// HGVS Nomenclature, e.g., c.8242G>T
    allele: String,
    /// Transcript, e.g., NM_000138.5
    transcript: String,
    /// Genomic HGVS nomenclature, e.g., NC_000015.10:g.48411364C>A
    g_hgvs: String,
    /// Protein level HGVS, if available
    p_hgvs: Option<String>,
    /// Key to specify this variant in the HGVS HashMap of the CohortDto
    /// In our implementation for PubMed curation we will also use the key as the variant_id
    /// to export to phenopacket
    variant_key: String,
}

impl ValidatedHgvs {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        assembly: String,
        vcf_var: VcfVar,
        symbol: String,
        hgnc_id: String,
        allele: String,
        p_hgvs: Option<String>,
        transcript: String,
        g_hgvs: String,
    ) -> Self {
        let chr = vcf_var.chrom();
        let position = vcf_var.pos();
        let ref_allele = vcf_var.ref_allele();
        let alt_allele = vcf_var.alt_allele();
        let variant_key = UnvalidatedHgvs::generate_variant_key(&transcript, &allele);

        ValidatedHgvs {
            assembly,
            chr,
            position,
            ref_allele,
            alt_allele,
            symbol,
            hgnc_id,
            allele,
            p_hgvs,
            transcript,
            g_hgvs,
            variant_key,
        }
    }

    #[allow(clippy::too_many_arguments)]
    pub fn new_from_parts(
        assembly: String,
        chromosome: String,
        pos: u32,
        reference: String,
        alternate: String,
        symbol: String,
        hgnc_id: String,
        hgvs: String,
        transcript: String,
        g_hgvs: String,
    ) -> Self {
        let vcf_var = VcfVar::new(chromosome, pos, reference, alternate);
        Self::new(
            assembly, vcf_var, symbol, hgnc_id, hgvs, None, transcript, g_hgvs,
        )
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

    pub fn symbol(&self) -> &str {
        &self.symbol
    }

    pub fn hgnc_id(&self) -> &str {
        self.hgnc_id.as_ref()
    }

    pub fn hgvs(&self) -> &str {
        self.allele.as_ref()
    }

    pub fn p_hgvs(&self) -> Option<String> {
        self.p_hgvs.as_ref().map(|phgvs| phgvs.to_string())
    }

    pub fn transcript(&self) -> &str {
        self.transcript.as_ref()
    }

    pub fn g_hgvs(&self) -> &str {
        self.g_hgvs.as_ref()
    }

    pub fn is_x_chromosomal(&self) -> bool {
        self.chr.contains("X")
    }

    pub fn validate_against_gene(&self, gene: &str) -> Result<(), PivotError> {
        let (expected, id_type) = if is_hgnc_id(gene) {
            (self.hgnc_id.as_str(), "HGNC ID")
        } else {
            (self.symbol.as_str(), "gene symbol")
        };

        if gene == expected {
            Ok(())
        } else {
            Err(PivotError::IncorrectGeneData {
                id_type: id_type.to_string(),
                gene: gene.to_string(),
                hgnc_id: self.hgnc_id.to_string(),
            })
        }
    }

    /// Creates a VI from the ValidatedHGVS
    pub fn get_hgvs_variant_interpretation(&self, allele_count: usize) -> VariantInterpretation {
        let gene_ctxt = GeneDescriptor {
            value_id: self.hgnc_id.to_string(),
            symbol: self.symbol.to_string(),
            description: String::default(),
            alternate_ids: vec![],
            alternate_symbols: vec![],
            xrefs: vec![],
        };
        let vcf_record = VcfRecord {
            genome_assembly: self.assembly.to_string(),
            chrom: self.chr.to_string(),
            pos: self.position as u64,
            id: String::default(),
            r#ref: self.ref_allele.to_string(),
            alt: self.alt_allele.to_string(),
            qual: String::default(),
            filter: String::default(),
            info: String::default(),
        };

        let hgvs_c = Expression {
            syntax: "hgvs.c".to_string(),
            value: format!("{}:{}", self.transcript, self.allele),
            version: String::default(),
        };
        let mut expression_list = vec![hgvs_c];
        let hgvs_g = Expression {
            syntax: "hgvs.g".to_string(),
            value: self.g_hgvs.to_string(),
            version: String::default(),
        };
        expression_list.push(hgvs_g);
        if let Some(hgsvp) = &self.p_hgvs {
            let hgvs_p = Expression {
                syntax: "hgvs.p".to_string(),
                value: hgsvp.clone(),
                version: String::default(),
            };
            expression_list.push(hgvs_p);
        };
        let allelic_state = get_allele_term(allele_count, self.is_x_chromosomal());
        let vdesc = VariationDescriptor {
            id: self.variant_key.to_string(),
            variation: None,
            label: String::default(),
            description: String::default(),
            gene_context: Some(gene_ctxt),
            expressions: expression_list,
            vcf_record: Some(vcf_record),
            xrefs: vec![],
            alternate_labels: vec![],
            extensions: vec![],
            molecule_context: MoleculeContext::Genomic.into(),
            structural_type: None,
            vrs_ref_allele_seq: String::default(),
            allelic_state: Some(allelic_state),
        };
        VariantInterpretation {
            acmg_pathogenicity_classification: AcmgPathogenicityClassification::Pathogenic.into(),
            therapeutic_actionability: TherapeuticActionability::UnknownActionability.into(),
            variation_descriptor: Some(vdesc),
        }
    }
}
