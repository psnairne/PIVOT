// This is a Rust port of the Python VariantValidator class
// Dependencies: reqwest = { version = "0.11", features = ["blocking", "json"] }, serde, serde_json, anyhow

use crate::error::PivotError;
use crate::hgvs::unvalidated_hgvs::UnvalidatedHgvs;
use crate::hgvs::validated_hgvs::ValidatedHgvs;
use crate::hgvs::vcf_var::VcfVar;
use reqwest::blocking::get;
use serde_json::Value;

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

impl HgvsVariantValidator {
    pub fn hg38() -> Self {
        Self {
            genome_assembly: GENOME_ASSEMBLY_HG38.to_string(),
        }
    }

    /// Reach out to the VariantValidator API and create an HgvsVariant object from a transcript and HGVS expression
    ///
    /// # Arguments
    ///
    /// * `hgvs` - a Human Genome Variation Society (HGVS) string such as c.123C>T
    /// * `transcript`- the transcript with version number for the HGVS expression
    ///
    /// # Returns
    ///
    /// - `Ok(HgvsVariant)` - An object with information about the variant derived from VariantValidator
    /// - `Err(Error)` - An error if the API call fails (which may happen because of malformed input or network issues).
    pub fn validate(
        &self,
        unvalidated_hgvs: &UnvalidatedHgvs,
    ) -> Result<ValidatedHgvs, PivotError> {
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
        println!("{}", serde_json::to_string_pretty(var).unwrap());

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
        println!("transcript: {transcript} hgvs var tr {hgvs_transcript_var}");

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

    fn extract_variant_validator_warnings(response: &Value) -> Result<(), PivotError> {
        if let Some(flag) = response.get("flag").and_then(|f| f.as_str())
            && flag == "warning"
            && let Some(warnings) = response
                .get("validation_warning_1")
                .and_then(|v| v.get("validation_warnings"))
                .and_then(|w| w.as_array())
        {
            let warning_strings: Vec<String> = warnings
                .iter()
                .filter_map(|w| w.as_str().map(|s| s.to_string()))
                .collect();
            if warning_strings.into_iter().next().is_some() {
                return Err(PivotError::TemporaryError);
            } else {
                // Should never happen, if it does, we need to check parsing of variant validator API.
                return Err(PivotError::TemporaryError);
            }
        }
        Ok(())
    }
}

// region:    --- Tests

#[cfg(test)]
mod tests {
    use rstest::{fixture, rstest};

    use super::*;

    // NM_000138.5(FBN1):c.8230C>T (p.Gln2744Ter)
    #[fixture]
    fn unvalidated_hgvs() -> UnvalidatedHgvs {
        UnvalidatedHgvs::new_from_strs("NM_000138.5", "c.8230C>T")
    }

    /// Invalid version of the above with the wrong nucleotide (G instead of C)
    /// Designed to elicit an error from VariantValidator
    #[fixture]
    fn invalid_unvalidated_hgvs() -> UnvalidatedHgvs {
        UnvalidatedHgvs::new_from_strs("NM_000138.5", "c.8230G>T")
    }

    #[rstest]
    fn test_url(unvalidated_hgvs: UnvalidatedHgvs) {
        let expected = "https://rest.variantvalidator.org/VariantValidator/variantvalidator/hg38/NM_000138.5%3Ac.8230C>T/NM_000138.5?content-type=application%2Fjson";
        let my_url = get_variant_validator_url(
            "hg38",
            unvalidated_hgvs.get_transcript(),
            unvalidated_hgvs.get_allele(),
        );
        assert_eq!(expected, my_url);
    }

    #[rstest]
    fn test_variant_validator(unvalidated_hgvs: UnvalidatedHgvs) {
        let vvalidator = HgvsVariantValidator::hg38();
        let json = vvalidator.validate(&unvalidated_hgvs);
        assert!(json.is_ok());
        let json = json.unwrap();
        println!("{:?}", json);
    }

    #[rstest]
    fn test_variant_validator_invalid(invalid_unvalidated_hgvs: UnvalidatedHgvs) {
        let vvalidator = HgvsVariantValidator::hg38();
        // This is an invalid HGVS because the reference base should be C and not G
        let result = vvalidator.validate(&invalid_unvalidated_hgvs);
        assert!(result.is_err());
        //todo! once I've sorted PIVOT errors here
        /*        if let Err(e) = result {
            assert_eq!(
                "NM_000138.5:c.8230G>T: Variant reference (G) does not agree with reference sequence (C)",
                e
            );
        }*/
    }
}

// endregion: --- Tests
