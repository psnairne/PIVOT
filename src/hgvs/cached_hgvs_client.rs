#![allow(unused)]

use crate::caching::redb_cacher::RedbCacher;
use crate::hgvs::error::HGVSError;
use crate::hgvs::hgvs_client::HGVSClient;
use crate::hgvs::hgvs_variant::HgvsVariant;
use crate::hgvs::traits::HGVSData;
use std::path::PathBuf;

#[derive(Default, Debug)]
pub struct CachedHGVSClient {
    cacher: RedbCacher<HgvsVariant>,
    hgvs_client: HGVSClient,
}


impl CachedHGVSClient {
    pub fn new(cache_file_path: PathBuf, hgvs_client: HGVSClient) -> Result<Self, HGVSError> {
        let cacher = RedbCacher::new(cache_file_path);
        cacher.init_cache()?;
        Ok(CachedHGVSClient {
            cacher,
            hgvs_client,
        })
    }
}

impl HGVSData for CachedHGVSClient {
    fn request_and_validate_hgvs(&self, unvalidated_hgvs: &str) -> Result<HgvsVariant, HGVSError> {
        let cache = self.cacher.open_cache()?;
        if let Some(hgvs_variant) = self.cacher.find_cache_entry(unvalidated_hgvs, &cache) {
            return Ok(hgvs_variant);
        }

        let hgvs_variant = self
            .hgvs_client
            .request_and_validate_hgvs(unvalidated_hgvs)?;
        self.cacher.cache_object(hgvs_variant.clone(), &cache)?;
        Ok(hgvs_variant.clone())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::caching::traits::Cacheable;
    use redb::{Database as RedbDatabase, ReadableDatabase};
    use rstest::{fixture, rstest};
    use tempfile::TempDir;

    #[fixture]
    fn temp_dir() -> TempDir {
        tempfile::tempdir().expect("Failed to create temporary directory")
    }

    #[fixture]
    #[once]
    fn cached_client(temp_dir: TempDir) -> CachedHGVSClient {
        let cache_file_path = temp_dir.path().join("cache.hgvs");
        CachedHGVSClient::new(cache_file_path, HGVSClient::default()).unwrap()
    }

    #[rstest]
    fn test_request_and_validate_hgvs(cached_client: &CachedHGVSClient) {
        let unvalidated_hgvs = "NM_001173464.1:c.2860C>T";
        let validated_hgvs = cached_client.request_and_validate_hgvs(unvalidated_hgvs).unwrap();
        assert_eq!(validated_hgvs.transcript_hgvs(), unvalidated_hgvs);
    }

    #[rstest]
    fn test_cache(cached_client: &CachedHGVSClient) {
        let unvalidated_hgvs = "NM_001173464.1:c.2860C>T";

        cached_client.request_and_validate_hgvs(unvalidated_hgvs).unwrap();

        let cache = cached_client.cacher.open_cache().unwrap();
        let cached_hgvs = cached_client
            .cacher
            .find_cache_entry(unvalidated_hgvs, &cache)
            .unwrap();
        assert_eq!(cached_hgvs.transcript_hgvs(), unvalidated_hgvs);
    }
}
