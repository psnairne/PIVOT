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
