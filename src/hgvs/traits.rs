use crate::hgvs::error::HGVSError;
use crate::hgvs::validated_hgvs::ValidatedHgvs;

pub trait HGVSData {
    fn request_variant_data(&self, unvalidated_hgvs: &str) -> Result<ValidatedHgvs, HGVSError>;
}
