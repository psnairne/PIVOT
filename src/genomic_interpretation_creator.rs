use crate::error::PivotError;
use crate::hgnc::traits::HGNCData;
use crate::hgvs_variant::HgvsVariant;
use crate::pathogenic_gene_variant_data::PathogenicGeneVariantData;
use phenopackets::ga4gh::vrsatile::v1::GeneDescriptor;
use phenopackets::schema::v2::core::GenomicInterpretation;
use phenopackets::schema::v2::core::genomic_interpretation::Call;
use regex::Regex;
use crate::hgnc::enums::GeneQuery;

pub struct GenomicInterpretationCreator<T: HGNCData> {
    hgnc_client: T,
    hgnc_id_regex: Regex
}

impl<T> GenomicInterpretationCreator<T>
where
    T: HGNCData,
{
    pub fn new(hgnc_client: T) -> GenomicInterpretationCreator<T> {
        GenomicInterpretationCreator { hgnc_client, hgnc_id_regex: Regex::new("^HGNC:[0-9_]+$").expect("Invalid regex") }
    }

    /// Takes the PathogenicGeneVariantData enum and outputs the appropriate GenomicInterpretation Phenopacket element.
    ///
    /// PathogenicGeneVariantData --> vec![]
    /// CausativeGene --> A single GenomicInterpretation with Call = GeneDescriptor
    /// HeterozygousVariant --> A single GenomicInterpretation with Call = VariantInterpretation and allelic_state = Heterozygous
    /// HomozygousVariant --> A single GenomicInterpretation with Call = VariantInterpretation and allelic_state = Homozygous
    /// CompoundHeterozygousVariantPair --> Two separate GenomicInterpretations with Call = VariantInterpretation and allelic_state = Heterozygous
    ///
    /// Validation:
    ///
    /// - The variants must be in HGVS format and will be externally validated by VariantValidator
    ///
    /// If PathogenicGeneVariantData = HeterozygousVariant | HomozygousVariant | CompoundHeterozygousVariantPair and if gene = Some then it will be validated that the variants lie within this gene.
    pub fn create(
        &self,
        patient_id: &str,
        gene_variant_data: &PathogenicGeneVariantData,
    ) -> Result<Vec<GenomicInterpretation>, PivotError> {
        let mut genomic_interpretations = vec![];

        if !matches!(gene_variant_data, &PathogenicGeneVariantData::None) {
            if let PathogenicGeneVariantData::CausativeGene(gene) = gene_variant_data {
                let request_query = match self.hgnc_id_regex.is_match(gene){
                    true => {GeneQuery::HgncId(gene)}
                    false => {GeneQuery::Symbol(gene)}
                };
                let (hgnc_id, gene_symbol) = self.hgnc_client.request_gene_identifier_pair(request_query)?;

                let gi = GenomicInterpretation {
                    subject_or_biosample_id: patient_id.to_string(),
                    call: Some(Call::Gene(GeneDescriptor {
                        value_id: hgnc_id,
                        symbol: gene_symbol,
                        ..Default::default()
                    })),
                    ..Default::default()
                };
                genomic_interpretations.push(gi);
            }

            if matches!(
                gene_variant_data,
                PathogenicGeneVariantData::HeterozygousVariant { .. }
                    | PathogenicGeneVariantData::HomozygousVariant { .. }
                    | PathogenicGeneVariantData::CompoundHeterozygousVariantPair { .. }
            ) {
                let gene = gene_variant_data.get_gene();
                let allelic_count = gene_variant_data.get_allelic_count();

                for hgvs in gene_variant_data.get_vars() {
                    let gi = Self::get_genomic_interpretation_from_hgvs_data(
                        patient_id,
                        hgvs,
                        gene,
                        allelic_count,
                    )?;
                    genomic_interpretations.push(gi);
                }
            }
        }

        Ok(genomic_interpretations)
    }

    fn get_genomic_interpretation_from_hgvs_data(
        patient_id: &str,
        hgvs: &str,
        gene: Option<&str>,
        allelic_count: usize,
    ) -> Result<GenomicInterpretation, PivotError> {
        let hgvs_variant = HgvsVariant::from_hgvs_string(hgvs)?;

        if let Some(gene) = gene {
            hgvs_variant.validate_against_gene(gene)?;
        }

        let variant_interpretation = hgvs_variant.get_hgvs_variant_interpretation(allelic_count);

        Ok(GenomicInterpretation {
            subject_or_biosample_id: patient_id.to_string(),
            call: Some(Call::VariantInterpretation(variant_interpretation)),
            ..Default::default()
        })
    }
}
