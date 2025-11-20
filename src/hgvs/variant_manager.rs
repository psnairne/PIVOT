use crate::error::PivotError;
use crate::hgvs::hgvs_variant_validator::HgvsVariantValidator;
use crate::hgvs::unvalidated_hgvs::UnvalidatedHgvs;
use crate::hgvs::validated_hgvs::ValidatedHgvs;
use std::collections::{HashMap, HashSet};
use std::thread;
use std::time::Duration;

pub struct VariantManager {
    hgvs_validator: HgvsVariantValidator,
    /// Set of all hgvs strings (e.g., NM_blahblahblah:c.123A>T or NM_blahblahblah:DEL Ex 5 todo!) to be validated
    hgvs_set: HashSet<UnvalidatedHgvs>,
    /// HGVS Variants that could be validated. The key is the original allele denomination (e.g., c.1234A>T), not the variantKey
    validated_hgvs: HashMap<String, ValidatedHgvs>,
    ///attempts with VariantValidatorAPI
    no_attempts: u32,
    ///wait time between VariantValidatorAPI calls
    start_latency: u64,
    ///how much the latency increases after each API call
    latency_increase: u64,
}

impl VariantManager {
    pub fn new(hgvs_set: HashSet<UnvalidatedHgvs>) -> Self {
        Self {
            hgvs_validator: HgvsVariantValidator::hg38(),
            hgvs_set,
            validated_hgvs: HashMap::new(),
            no_attempts: 4,
            start_latency: 250,
            latency_increase: 250,
        }
    }

    /// Perform up to no_attempts rounds of validation using the VariantValidator API
    /// For each round, increase the latency between network calls
    pub fn validate_all_variants<F>(&mut self, mut progress_cb: F) -> Result<(), PivotError>
    where
        F: FnMut(u32, u32),
    {
        let mut latency = self.start_latency;
        let mut attempts = 0;

        let mut n_validated = 0;
        let n_hgvs = self.hgvs_set.len() as u32;

        while n_validated < n_hgvs && attempts < self.no_attempts {
            for hgvs in self.hgvs_set.clone() {
                if !hgvs.is_ascii() {
                    return Err(PivotError::NonAsciiCharacter(hgvs.to_string()));
                }
                if self.validate_hgvs(hgvs) {
                    n_validated += 1;
                }
                // sleep to try to avoid network issues; (start at 250 milliseconds, increase as much in each iteration)
                thread::sleep(Duration::from_millis(latency));
                progress_cb(n_validated, n_hgvs);
            }
            latency += self.latency_increase;
            attempts += 1;
        }
        Ok(())
    }

    /// blah
    pub fn validate_hgvs(&mut self, unvalidated_hgvs: UnvalidatedHgvs) -> bool {
        let hgvs_string = unvalidated_hgvs.to_string();
        let variant_key = unvalidated_hgvs.get_variant_key().to_string();
        if self.validated_hgvs.contains_key(&variant_key) {
            true
        } else if let Ok(validated_hgvs) = self.hgvs_validator.validate(&unvalidated_hgvs) {
            self.validated_hgvs.insert(variant_key, validated_hgvs);
            true
        } else {
            eprint!("Could not validate HGVS {}", hgvs_string);
            false
        }
    }

    pub fn get_validated_hgvs(
        &self,
        unvalidated_hgvs: &UnvalidatedHgvs,
    ) -> Result<ValidatedHgvs, PivotError> {
        let variant_key = unvalidated_hgvs.get_variant_key();
        if let Some(var) = self.validated_hgvs.get(variant_key) {
            Ok(var.clone())
        } else {
            self.hgvs_validator.validate(unvalidated_hgvs)
        }
    }
}

impl Default for VariantManager {
    fn default() -> Self {
        Self::new(HashSet::new())
    }
}
