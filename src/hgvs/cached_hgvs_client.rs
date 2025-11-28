use std::path::PathBuf;
use crate::hgvs::error::HGVSError;
use crate::hgvs::hgvs_client::HGVSClient;
use crate::hgvs::traits::HGVSData;
use crate::hgvs::validated_hgvs::ValidatedHgvs;

pub struct CachedHGVSClient {
    cache_file_path: PathBuf,
    hgvs_client: HGVSClient,
}

impl CachedHGVSClient {
    //todo!
}

impl HGVSData for CachedHGVSClient {
    fn request_and_validate_hgvs(&self, unvalidated_hgvs: &str) -> Result<ValidatedHgvs, HGVSError> {

    }
}