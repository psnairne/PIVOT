use crate::json_schema::{Metadata, SingleVariantInfo, VariantValidatorResponse};
use crate::utils::get_transcript_and_allele;
use crate::{GenomeAssembly, HGVSError, HgvsVariant};

/// This struct contains all the data about a single variant returned by VariantValidator.
#[derive(Default, Debug, Clone)]
pub struct SingleVariantResponse {
    pub transcript_hgvs: String,
    pub single_variant_info: SingleVariantInfo,
    pub flag: String,
    pub metadata: Metadata,
}

impl TryFrom<VariantValidatorResponse> for SingleVariantResponse {
    type Error = HGVSError;
    fn try_from(vv_response: VariantValidatorResponse) -> Result<Self, Self::Error> {
        let no_variant_infos = vv_response.variant_info.len();
        let (transcript_hgvs, single_variant_info) = if no_variant_infos == 1 {
            vv_response.variant_info.into_iter().next().unwrap()
        } else {
            return Err(HGVSError::WrongNumberOfVariantInfos {
                expected: 1,
                found: no_variant_infos,
            });
        };
        Ok(SingleVariantResponse {
            transcript_hgvs,
            single_variant_info,
            flag: vv_response.flag,
            metadata: vv_response.metadata,
        })
    }
}

impl SingleVariantResponse {
    pub fn abbreviate_response(
        self,
        genome_assembly: GenomeAssembly,
    ) -> Result<HgvsVariant, HGVSError> {
        let transcript_hgvs = self.transcript_hgvs;

        let (transcript, allele) = get_transcript_and_allele(transcript_hgvs.as_str())?;

        let variant_info = self.single_variant_info;

        let assemblies = variant_info.primary_assembly_loci;

        let assembly = assemblies
            .get(&genome_assembly.to_string())
            .ok_or_else(|| HGVSError::GenomeAssemblyNotFound {
                hgvs: transcript_hgvs.to_string(),
                desired_assembly: genome_assembly.to_string(),
                found_assemblies: assemblies.keys().cloned().collect::<Vec<String>>(),
            })?
            .clone();

        let position_string = assembly.vcf.pos;
        let position = position_string.parse::<u32>().map_err(|_| {
            HGVSError::InvalidVariantValidatorResponseElement {
                hgvs: transcript_hgvs.clone(),
                element: position_string,
                problem: "position should be parseable to u32".to_string(),
            }
        })?;

        let p_hgvs = if variant_info
            .hgvs_predicted_protein_consequence
            .tlr
            .is_empty()
        {
            None
        } else {
            Some(variant_info.hgvs_predicted_protein_consequence.tlr)
        };

        let validated_hgvs = HgvsVariant::new(
            genome_assembly.to_string(),
            assembly.vcf.chr,
            position,
            assembly.vcf.reference,
            assembly.vcf.alt,
            variant_info.gene_symbol,
            variant_info.gene_ids.hgnc_id,
            transcript.to_string(),
            allele.to_string(),
            transcript_hgvs.to_string(),
            assembly.hgvs_genomic_description,
            p_hgvs,
        );
        Ok(validated_hgvs)
    }
}

#[cfg(test)]
mod tests {
    use crate::json_schema::{
        Annotations, DbXref, ExonicPosition, GeneIds, Metadata, PredictedProteinConsequence,
        PrimaryAssemblyLoci, ReferenceSequenceRecords, SingleVariantInfo, VariantExonicPositions,
        VcfCoordinates,
    };
    use crate::{GenomeAssembly, SingleVariantResponse};
    use rstest::rstest;
    use std::collections::HashMap;

