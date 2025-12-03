#![allow(unused)]
use crate::hgvs::error::HGVSError;
use crate::hgvs::validated_c_hgvs::HgvsVariant;

pub trait HGVSData {
    fn request_and_validate_hgvs(
        &self,
        unvalidated_c_hgvs: &str,
    ) -> Result<HgvsVariant, HGVSError>;
}
