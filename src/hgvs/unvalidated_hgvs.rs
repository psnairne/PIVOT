use crate::error::PivotError;
use serde::{Deserialize, Serialize};
use std::fmt::Display;

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UnvalidatedHgvs {
    /// blah give example
    transcript: String,
    /// blah
    allele: String,
    /// blah
    variant_key: String,
}

impl UnvalidatedHgvs {
    pub fn new(transcript: String, allele: String, variant_key: String) -> Self {
        UnvalidatedHgvs {
            transcript,
            allele,
            variant_key,
        }
    }

    pub fn new_from_strs(transcript: &str, allele: &str) -> Self {
        let transcript_string = transcript.to_string();
        let allele_string = allele.to_string();
        let variant_key = Self::generate_variant_key(transcript, allele);
        UnvalidatedHgvs {
            transcript: transcript_string,
            allele: allele_string,
            variant_key,
        }
    }

    pub fn get_variant_key(&self) -> &str {
        &self.variant_key
    }

    pub fn get_transcript(&self) -> &str {
        &self.transcript
    }

    pub fn get_allele(&self) -> &str {
        &self.allele
    }

    pub fn is_ascii(&self) -> bool {
        self.transcript.is_ascii() && self.allele.is_ascii()
    }

    pub fn from_hgvs_string(hgvs: &str) -> Result<UnvalidatedHgvs, PivotError> {
        let colon_count = hgvs.matches(':').count();
        if colon_count != 1 {
            Err(PivotError::IncorrectHGVSFormat {
                hgvs: hgvs.to_string(),
                problem: "There must be exactly one colon in a HGVS string.".to_string(),
            })
        } else {
            let split_hgvs = hgvs.split(':').collect::<Vec<&str>>();
            let transcript = split_hgvs[0];
            let allele = split_hgvs[1];
            Ok(UnvalidatedHgvs::new_from_strs(transcript, allele))
        }
    }

    /// blah
    pub fn generate_variant_key(transcript: &str, allele: &str) -> String {
        let transcript_norm = transcript.replace('.', "v");

        let mut allele_norm = allele.replace("c.", "c").replace('>', "to");
        allele_norm = allele_norm
            .chars()
            .map(|c| if c.is_alphanumeric() { c } else { '_' })
            .collect();

        format!("{}_{}", transcript_norm, allele_norm)
    }
}

impl Display for UnvalidatedHgvs {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::result::Result<(), std::fmt::Error> {
        write!(f, "{}:{}", self.transcript, self.allele)
    }
}
