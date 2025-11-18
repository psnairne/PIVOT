use redb::{
    Database as RedbDatabase, Database, DatabaseError, ReadableDatabase, TableDefinition, TypeName,
    Value,
};
use std::any::type_name;
use std::env::home_dir;
use std::fmt::{Debug, Formatter};
use std::fs;
use std::path::{Path, PathBuf};

use crate::hgnc::enums::GeneQuery;
use crate::hgnc::error::HGNCError;
use crate::hgnc::hgnc_client::HGNCClient;
use crate::hgnc::json_schema::{GeneDoc, GeneResponse};
use crate::hgnc::traits::HGNCData;
use directories::ProjectDirs;

const TABLE: TableDefinition<&str, GeneDoc> = TableDefinition::new("hgnc_request_cache");

impl GeneDoc {
    fn as_bytes(&self) -> Vec<u8> {
        serde_json::to_vec(self).unwrap()
    }

    fn from_bytes(bytes: &[u8]) -> Result<Self, &'static str> {
        serde_json::from_slice(bytes).map_err(|_| "failed to decode json")
    }

    fn struct_name() -> String {
        type_name::<GeneResponse>()
            .split("::")
            .last()
            .expect("Could not get Struct name")
            .to_string()
    }
}
impl Value for GeneDoc {
    type SelfType<'a> = GeneDoc;
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
        TypeName::new(GeneDoc::struct_name().as_str())
    }
}

pub struct CachedHGNCClient {
    cache_file_path: PathBuf,
    hgnc_client: HGNCClient,
}

impl HGNCData for CachedHGNCClient {
    fn request_gene_data(&self, query: &GeneQuery) -> Result<GeneDoc, HGNCError> {
        let cache = self.cache()?;
        if let Some(gene_doc) = Self::find_cache_entry(query, &cache) {
            return Ok(gene_doc);
        }

        let doc = self.hgnc_client.request_gene_data(query)?;
        CachedHGNCClient::cache_documents(&doc, &cache)?;
        Ok(doc.clone())
    }

    fn request_hgnc_id(&self, symbol: &str) -> Result<String, HGNCError> {
        let doc = self.request_gene_data(&GeneQuery::Symbol(symbol))?;

        if let Some(hgnc_id) = &doc.hgnc_id {
            Ok(hgnc_id.clone())
        } else {
            Err(HGNCError::NoDocumentFound(symbol.to_string()))
        }
    }

    fn request_gene_symbol(&self, hgnc_id: &str) -> Result<String, HGNCError> {
        let doc = self.request_gene_data(&GeneQuery::HgncId(hgnc_id))?;

        if let Some(hgnc_id) = &doc.hgnc_id {
            Ok(hgnc_id.clone())
        } else {
            Err(HGNCError::NoDocumentFound(hgnc_id.to_string()))
        }
    }
}

impl CachedHGNCClient {
    pub fn new(cache_file_path: PathBuf, api_url: String) -> Result<Self, HGNCError> {
        Self::init_cache(&cache_file_path)?;
        Ok(CachedHGNCClient {
            cache_file_path,
            hgnc_client: HGNCClient::new(api_url),
        })
    }

    pub(super) fn find_cache_entry(query: &GeneQuery, cache: &Database) -> Option<GeneDoc> {
        let cache_reader = cache.begin_read().ok()?;
        let table = cache_reader.open_table(TABLE).ok()?;

        if let Ok(Some(cache_entry)) = table.get(query.inner()) {
            return Some(cache_entry.value());
        }

        None
    }

    pub(super) fn cache_documents(doc: &GeneDoc, cache: &Database) -> Result<(), HGNCError> {
        let cache_writer = cache.begin_write()?;
        {
            if let Some(symbol) = &doc.symbol {
                let mut table = cache_writer.open_table(TABLE)?;
                table.insert(symbol.as_str(), doc.clone())?;
            }

            if let Some(hgnc_id) = &doc.hgnc_id {
                let mut table = cache_writer.open_table(TABLE)?;
                table.insert(hgnc_id.as_str(), doc.clone())?;
            }
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

        Some(phenox_cache_dir.join("hgnc_cache"))
    }
}

impl Default for CachedHGNCClient {
    fn default() -> Self {
        let cache_dir = Self::default_cache_dir().expect("Could not find default cache dir.");

        CachedHGNCClient::new(cache_dir, "https://rest.genenames.org/".to_string())
            .expect("Failure when creating HGNC client.")
    }
}

impl Debug for CachedHGNCClient {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("HGNCClient")
            .field("cache_file_path", &self.cache_file_path)
            .field("api_url", &self.hgnc_client)
            .field("rate_limiter", &"<Ratelimiter>")
            .finish()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rstest::{fixture, rstest};
    use tempfile::TempDir;


    #[fixture]
    fn temp_dir() -> TempDir {
        tempfile::tempdir().expect("Failed to create temporary directory")
    }

    #[rstest]
    fn test_cache(temp_dir: TempDir) {
        let symbol = "CLOCK";
        let client = CachedHGNCClient::default();
        let client  =client.with_cache_dir(temp_dir.path().to_path_buf().join("test_cache")).unwrap();

        let _ = client.request_gene_data(&GeneQuery::Symbol(symbol)).unwrap();

        let cache = RedbDatabase::create(&client.cache_file_path).unwrap();
        let cache_reader = cache.begin_read().unwrap();
        let table = cache_reader.open_table(TABLE).unwrap();

        if let Ok(Some(cache_entry)) = table.get(symbol) {
            let value = cache_entry.value();

            assert_eq!(
                value.hgnc_id.unwrap(),
                "HGNC:2082"
            );
            assert_eq!(value.symbol.unwrap(), symbol);
        }

    }
}
