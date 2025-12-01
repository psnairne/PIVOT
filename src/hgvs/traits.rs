use crate::hgvs::error::HGVSError;
use crate::hgvs::validated_c_hgvs::ValidatedCHgvs;

pub trait HGVSData {
    fn request_and_validate_c_hgvs(
        &self,
        unvalidated_c_hgvs: &str,
    ) -> Result<ValidatedCHgvs, HGVSError>;
}
