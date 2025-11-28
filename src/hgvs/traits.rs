use crate::hgvs::error::HGVSError;
use crate::hgvs::validated_hgvs::ValidatedHgvs;

pub trait HGVSData {
    fn request_and_validate_hgvs(&self, unvalidated_hgvs: &str) -> Result<ValidatedHgvs, HGVSError>;
}