    pub fn single_variant_response() -> SingleVariantResponse {
        let mut primary_assembly_loci = HashMap::new();

        primary_assembly_loci.insert(
            "grch37".to_string(),
            PrimaryAssemblyLoci {
                hgvs_genomic_description: "NC_000012.11:g.39726207G>A".to_string(),
                vcf: VcfCoordinates {
                    alt: "A".to_string(),
                    chr: "12".to_string(),
                    pos: "39726207".to_string(),
                    reference: "G".to_string(),
                },
            },
        );

        primary_assembly_loci.insert(
            "hg19".to_string(),
            PrimaryAssemblyLoci {
                hgvs_genomic_description: "NC_000012.11:g.39726207G>A".to_string(),
                vcf: VcfCoordinates {
                    alt: "A".to_string(),
                    chr: "chr12".to_string(),
                    pos: "39726207".to_string(),
                    reference: "G".to_string(),
                },
            },
        );

        primary_assembly_loci.insert(
            "hg38".to_string(),
            PrimaryAssemblyLoci {
                hgvs_genomic_description: "NC_000012.12:g.39332405G>A".to_string(),
                vcf: VcfCoordinates {
                    alt: "A".to_string(),
                    chr: "chr12".to_string(),
                    pos: "39332405".to_string(),
                    reference: "G".to_string(),
                },
            },
        );

        primary_assembly_loci.insert(
            "grch38".to_string(),
            PrimaryAssemblyLoci {
                hgvs_genomic_description: "NC_000012.12:g.39332405G>A".to_string(),
                vcf: VcfCoordinates {
                    alt: "A".to_string(),
                    chr: "12".to_string(),
                    pos: "39332405".to_string(),
                    reference: "G".to_string(),
                },
            },
        );

        let mut exonic_positions = HashMap::new();
        exonic_positions.insert(
            "NC_000012.11".to_string(),
            ExonicPosition {
                start_exon: "21".to_string(),
                end_exon: "21".to_string(),
            },
        );
        exonic_positions.insert(
            "NC_000012.12".to_string(),
            ExonicPosition {
                start_exon: "21".to_string(),
                end_exon: "21".to_string(),
            },
        );
        exonic_positions.insert(
            "NG_017067.1".to_string(),
            ExonicPosition {
                start_exon: "21".to_string(),
                end_exon: "21".to_string(),
            },
        );

        SingleVariantResponse {
            transcript_hgvs: "NM_001173464.1:c.2860C>T".to_string(),
            single_variant_info: SingleVariantInfo {
                alt_genomic_loci: vec![],
                annotations: Annotations {
                    chromosome: "12".to_string(),
                    db_xref: DbXref {
                        ccds: Some("CCDS53776.1".to_string()),
                        ensemblgene: None,
                        hgnc: "HGNC:19349".to_string(),
                        ncbigene: "55605".to_string(),
                        select: false,
                    },
                    ensembl_select: false,
                    mane_plus_clinical: false,
                    mane_select: false,
                    map: "12q12".to_string(),
                    note: "kinesin family member 21A".to_string(),
                    refseq_select: false,
                    variant: "1".to_string(),
                },
                gene_ids: GeneIds {
                    ccds_ids: vec![],
                    ensembl_gene_id: "ENSG00000139116".to_string(),
                    entrez_gene_id: "55605".to_string(),
                    hgnc_id: "HGNC:19349".to_string(),
                    omim_id: vec!["608283".to_string()],
                    ucsc_id: "uc001rly.4".to_string(),
                },
                gene_symbol: "KIF21A".to_string(),
                genome_context_intronic_sequence: "".to_string(),
                hgvs_lrg_transcript_variant: "".to_string(),
                hgvs_lrg_variant: "".to_string(),
                hgvs_predicted_protein_consequence: PredictedProteinConsequence {
                    lrg_slr: "".to_string(),
                    lrg_tlr: "".to_string(),
                    slr: "NP_001166935.1:p.(R954W)".to_string(),
                    tlr: "NP_001166935.1:p.(Arg954Trp)".to_string(),
                },
                hgvs_refseqgene_variant: "NG_017067.1:g.115986C>T".to_string(),
                hgvs_transcript_variant: "NM_001173464.1:c.2860C>T".to_string(),
                lovd_corrections: None,
                lovd_messages: None,
                primary_assembly_loci,
                reference_sequence_records: ReferenceSequenceRecords {
                    transcript: "https://www.ncbi.nlm.nih.gov/nuccore/NM_001173464.1".to_string(),
                },
                refseqgene_context_intronic_sequence: "".to_string(),
                rna_variant_descriptions: None,
                selected_assembly: "hg38".to_string(),
                submitted_variant: "NM_001173464.1:c.2860C>T".to_string(),
                transcript_description: "Homo sapiens kinesin family member 21A (KIF21A), transcript variant 1, mRNA".to_string(),
                validation_warnings: vec![
                    "TranscriptVersionWarning: A more recent version of the selected reference sequence NM_001173464.1 is available for genome build GRCh38 (NM_001173464.2)".to_string(),
                ],
                variant_exonic_positions: VariantExonicPositions {
                    exonic_positions,
                },
            },
            flag: "gene_variant".to_string(),
            metadata: Metadata {
                variantvalidator_hgvs_version: "2.2.1.dev17+gd620dd190".to_string(),
                variantvalidator_version: "3.0.2.dev143+g6213c80fe".to_string(),
                vvdb_version: "vvdb_2025_3".to_string(),
                vvseqrepo_db: "VV_SR_2025_02/master".to_string(),
                vvta_version: "vvta_2025_02".to_string(),
            },
        }
    }

    #[rstest]
    fn test_abbreviate_data() {
        let example_data = single_variant_response();
        let abbreviated_data = example_data
            .clone()
            .abbreviate_response(GenomeAssembly::Hg38)
            .unwrap();
        assert_eq!(
            abbreviated_data.transcript_hgvs(),
            example_data.transcript_hgvs
        );
    }
}
