#![allow(unused)]
use crate::hgvs::error::HGVSError;
use crate::hgvs::hgvs_variant::HgvsVariant;
use crate::hgvs::json_schema::VariantValidatorResponse;
use crate::hgvs::traits::HGVSData;
use crate::hgvs::utils::{is_c_hgvs, is_n_hgvs};
use ratelimit::Ratelimiter;
use reqwest::blocking::Client;
use serde_json::Value;
use std::thread::sleep;
use std::time::Duration;

const GENOME_ASSEMBLY_HG38: &str = "hg38";

pub struct HGVSClient {
    rate_limiter: Ratelimiter,
    api_url: String,
    client: Client,
    genome_assembly: String,
}

impl Default for HGVSClient {
    fn default() -> Self {
        let rate_limiter = Ratelimiter::builder(10, Duration::from_secs(1))
            .max_tokens(10)
            .build()
            .expect("Building rate limiter failed");
        let api_url =
            "https://rest.variantvalidator.org/VariantValidator/variantvalidator/".to_string();
        HGVSClient::new(
            rate_limiter,
            api_url.to_string(),
            Client::new(),
            GENOME_ASSEMBLY_HG38.to_string(),
        )
    }
}

impl HGVSClient {
    pub fn new(
        rate_limiter: Ratelimiter,
        api_url: String,
        client: Client,
        genome_assembly: String,
    ) -> Self {
        HGVSClient {
            rate_limiter,
            api_url,
            client,
            genome_assembly,
        }
    }

    pub fn get_fetch_url(&self, transcript: &str, allele: &str) -> String {
        format!(
            "{}/{}/{}%3A{}/{}?content-type=application%2Fjson",
            self.api_url, self.genome_assembly, transcript, allele, transcript
        )
    }

    fn fetch_request(
        &self,
        fetch_url: String,
        unvalidated_hgvs: &str,
    ) -> Result<VariantValidatorResponse, HGVSError> {
        if let Err(duration) = self.rate_limiter.try_wait() {
            sleep(duration);
        }

        let response = self
            .client
            .get(fetch_url.clone())
            .header("User-Agent", "PIVOT")
            .header("Accept", "application/json")
            .send()
            .map_err(|err| HGVSError::FetchRequest {
                hgvs: unvalidated_hgvs.to_string(),
                err: err.to_string(),
            })?;
        response.json::<VariantValidatorResponse>().map_err(|err| {
            HGVSError::DeserializeVariantValidatorResponseToSchema {
                hgvs: unvalidated_hgvs.to_string(),
                err: err.to_string(),
            }
        })
    }
}

impl HGVSData for HGVSClient {
    fn request_and_validate_hgvs(&self, unvalidated_hgvs: &str) -> Result<HgvsVariant, HGVSError> {
        let (transcript, allele) = Self::get_transcript_and_allele(unvalidated_hgvs)?;
        if !is_c_hgvs(allele) && !is_n_hgvs(allele) {
            return Err(HGVSError::HgvsFormatNotAccepted {
                hgvs: unvalidated_hgvs.to_string(),
                problem: "Allele did not begin with c. or n.".to_string(),
            });
        }

        let fetch_url = self.get_fetch_url(transcript, allele);

        let response = self.fetch_request(fetch_url.clone(), unvalidated_hgvs)?;

        if response.flag != "gene_variant" {
            return Err(HGVSError::NonGeneVariant {
                hgvs: unvalidated_hgvs.to_string(),
                flag: response.flag.clone(),
            });
        }

        let variant_info = response.variant_info[unvalidated_hgvs].clone();

        let assemblies = variant_info.primary_assembly_loci;

        let assembly = assemblies
            .get(&self.genome_assembly)
            .ok_or_else(|| HGVSError::GenomeAssemblyNotFound {
                hgvs: unvalidated_hgvs.to_string(),
                desired_assembly: self.genome_assembly.clone(),
                found_assemblies: assemblies.keys().cloned().collect::<Vec<String>>(),
            })?
            .clone();

        let position_string = assembly.vcf.pos;
        let position = position_string.parse::<u32>().map_err(|_| {
            HGVSError::InvalidVariantValidatorResponseElement {
                hgvs: unvalidated_hgvs.to_string(),
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
            self.genome_assembly.clone(),
            assembly.vcf.chr,
            position,
            assembly.vcf.reference,
            assembly.vcf.alt,
            variant_info.gene_ids.hgnc_id,
            variant_info.gene_symbol,
            transcript.to_string(),
            allele.to_string(),
            unvalidated_hgvs.to_string(),
            assembly.hgvs_genomic_description,
            p_hgvs,
        );
        Ok(validated_hgvs)
    }
}

impl HGVSClient {
    fn get_transcript_and_allele(unvalidated_hgvs: &str) -> Result<(&str, &str), HGVSError> {
        let colon_count = unvalidated_hgvs.matches(':').count();
        if colon_count != 1 {
            Err(HGVSError::HgvsFormatNotAccepted {
                hgvs: unvalidated_hgvs.to_string(),
                problem: "There must be exactly one colon in a HGVS string.".to_string(),
            })
        } else {
            let split_hgvs = unvalidated_hgvs.split(':').collect::<Vec<&str>>();
            let transcript = split_hgvs[0];
            let allele = split_hgvs[1];
            Ok((transcript, allele))
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::hgvs::error::HGVSError;
    use crate::hgvs::hgvs_client::HGVSClient;
    use crate::hgvs::traits::HGVSData;
    use rstest::rstest;

    #[rstest]
    fn test_request_and_validate_hgvs_c() {
        let unvalidated_hgvs = "NM_001173464.1:c.2860C>T";
        let client = HGVSClient::default();
        let validated_hgvs = client.request_and_validate_hgvs(unvalidated_hgvs).unwrap();
        assert_eq!(validated_hgvs.transcript_hgvs(), unvalidated_hgvs);
    }

    #[rstest]
    fn test_request_and_validate_hgvs_n() {
        let unvalidated_hgvs = "NR_002196.1:n.601G>T";
        let client = HGVSClient::default();
        let validated_hgvs = client.request_and_validate_hgvs(unvalidated_hgvs).unwrap();
        assert_eq!(validated_hgvs.transcript_hgvs(), unvalidated_hgvs);
    }

    #[rstest]
    fn test_request_and_validate_hgvs_wrong_reference_base_err() {
        let unvalidated_hgvs = "NM_001173464.1:c.2860G>T";
        let client = HGVSClient::default();
        let result = client.request_and_validate_hgvs(unvalidated_hgvs);
        assert!(matches!(
            result,
            Err(HGVSError::DeserializeVariantValidatorResponseToSchema { .. })
        ));
    }

    #[rstest]
    fn test_request_and_validate_hgvs_not_c_or_n_hgvs_err() {
        let unvalidated_hgvs = "NC_000012.12:g.39332405G>A";
        let client = HGVSClient::default();
        let result = client.request_and_validate_hgvs(unvalidated_hgvs);
        assert!(matches!(
            result,
            Err(HGVSError::HgvsFormatNotAccepted { .. })
        ));
    }
}
