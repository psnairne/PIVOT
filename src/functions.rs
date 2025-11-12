use ga4ghphetools::dto::hgvs_variant::HgvsVariant;
use ga4ghphetools::variant::variant_manager::VariantManager;
use phenopackets::ga4gh::vrsatile::v1::{Expression, GeneDescriptor, MoleculeContext, VariationDescriptor, VcfRecord};
use phenopackets::schema::v2::core::{AcmgPathogenicityClassification, OntologyClass, TherapeuticActionability, VariantInterpretation};

fn validate_one_hgvs_variant(
    symbol: &str,
    hgnc: &str,
    transcript: &str,
    allele: &str)
    -> Result<HgvsVariant, String> {
    let vmanager = VariantManager::new(symbol, hgnc, transcript);
    vmanager.get_validated_hgvs(allele)
}

fn get_hgvs_variant_interpretation(
    hgvs: &HgvsVariant,
    allele_count: usize,
) -> VariantInterpretation {
    let gene_ctxt = GeneDescriptor {
        value_id: hgvs.hgnc_id().to_string(),
        symbol: hgvs.symbol().to_string(),
        description: String::default(),
        alternate_ids: vec![],
        alternate_symbols: vec![],
        xrefs: vec![],
    };
    let vcf_record = VcfRecord {
        genome_assembly: hgvs.assembly().to_string(),
        chrom: hgvs.chr().to_string(),
        pos: hgvs.position() as u64,
        id: String::default(),
        r#ref: hgvs.ref_allele().to_string(),
        alt: hgvs.alt_allele().to_string(),
        qual: String::default(),
        filter: String::default(),
        info: String::default(),
    };

    let hgvs_c = Expression {
        syntax: "hgvs.c".to_string(),
        value: format!("{}:{}", hgvs.transcript(), hgvs.hgvs()),
        version: String::default(),
    };
    let mut expression_list = vec![hgvs_c];
    let hgvs_g = Expression {
        syntax: "hgvs.g".to_string(),
        value: hgvs.g_hgvs().to_string(),
        version: String::default(),
    };
    expression_list.push(hgvs_g);
    if let Some(hgsvp) = hgvs.p_hgvs() {
        let hgvs_p = Expression {
            syntax: "hgvs.p".to_string(),
            value: hgsvp,
            version: String::default(),
        };
        expression_list.push(hgvs_p);
    };
    let allelic_state = get_allele_term(allele_count, hgvs.is_x_chromosomal());
    let vdesc = VariationDescriptor {
        id: hgvs.variant_key().to_string(),
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

fn get_allele_term(
    allele_count: usize,
    is_x: bool,
) -> OntologyClass {
    if allele_count == 2 {
        OntologyClass {
            id: "GENO:0000136".to_string(),
            label: "homozygous".to_string(),
        }
    } else if is_x {
        OntologyClass {
            id: "GENO:0000134".to_string(),
            label: "hemizygous".to_string(),
        }
    } else {
        OntologyClass {
            id: "GENO:0000135".to_string(),
            label: "heterozygous".to_string(),
        }
    }
}