use crate::error::PivotError;
use crate::hgnc::enums::GeneQuery;
use crate::hgnc::traits::HGNCData;
use crate::hgvs::unvalidated_hgvs::UnvalidatedHgvs;
use crate::hgvs::validated_hgvs::ValidatedHgvs;
use crate::hgvs::variant_manager::VariantManager;
use crate::pathogenic_gene_variant_data::PathogenicGeneVariantData;
use phenopackets::ga4gh::vrsatile::v1::GeneDescriptor;
use phenopackets::schema::v2::core::GenomicInterpretation;
use phenopackets::schema::v2::core::genomic_interpretation::Call;
use regex::Regex;

///This should be created whenever you want to convert some variants and genes into GenomicInterpretations
/// It should be an attribute of the Phenopacket Builder in PhenoXtract
pub struct GenomicInterpretationCreator<T: HGNCData> {
    hgnc_client: T,
    hgnc_id_regex: Regex,
    variant_manager: VariantManager,
}

impl<T> GenomicInterpretationCreator<T>
where
    T: HGNCData,
{
    pub fn new(hgnc_client: T) -> GenomicInterpretationCreator<T> {
        GenomicInterpretationCreator {
            hgnc_client,
            hgnc_id_regex: Regex::new("^HGNC:[0-9_]+$").expect("Invalid regex"),
            variant_manager: VariantManager::default(),
        }
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
        &mut self,
        patient_id: &str,
        gene_variant_data: &PathogenicGeneVariantData,
    ) -> Result<Vec<GenomicInterpretation>, PivotError> {
        let mut genomic_interpretations = vec![];

        if !matches!(gene_variant_data, &PathogenicGeneVariantData::None) {
            if let PathogenicGeneVariantData::CausativeGene(gene) = gene_variant_data {
                let request_query = match self.hgnc_id_regex.is_match(gene) {
                    true => GeneQuery::HgncId(gene),
                    false => GeneQuery::Symbol(gene),
                };
                let (hgnc_id, gene_symbol) = self
                    .hgnc_client
                    .request_gene_identifier_pair(request_query)?;

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
                    let gi = self.get_genomic_interpretation_from_hgvs_data(
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

    /// validates the HGVS using VariantValidator (also validates that the gene is correct)
    /// and returns the Genomic Interpretation if successful
    fn get_genomic_interpretation_from_hgvs_data(
        &mut self,
        patient_id: &str,
        hgvs: &str,
        gene: Option<&str>,
        allelic_count: usize,
    ) -> Result<GenomicInterpretation, PivotError> {
        let unvalidated_hgvs = UnvalidatedHgvs::from_hgvs_string(hgvs)?;

        let mut latency = 250;
        let mut attempts = 1;

        while attempts < 4 {
            let validation_result = self.variant_manager.validate_hgvs(&unvalidated_hgvs, gene);
            if let Ok(validated_hgvs) = validation_result {
                return Ok(Self::create_gi_from_validated_hgvs(
                    patient_id,
                    validated_hgvs,
                    allelic_count,
                ));
            }
            latency += 250;
            attempts += 1;

            std::thread::sleep(std::time::Duration::from_millis(latency));
        }

        let validation_result = self.variant_manager.validate_hgvs(&unvalidated_hgvs, gene);
        match validation_result {
            Ok(validated_hgvs) => Ok(Self::create_gi_from_validated_hgvs(
                patient_id,
                validated_hgvs,
                allelic_count,
            )),
            Err(e) => Err(e),
        }
    }

    /// Given a validated hgvs, this creates the GI
    fn create_gi_from_validated_hgvs(
        patient_id: &str,
        validated_hgvs: &ValidatedHgvs,
        allelic_count: usize,
    ) -> GenomicInterpretation {
        GenomicInterpretation {
            subject_or_biosample_id: patient_id.to_string(),
            call: Some(Call::VariantInterpretation(
                validated_hgvs.get_hgvs_variant_interpretation(allelic_count),
            )),
            ..Default::default()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::hgnc::cached_hgnc_client::CachedHGNCClient;
    use rstest::{fixture, rstest};
    use tempfile::TempDir;

    #[fixture]
    fn temp_dir() -> TempDir {
        tempfile::tempdir().expect("Failed to create temporary directory")
    }

    #[rstest]
    fn test_create() {
        let hgnc_client = CachedHGNCClient::default();
        let mut gi_creator = GenomicInterpretationCreator::new(hgnc_client);

        let variant_gene_data = PathogenicGeneVariantData::HeterozygousVariant {
            hgvs: "NM_001173464.1:c.2860C>T",
            gene: Some("HGNC:19349"),
        };

        let _gi = gi_creator.create("P001", &variant_gene_data).unwrap();
    }
}
