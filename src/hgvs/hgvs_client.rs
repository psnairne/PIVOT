use crate::hgvs::error::HGVSError;
use crate::hgvs::json_schema::VariantValidatorResponse;
use crate::hgvs::traits::HGVSData;
use crate::hgvs::validated_c_hgvs::ValidatedCHgvs;
use ratelimit::Ratelimiter;
use reqwest::blocking::Client;
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

    fn fetch_request(&self, fetch_url: String) -> VariantValidatorResponse {
        if let Err(duration) = self.rate_limiter.try_wait() {
            sleep(duration);
        }

        let response = self
            .client
            .get(fetch_url.clone())
            .header("User-Agent", "PIVOT")
            .header("Accept", "application/json")
            .send()
            .unwrap();
        response.json::<VariantValidatorResponse>().unwrap()
    }
}

impl HGVSData for HGVSClient {
    fn request_and_validate_c_hgvs(
        &self,
        unvalidated_c_hgvs: &str,
    ) -> Result<ValidatedCHgvs, HGVSError> {
        let (transcript, allele) = Self::get_transcript_and_allele(unvalidated_c_hgvs)?;
        if !Self::validate_is_c_hgvs(allele) {
            return Err(HGVSError::IncorrectCHGVSFormat {
                c_hgvs: unvalidated_c_hgvs.to_string(),
                problem: "Allele did not begin with c.".to_string(),
            });
        }

        let fetch_url = self.get_fetch_url(transcript, allele);

        let response = self.fetch_request(fetch_url.clone());

        if response.flag != "gene_variant" {
            return Err(HGVSError::NonGeneVariant {
                c_hgvs: unvalidated_c_hgvs.to_string(),
                flag: response.flag.clone(),
            });
        }

        let variant_info = response.variant_info[unvalidated_c_hgvs].clone();

        let assemblies = variant_info.primary_assembly_loci;

        let assembly = assemblies
            .get(&self.genome_assembly)
            .ok_or_else(|| HGVSError::GenomeAssemblyNotFound {
                c_hgvs: unvalidated_c_hgvs.to_string(),
                desired_assembly: self.genome_assembly.clone(),
                found_assemblies: assemblies.keys().cloned().collect::<Vec<String>>(),
            })?
            .clone();

        let position_string = assembly.vcf.pos;
        let position = position_string.parse::<u64>().map_err(|_| {
            HGVSError::InvalidVariantValidatorResponseElement {
                c_hgvs: unvalidated_c_hgvs.to_string(),
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

        let validated_c_hgvs = ValidatedCHgvs::new(
            self.genome_assembly.clone(),
            assembly.vcf.chr,
            position,
            assembly.vcf.reference,
            assembly.vcf.alt,
            variant_info.gene_ids.hgnc_id,
            variant_info.gene_symbol,
            transcript.to_string(),
            allele.to_string(),
            unvalidated_c_hgvs.to_string(),
            assembly.hgvs_genomic_description,
            p_hgvs,
        );
        Ok(validated_c_hgvs)
    }
}

impl HGVSClient {
    fn get_transcript_and_allele(unvalidated_c_hgvs: &str) -> Result<(&str, &str), HGVSError> {
        let colon_count = unvalidated_c_hgvs.matches(':').count();
        if colon_count != 1 {
            Err(HGVSError::IncorrectCHGVSFormat {
                c_hgvs: unvalidated_c_hgvs.to_string(),
                problem: "There must be exactly one colon in a HGVS string.".to_string(),
            })
        } else {
            let split_hgvs = unvalidated_c_hgvs.split(':').collect::<Vec<&str>>();
            let transcript = split_hgvs[0];
            let allele = split_hgvs[1];
            Ok((transcript, allele))
        }
    }

    fn validate_is_c_hgvs(allele: &str) -> bool {
        allele.starts_with("c.")
    }
}

#[cfg(test)]
mod tests {
    use crate::hgvs::hgvs_client::HGVSClient;
    use crate::hgvs::traits::HGVSData;
    use rstest::rstest;

    #[rstest]
    fn test_request() {
        let c_hgvs = "NM_001173464.1:c.2860C>T";
        let client = HGVSClient::default();
        let validated_c_hgvs = client.request_and_validate_c_hgvs(c_hgvs).unwrap();
        dbg!(validated_c_hgvs);
    }
}
