#![allow(unused)]
use crate::hgvs::error::HGVSError;
use crate::hgvs::hgvs_variant::HgvsVariant;

pub trait HGVSData {
    fn request_and_validate_hgvs(&self, unvalidated_hgvs: &str) -> Result<HgvsVariant, HGVSError>;
}
