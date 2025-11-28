use std::thread::sleep;
use std::time::Duration;
use ratelimit::Ratelimiter;
use reqwest::blocking::Client;
use crate::error::PivotError;
use crate::hgnc::error::HGNCError;
use crate::hgnc::json_schema::{GeneDoc, GeneResponse};
use crate::hgvs::error::HGVSError;
use crate::hgvs::traits::HGVSData;
use crate::hgvs::validated_hgvs::ValidatedHgvs;
use crate::hgvs::vcf_var::VcfVar;

const GENOME_ASSEMBLY_HG38: &str = "hg38";

pub struct HgvsVariantValidator {
    genome_assembly: String,
}

fn get_variant_validator_url(genome_assembly: &str, transcript: &str, allele: &str) -> String {
    let api_url = format!(
        "https://rest.variantvalidator.org/VariantValidator/variantvalidator/{genome_assembly}/{transcript}%3A{allele}/{transcript}?content-type=application%2Fjson",
    );
    api_url
}

pub struct HGVSClient {
    rate_limiter: Ratelimiter,
    api_url: String,
    client: Client,
    genome_assembly: String,
}

impl HGVSClient {
    pub fn new(api_url: String) -> Self {
        let rate_limiter = Ratelimiter::builder(10, Duration::from_secs(1))
            .max_tokens(10)
            .build()
            .expect("Building rate limiter failed");

        HGVSClient {
            rate_limiter,
            api_url,
            client: Client::new(),
            genome_assembly: GENOME_ASSEMBLY_HG38.to_string(),
        }
    }

    fn fetch_request(&self, url: String) -> Result<Vec<GeneDoc>, HGNCError> {
        if let Err(duration) = self.rate_limiter.try_wait() {
            sleep(duration);
        }
        let response = self
            .client
            .get(url.clone())
            .header("User-Agent", "PIVOT")
            .header("Accept", "application/json")
            .send()?;

        let gene_response = response.json::<GeneResponse>()?;

        Ok(gene_response.response.docs)
    }
}

impl HGVSData for HGVSClient {
    fn request_variant_data(&self, unvalidated_hgvs: &str) -> Result<ValidatedHgvs, HGVSError> {
        let transcript = unvalidated_hgvs.get_transcript();
        let allele = unvalidated_hgvs.get_allele();
        let url = get_variant_validator_url(&self.genome_assembly, transcript, allele);
        let response: Value = get(&url)
            .map_err(|_| PivotError::TemporaryError)?
            .json()
            .map_err(|_| PivotError::TemporaryError)?;
        Self::extract_variant_validator_warnings(&response)?;

        if let Some(flag) = response.get("flag")
            && flag != "gene_variant"
        {
            return Err(PivotError::TemporaryError);
        }

        let variant_key = response
            .as_object()
            .unwrap()
            .keys()
            .find(|&k| k != "flag" && k != "metadata")
            .ok_or_else(|| PivotError::TemporaryError)?;

        let var = &response[variant_key];

        let hgnc = var
            .get("gene_ids")
            .and_then(|ids| ids.get("hgnc_id"))
            .and_then(|v| v.as_str())
            .map(|s| s.to_string())
            .ok_or_else(|| PivotError::TemporaryError)?;

        let symbol = var
            .get("gene_symbol")
            .and_then(|s| s.as_str())
            .map(|s| s.to_string())
            .ok_or_else(|| PivotError::TemporaryError)?;

        // The following will either be a String or None, and can be assigned to an Option<String>
        let hgvs_predicted_protein_consequence = var
            .get("hgvs_predicted_protein_consequence")
            .and_then(|hgvs_protein| hgvs_protein.get("tlr"))
            .and_then(|tlr| tlr.as_str())
            .map(|s| s.to_string());

        let assemblies = var
            .get("primary_assembly_loci")
            .ok_or_else(|| PivotError::TemporaryError)?;

        let assembly = assemblies
            .get(&self.genome_assembly)
            .ok_or_else(|| PivotError::TemporaryError)?;

        let hgvs_transcript_var = var
            .get("hgvs_transcript_variant")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string())
            .ok_or_else(|| PivotError::TemporaryError)?;
        // this field is like NM_000138.5:c.8242G>T - let's just take the first part
        let transcript = hgvs_transcript_var.split(':').next().unwrap_or("");

        let genomic_hgvs = assembly
            .get("hgvs_genomic_description")
            .and_then(|v| v.as_str())
            .map(str::to_string)
            .ok_or_else(|| PivotError::TemporaryError)?;

        let vcf = assembly
            .get("vcf")
            .ok_or_else(|| PivotError::TemporaryError)?;
        let chrom: String = vcf
            .get("chr")
            .and_then(Value::as_str)
            .ok_or_else(|| PivotError::TemporaryError)?
            .to_string();
        let position: u32 = vcf
            .get("pos")
            .and_then(Value::as_str) // "pos" is stored as a string
            .ok_or_else(|| PivotError::TemporaryError)?
            .parse()
            .map_err(|_| PivotError::TemporaryError)?;
        let reference = vcf
            .get("ref")
            .and_then(Value::as_str)
            .ok_or_else(|| PivotError::TemporaryError)?
            .to_string();
        let alternate = vcf
            .get("alt")
            .and_then(Value::as_str)
            .ok_or_else(|| PivotError::TemporaryError)?
            .to_string();
        let vcf_var = VcfVar::new(chrom, position, reference, alternate);
        let hgvs_v = ValidatedHgvs::new(
            self.genome_assembly.clone(),
            vcf_var,
            symbol,
            hgnc,
            allele.to_string(),
            hgvs_predicted_protein_consequence,
            transcript.to_string(),
            genomic_hgvs,
        );
        Ok(hgvs_v)
    }
}

impl HGVSClient {
    fn get_transcript_and_allele(unvalidated_hgvs: &str) -> (&str, &str) {
        
    }
}