#![allow(unused)]

use crate::cache_structs_and_traits::cacher::Cacher;
use crate::hgvs::error::HGVSError;
use crate::hgvs::hgvs_client::HGVSClient;
use crate::hgvs::hgvs_variant::HgvsVariant;
use crate::hgvs::traits::HGVSData;
use std::path::PathBuf;

pub struct CachedHGVSClient {
    cacher: Cacher<HgvsVariant>,
    hgvs_client: HGVSClient,
}

impl CachedHGVSClient {
    pub fn new(cache_file_path: PathBuf, hgvs_client: HGVSClient) -> Result<Self, HGVSError> {
        let cacher = Cacher::new(cache_file_path);
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
        if let Some(validated_hgvs) = self.cacher.find_cache_entry(unvalidated_hgvs, &cache) {
            return Ok(validated_hgvs);
        }

        let validated_hgvs = self
            .hgvs_client
            .request_and_validate_hgvs(unvalidated_hgvs)?;
        self.cacher.cache_object(validated_hgvs.clone(), &cache)?;
        Ok(validated_hgvs.clone())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cache_structs_and_traits::cacher::Cacheable;
    use redb::{Database as RedbDatabase, ReadableDatabase};
    use rstest::{fixture, rstest};
    use tempfile::TempDir;

    #[fixture]
    fn temp_dir() -> TempDir {
        tempfile::tempdir().expect("Failed to create temporary directory")
    }

    #[rstest]
    fn test_request_and_validate_hgvs(temp_dir: TempDir) {
        let unvalidated_hgvs = "NM_001173464.1:c.2860C>T";
        let cache_file_path = temp_dir.path().join("cache.hgvs");
        let client = CachedHGVSClient::new(cache_file_path, HGVSClient::default()).unwrap();

        let validated_hgvs = client.request_and_validate_hgvs(unvalidated_hgvs).unwrap();
        assert_eq!(validated_hgvs.transcript_hgvs(), unvalidated_hgvs);
    }

    #[rstest]
    fn test_cache(temp_dir: TempDir) {
        let unvalidated_hgvs = "NM_001173464.1:c.2860C>T";
        let cache_file_path = temp_dir.path().join("cache.hgvs");
        let client = CachedHGVSClient::new(cache_file_path, HGVSClient::default()).unwrap();

        client.request_and_validate_hgvs(unvalidated_hgvs).unwrap();

        let cache = client.cacher.open_cache().unwrap();
        let cached_hgvs = client
            .cacher
            .find_cache_entry(unvalidated_hgvs, &cache)
            .unwrap();
        assert_eq!(cached_hgvs.transcript_hgvs(), unvalidated_hgvs);
    }
}
