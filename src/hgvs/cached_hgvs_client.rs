use crate::hgnc::error::HGNCError;
use crate::hgvs::error::HGVSError;
use crate::hgvs::hgvs_client::HGVSClient;
use crate::hgvs::traits::HGVSData;
use crate::hgvs::validated_c_hgvs::ValidatedCHgvs;
use directories::ProjectDirs;
use redb::{
    Database as RedbDatabase, Database, DatabaseError, ReadableDatabase, TableDefinition, TypeName,
    Value,
};
use std::any::type_name;
use std::env::home_dir;
use std::fs;
use std::path::{Path, PathBuf};

const TABLE: TableDefinition<&str, ValidatedCHgvs> = TableDefinition::new("validated_c_hgvs_cache");

pub struct CachedHGVSClient {
    cache_file_path: PathBuf,
    hgvs_client: HGVSClient,
}

impl CachedHGVSClient {
    pub fn new(cache_file_path: PathBuf, hgvs_client: HGVSClient) -> Result<Self, HGNCError> {
        Self::init_cache(&cache_file_path)?;
        Ok(CachedHGVSClient {
            cache_file_path,
            hgvs_client,
        })
    }

    pub(super) fn find_cache_entry(
        unvalidated_c_hgvs: &str,
        cache: &Database,
    ) -> Option<ValidatedCHgvs> {
        let cache_reader = cache.begin_read().ok()?;
        let table = cache_reader.open_table(TABLE).ok()?;

        if let Ok(Some(cache_entry)) = table.get(unvalidated_c_hgvs) {
            return Some(cache_entry.value());
        }

        None
    }

    pub(super) fn cache_validated_c_hgvs(
        validated_c_hgvs: &ValidatedCHgvs,
        cache: &Database,
    ) -> Result<(), HGVSError> {
        let cache_writer = cache.begin_write()?;
        {
            let mut table = cache_writer.open_table(TABLE)?;
            table.insert(validated_c_hgvs.c_hgvs(), validated_c_hgvs.clone())?;
        }
        cache_writer.commit()?;
        Ok(())
    }

    pub fn with_cache_dir(mut self, cache_dir: PathBuf) -> Result<Self, HGNCError> {
        Self::init_cache(&cache_dir)?;
        self.cache_file_path = cache_dir.clone();
        Ok(self)
    }

    fn cache(&self) -> Result<RedbDatabase, DatabaseError> {
        RedbDatabase::open(&self.cache_file_path)
    }

    fn init_cache(cache_dir: &Path) -> Result<(), HGNCError> {
        let cache = RedbDatabase::create(cache_dir)?;
        let write_txn = cache.begin_write()?;
        {
            write_txn.open_table(TABLE)?;
        }
        write_txn.commit()?;
        Ok(())
    }

    pub fn default_cache_dir() -> Option<PathBuf> {
        let pkg_name = env!("CARGO_PKG_NAME");

        let phenox_cache_dir = if let Some(project_dir) = ProjectDirs::from("", "", pkg_name) {
            project_dir.cache_dir().to_path_buf()
        } else if let Some(home_dir) = home_dir() {
            home_dir.join(pkg_name)
        } else {
            return None;
        };

        if !phenox_cache_dir.exists() {
            fs::create_dir_all(&phenox_cache_dir).ok()?;
        }

        Some(phenox_cache_dir.join("hgvs_cache"))
    }
}

impl HGVSData for CachedHGVSClient {
    fn request_and_validate_c_hgvs(
        &self,
        unvalidated_c_hgvs: &str,
    ) -> Result<ValidatedCHgvs, HGVSError> {
        let cache = self.cache()?;
        if let Some(response) = Self::find_cache_entry(unvalidated_c_hgvs, &cache) {
            return Ok(response);
        }

        let validated_c_hgvs = self
            .hgvs_client
            .request_and_validate_c_hgvs(unvalidated_c_hgvs)?;
        CachedHGVSClient::cache_validated_c_hgvs(&validated_c_hgvs, &cache)?;
        Ok(validated_c_hgvs.clone())
    }
}

impl ValidatedCHgvs {
    fn as_bytes(&self) -> Vec<u8> {
        serde_json::to_vec(self).unwrap()
    }

    fn from_bytes(bytes: &[u8]) -> Result<Self, &'static str> {
        serde_json::from_slice(bytes).map_err(|_| "failed to decode json")
    }

    fn struct_name() -> String {
        type_name::<ValidatedCHgvs>().to_string()
    }
}

impl Value for ValidatedCHgvs {
    type SelfType<'a> = ValidatedCHgvs;
    type AsBytes<'a> = Vec<u8>;

    fn fixed_width() -> Option<usize> {
        None
    }

    fn from_bytes<'a>(data: &'a [u8]) -> Self::SelfType<'a>
    where
        Self: 'a,
    {
        Self::from_bytes(data).expect("Could not convert to bytes.")
    }

    fn as_bytes<'a, 'b: 'a>(value: &'a Self::SelfType<'b>) -> Self::AsBytes<'a>
    where
        Self: 'b,
    {
        value.as_bytes()
    }

    fn type_name() -> TypeName {
        TypeName::new(ValidatedCHgvs::struct_name().as_str())
    }
}
