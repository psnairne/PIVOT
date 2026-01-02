#![allow(unused)]

use crate::hgvs::error::HGVSError;
use crate::hgvs::hgvs_variant::HgvsVariant;
use std::fmt::Debug;

pub trait HGVSData: Debug {
    fn request_and_validate_hgvs(&self, unvalidated_hgvs: &str) -> Result<HgvsVariant, HGVSError>;
}
