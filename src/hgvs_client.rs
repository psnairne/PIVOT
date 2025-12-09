use crate::SingleVariantResponse;
use crate::enums::GenomeAssembly;
use crate::error::HGVSError;
use crate::json_schema::VariantValidatorResponse;
use crate::traits::HGVSData;
use crate::utils::{get_transcript_and_allele, is_c_hgvs, is_n_hgvs};
use ratelimit::Ratelimiter;
use reqwest::blocking::Client;
use std::thread::sleep;
use std::time::Duration;

pub struct HGVSClient {
    rate_limiter: Ratelimiter,
    api_url: String,
    client: Client,
    genome_assembly: GenomeAssembly,
}

impl Default for HGVSClient {
    fn default() -> Self {
        let rate_limiter = Ratelimiter::builder(2, Duration::from_secs(1))
            .max_tokens(2)
            .build()
            .expect("Building rate limiter failed");
        let api_url =
            "https://rest.variantvalidator.org/VariantValidator/variantvalidator/".to_string();
        HGVSClient::new(
            rate_limiter,
            api_url.to_string(),
            Client::new(),
            GenomeAssembly::Hg38,
        )
    }
}

impl HGVSClient {
    pub fn new(
        rate_limiter: Ratelimiter,
        api_url: String,
        client: Client,
        genome_assembly: GenomeAssembly,
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
    fn get_full_validated_hgvs_data(
        &self,
        unvalidated_hgvs: &str,
    ) -> Result<SingleVariantResponse, HGVSError> {
        let (transcript, allele) = get_transcript_and_allele(unvalidated_hgvs)?;
        if !is_c_hgvs(allele) && !is_n_hgvs(allele) {
            return Err(HGVSError::HgvsFormatNotAccepted {
                hgvs: unvalidated_hgvs.to_string(),
                problem: "Allele did not begin with c. or n.".to_string(),
            });
        }

        let fetch_url = self.get_fetch_url(transcript, allele);

        let response = self.fetch_request(fetch_url.clone(), unvalidated_hgvs)?;

        let single_variant_response = SingleVariantResponse::try_from(response)?;

        if single_variant_response.flag != "gene_variant" {
            Err(HGVSError::NonGeneVariant {
                hgvs: unvalidated_hgvs.to_string(),
                flag: single_variant_response.flag.clone(),
            })
        } else {
            Ok(single_variant_response)
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::error::HGVSError;
    use crate::hgvs_client::HGVSClient;
    use crate::traits::HGVSData;
    use rstest::rstest;

    #[rstest]
    fn test_get_full_validated_hgvs_data_hgvs_c() {
        let unvalidated_hgvs = "NM_001173464.1:c.2860C>T";
        let client = HGVSClient::default();
        let validated_data = client
            .get_full_validated_hgvs_data(unvalidated_hgvs)
            .unwrap();
        assert_eq!(validated_data.transcript_hgvs, unvalidated_hgvs);
    }

    #[rstest]
    fn test_get_full_validated_hgvs_data_hgvs_n() {
        let unvalidated_hgvs = "NR_002196.1:n.601G>T";
        let client = HGVSClient::default();
        let validated_data = client
            .get_full_validated_hgvs_data(unvalidated_hgvs)
            .unwrap();
        assert_eq!(validated_data.transcript_hgvs, unvalidated_hgvs);
    }

    #[rstest]
    fn test_get_full_validated_hgvs_data_wrong_reference_base_err() {
        let unvalidated_hgvs = "NM_001173464.1:c.2860G>T";
        let client = HGVSClient::default();
        let result = client.get_full_validated_hgvs_data(unvalidated_hgvs);
        assert!(matches!(
            result,
            Err(HGVSError::DeserializeVariantValidatorResponseToSchema { .. })
        ));
    }

    #[rstest]
    fn test_get_full_validated_hgvs_data_not_c_or_n_hgvs_err() {
        let unvalidated_hgvs = "NC_000012.12:g.39332405G>A";
        let client = HGVSClient::default();
        let result = client.get_full_validated_hgvs_data(unvalidated_hgvs);
        assert!(matches!(
            result,
            Err(HGVSError::HgvsFormatNotAccepted { .. })
        ));
    }
}
