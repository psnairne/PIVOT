use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// If the request is a success, a response with the following structure will be returned
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct VariantValidatorResponse {
    #[serde(flatten)]
    pub variant_info: HashMap<String, SingleVariantInfo>,
    pub flag: String,
    pub metadata: Metadata,
}

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
#[serde(default)]
pub struct SingleVariantInfo {
    pub alt_genomic_loci: Vec<serde_json::Value>, // Uncertain format
    pub annotations: Annotations,
    pub gene_ids: GeneIds,
    pub gene_symbol: String,
    pub genome_context_intronic_sequence: String,
    pub hgvs_lrg_transcript_variant: String,
    pub hgvs_lrg_variant: String,
    pub hgvs_predicted_protein_consequence: PredictedProteinConsequence,
    pub hgvs_refseqgene_variant: String,
    pub hgvs_transcript_variant: String,
    pub lovd_corrections: Option<HashMap<String, u32>>,
    pub lovd_messages: Option<LovdMessages>,
    pub primary_assembly_loci: HashMap<String, PrimaryAssemblyLoci>,
    pub reference_sequence_records: Option<serde_json::Value>, // the format of this seems to change
    pub refseqgene_context_intronic_sequence: String,
    pub rna_variant_descriptions: Option<serde_json::Value>, // Uncertain format
    pub selected_assembly: String,
    pub submitted_variant: String,
    pub transcript_description: String,
    pub validation_warnings: Vec<String>,
    pub variant_exonic_positions: Option<VariantExonicPositions>,
}

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
#[serde(default)]
pub struct Annotations {
    pub chromosome: String,
    pub db_xref: DbXref,
    pub ensembl_select: bool,
    pub mane_plus_clinical: bool,
    pub mane_select: bool,
    pub map: String,
    pub note: String,
    pub refseq_select: bool,
    pub variant: String,
}

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
#[serde(default)]
pub struct DbXref {
    #[serde(rename = "CCDS")]
    pub ccds: Option<String>,
    pub ensemblgene: Option<serde_json::Value>, // Uncertain format
    pub hgnc: String,
    pub ncbigene: String,
    pub select: Option<serde_json::Value>,
}

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
#[serde(default)]
pub struct GeneIds {
    pub ccds_ids: Vec<serde_json::Value>, // Uncertain format
    pub ensembl_gene_id: String,
    pub entrez_gene_id: String,
    pub hgnc_id: String,
    pub omim_id: Vec<String>,
    pub ucsc_id: String,
}

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
#[serde(default)]
pub struct PredictedProteinConsequence {
    pub lrg_slr: String,
    pub lrg_tlr: String,
    pub slr: String,
    pub tlr: String,
}

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
#[serde(default)]
pub struct LovdMessages {
    #[serde(rename = "ISOURCE")]
    pub i_source: String,
    #[serde(rename = "LIBRARYVERSION")]
    pub library_version: String,
}

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
#[serde(default)]
pub struct PrimaryAssemblyLoci {
    pub hgvs_genomic_description: String,
    pub vcf: VcfCoordinates,
}

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
#[serde(default)]
pub struct VcfCoordinates {
    pub alt: String,
    pub chr: String,
    pub pos: String,
    #[serde(rename = "ref")]
    pub reference: String,
}

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
#[serde(default)]
pub struct VariantExonicPositions {
    #[serde(flatten)]
    pub exonic_positions: HashMap<String, ExonicPosition>,
}

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
#[serde(default)]
pub struct ExonicPosition {
    pub start_exon: String,
    pub end_exon: String,
}

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
#[serde(default)]
pub struct Metadata {
    pub variantvalidator_hgvs_version: String,
    pub variantvalidator_version: String,
    pub vvdb_version: String,
    pub vvseqrepo_db: String,
    pub vvta_version: String,
}
