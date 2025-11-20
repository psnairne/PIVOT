use crate::error::PivotError;
use crate::hgvs::hgvs_variant_validator::HgvsVariantValidator;
use crate::hgvs::unvalidated_hgvs::UnvalidatedHgvs;
use crate::hgvs::validated_hgvs::ValidatedHgvs;
use std::collections::HashMap;

pub struct VariantManager {
    hgvs_validator: HgvsVariantValidator,
    /// HGVS Variants that could be validated. The key is the original allele denomination (e.g., c.1234A>T), not the variantKey
    validated_hgvs: HashMap<String, ValidatedHgvs>,
}

impl VariantManager {
    pub fn new() -> Self {
        Self {
            hgvs_validator: HgvsVariantValidator::hg38(),
            validated_hgvs: HashMap::new(),
        }
    }

    /// blah
    pub fn validate_hgvs(
        &mut self,
        unvalidated_hgvs: &UnvalidatedHgvs,
    ) -> Result<&ValidatedHgvs, PivotError> {
        let variant_key = unvalidated_hgvs.get_variant_key().to_string();
        if self.validated_hgvs.contains_key(&variant_key) {
            Ok(self.validated_hgvs.get(&variant_key).unwrap())
        } else if let Ok(validated_hgvs) = self.hgvs_validator.validate(unvalidated_hgvs) {
            self.validated_hgvs
                .insert(variant_key.clone(), validated_hgvs);
            Ok(self.validated_hgvs.get(&variant_key).unwrap())
        } else {
            Err(PivotError::TemporaryError)
        }
    }
}

impl Default for VariantManager {
    fn default() -> Self {
        Self::new()
    }
}
