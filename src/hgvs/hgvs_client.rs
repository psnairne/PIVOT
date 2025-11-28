use crate::hgvs::error::HGVSError;
use crate::hgvs::json_schema::VariantValidatorResponse;
use crate::hgvs::traits::HGVSData;
use crate::hgvs::validated_hgvs::ValidatedHgvs;
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
    fn request_and_validate_hgvs(&self, unvalidated_hgvs: &str) -> Result<ValidatedHgvs, HGVSError> {
        let (transcript, allele) = Self::get_transcript_and_allele(unvalidated_hgvs)?;

        let fetch_url = self.get_fetch_url(transcript, allele);

        let response = self.fetch_request(fetch_url.clone());

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

        let chr = assembly.vcf.chr;
        let position_string = assembly.vcf.pos;
        let position = position_string.parse::<u64>().map_err(|_| {
            HGVSError::InvalidVariantValidatorResponseElement {
                hgvs: unvalidated_hgvs.to_string(),
                element: position_string,
                problem: "position should be parseable to u32".to_string(),
            }
        })?;
        let ref_allele = assembly.vcf.reference;
        let alt_allele = assembly.vcf.alt;

        let gene_symbol = variant_info.gene_symbol;
        let hgnc_id = variant_info.gene_ids.hgnc_id;
        
        let blah = variant_info.hgvs_transcript_variant

        let hgvs_predicted_protein_consequence = if variant_info
            .hgvs_predicted_protein_consequence
            .tlr
            .is_empty()
        {
            None
        } else {
            Some(variant_info.hgvs_predicted_protein_consequence.tlr)
        };

        let genomic_hgvs = assembly.hgvs_genomic_description;

        let validated_hgvs = ValidatedHgvs::new(
            self.genome_assembly.clone(),
            chr,
            position,
            ref_allele,
            alt_allele,
            hgnc_id,
            gene_symbol,
            transcript.to_string(),
            allele.to_string(),
            genomic_hgvs,
            hgvs_predicted_protein_consequence,
        );
        Ok(validated_hgvs)
    }
}

impl HGVSClient {
    fn get_transcript_and_allele(unvalidated_hgvs: &str) -> Result<(&str, &str), HGVSError> {
        let colon_count = unvalidated_hgvs.matches(':').count();
        if colon_count != 1 {
            Err(HGVSError::IncorrectHGVSFormat {
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
    use crate::hgvs::hgvs_client::HGVSClient;
    use crate::hgvs::traits::HGVSData;
    use rstest::rstest;

    #[rstest]
    fn test_request() {
        let hgvs = "NM_001173464.1:c.2860C>T";
        let client = HGVSClient::default();

        client.request_and_validate_hgvs(hgvs).unwrap();
    }
}
