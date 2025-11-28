use crate::hgvs::error::HGVSError;
use crate::hgvs::json_schema::VariantValidatorResponse;
use crate::hgvs::traits::HGVSData;
use crate::hgvs::validated_hgvs::ValidatedHgvs;
use crate::hgvs::vcf_var::VcfVar;
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
    fn request_variant_data(&self, unvalidated_hgvs: &str) -> Result<ValidatedHgvs, HGVSError> {
        let (transcript, allele) = Self::get_transcript_and_allele(unvalidated_hgvs)?;
        let fetch_url = self.get_fetch_url(transcript, allele);
        let response = self.fetch_request(fetch_url.clone());

/*        if let Some(flag) = response.get("flag")
            && flag != "gene_variant"
        {
            return Err(HGVSError::TemporaryError);
        }*/

/*        let variant_key = response
            .as_object()
            .unwrap()
            .keys()
            .find(|&k| k != "flag" && k != "metadata")
            .ok_or_else(|| HGVSError::TemporaryError)?;*/

        let var = response.variant_info[unvalidated_hgvs].clone();

        let hgnc = var
            .gene_ids.hgnc_id;

        let symbol = var.gene_symbol;

        let hgvs_predicted_protein_consequence = var.hgvs_predicted_protein_consequence.tlr;

        let assemblies = var.primary_assembly_loci;

        let assembly = assemblies
            .get(&self.genome_assembly)
            .ok_or_else(|| HGVSError::TemporaryError)?;

        let hgvs_transcript_var = var.hgvs_transcript_variant;
        let transcript = hgvs_transcript_var.split(':').next().unwrap_or("");

        let genomic_hgvs = assembly.hgvs_genomic_description.clone();

        let vcf = assembly.vcf.clone();
        let chrom = vcf.chr;

        let position: u32 = vcf.pos
            .parse()
            .map_err(|_| HGVSError::TemporaryError)?;
        let reference = vcf.reference;
        let alternate = vcf.alt;
        let vcf_var = VcfVar::new(chrom, position, reference, alternate);
        let hgvs_v = ValidatedHgvs::new(
            self.genome_assembly.clone(),
            vcf_var,
            symbol,
            hgnc,
            allele.to_string(),
            Some(hgvs_predicted_protein_consequence),
            transcript.to_string(),
            genomic_hgvs,
        );
        Ok(hgvs_v)
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

        client.request_variant_data(hgvs).unwrap();
    }
}
