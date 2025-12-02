#![allow(unused)]
use crate::cache_structs_and_traits::cacher::Cacher;
use crate::hgvs::error::HGVSError;
use crate::hgvs::hgvs_client::HGVSClient;
use crate::hgvs::traits::HGVSData;
use crate::hgvs::validated_c_hgvs::ValidatedCHgvs;

pub struct CachedHGVSClient {
    cacher: Cacher<ValidatedCHgvs>,
    hgvs_client: HGVSClient,
}

impl CachedHGVSClient {
    pub fn new(cacher: Cacher<ValidatedCHgvs>, hgvs_client: HGVSClient) -> Result<Self, HGVSError> {
        cacher.init_cache()?;
        Ok(CachedHGVSClient {
            cacher,
            hgvs_client,
        })
    }
}

impl HGVSData for CachedHGVSClient {
    fn request_and_validate_c_hgvs(
        &self,
        unvalidated_c_hgvs: &str,
    ) -> Result<ValidatedCHgvs, HGVSError> {
        let cache = self.cacher.open_cache()?;
        if let Some(validated_c_hgvs) = self.cacher.find_cache_entry(unvalidated_c_hgvs, &cache) {
            return Ok(validated_c_hgvs);
        }

        let validated_c_hgvs = self
            .hgvs_client
            .request_and_validate_c_hgvs(unvalidated_c_hgvs)?;
        self.cacher.cache_object(validated_c_hgvs.clone(), &cache)?;
        Ok(validated_c_hgvs.clone())
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
    fn test_request_and_validate_c_hgvs(temp_dir: TempDir) {
        let unvalidated_c_hgvs = "NM_001173464.1:c.2860C>T";
        let cache_file_path = temp_dir.path().join("cache.hgvs");
        let client =
            CachedHGVSClient::new(Cacher::new(cache_file_path), HGVSClient::default()).unwrap();

        let validated_c_hgvs = client
            .request_and_validate_c_hgvs(unvalidated_c_hgvs)
            .unwrap();
        assert_eq!(validated_c_hgvs.c_hgvs(), unvalidated_c_hgvs);
    }

    #[rstest]
    fn test_cache(temp_dir: TempDir) {
        let unvalidated_c_hgvs = "NM_001173464.1:c.2860C>T";
        let cache_file_path = temp_dir.path().join("cache.hgvs");
        let client =
            CachedHGVSClient::new(Cacher::new(cache_file_path), HGVSClient::default()).unwrap();

        client
            .request_and_validate_c_hgvs(unvalidated_c_hgvs)
            .unwrap();

        let cache = client.cacher.open_cache().unwrap();
        let cached_hgvs = client
            .cacher
            .find_cache_entry(unvalidated_c_hgvs, &cache)
            .unwrap();
        assert_eq!(cached_hgvs.c_hgvs(), unvalidated_c_hgvs);
    }
}
