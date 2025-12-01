use redb::{
    Database as RedbDatabase, Database, DatabaseError, ReadableDatabase, TableDefinition, TypeName,
    Value,
};
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

pub struct CachedHGNCClient {
    cache_file_path: PathBuf,
    hgnc_client: HGNCClient,
}

impl HGNCData for CachedHGNCClient {
    fn request_gene_data(&self, query: GeneQuery) -> Result<GeneDoc, HGNCError> {
        let cache = self.open_cache()?;
        if let Some(gene_doc) = Self::find_cache_entry(&query, &cache) {
            return Ok(gene_doc);
        }

        let doc = self.hgnc_client.request_gene_data(query)?;
        CachedHGNCClient::cache_gene_doc(&doc, &cache)?;
        Ok(doc.clone())
    }

    fn request_hgnc_id(&self, symbol: &str) -> Result<String, HGNCError> {
        let doc = self.request_gene_data(GeneQuery::Symbol(symbol))?;

        if let Some(hgnc_id) = &doc.hgnc_id {
            Ok(hgnc_id.clone())
        } else {
            Err(HGNCError::UnexpectedNumberOfDocuments {
                identifier: symbol.to_string(),
                n_found: 0,
                n_expected: 1,
            })
        }
    }

    fn request_gene_symbol(&self, hgnc_id: &str) -> Result<String, HGNCError> {
        let doc = self.request_gene_data(GeneQuery::HgncId(hgnc_id))?;

        if let Some(hgnc_id) = &doc.hgnc_id {
            Ok(hgnc_id.clone())
        } else {
            Err(HGNCError::UnexpectedNumberOfDocuments {
                identifier: hgnc_id.to_string(),
                n_found: 0,
                n_expected: 1,
            })
        }
    }

    fn request_gene_identifier_pair(
        &self,
        query: GeneQuery,
    ) -> Result<(String, String), HGNCError> {
        let doc = self.request_gene_data(query.clone())?;

        if let Some(symbol) = doc.symbol
            && let Some(hgnc_id) = doc.hgnc_id
        {
            return Ok((hgnc_id, symbol));
        }
        Err(HGNCError::UnexpectedNumberOfDocuments {
            identifier: query.inner().to_string(),
            n_found: 0,
            n_expected: 1,
        })
    }
}

impl CachedHGNCClient {
    pub fn new(cache_file_path: PathBuf, hgnc_client: HGNCClient) -> Result<Self, HGNCError> {
        Self::init_cache(&cache_file_path)?;
        Ok(CachedHGNCClient {
            cache_file_path,
            hgnc_client,
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

    pub(super) fn cache_gene_doc(doc: &GeneDoc, cache: &Database) -> Result<(), HGNCError> {
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
    fn open_cache(&self) -> Result<RedbDatabase, DatabaseError> {
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

        CachedHGNCClient::new(cache_dir, HGNCClient::default())
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
        let cache_file_path = temp_dir.path().join("cache.hgnc");
        let client = CachedHGNCClient::new(cache_file_path.clone(), HGNCClient::default()).unwrap();

        let _ = client.request_gene_data(GeneQuery::Symbol(symbol)).unwrap();

        let cache = RedbDatabase::create(&client.cache_file_path).unwrap();
        let cache_reader = cache.begin_read().unwrap();
        let table = cache_reader.open_table(TABLE).unwrap();

        if let Ok(Some(cache_entry)) = table.get(symbol) {
            let value = cache_entry.value();

            assert_eq!(value.hgnc_id.unwrap(), "HGNC:2082");
            assert_eq!(value.symbol.unwrap(), symbol);
        }
    }

    #[rstest]
    #[case(GeneQuery::Symbol("ZNF3"), ("HGNC:13089", "ZNF3"))]
    #[case(GeneQuery::HgncId("HGNC:13089"), ("HGNC:13089", "ZNF3"))]
    fn test_request_gene_identifier_pair(
        #[case] query: GeneQuery,
        #[case] expected_pair: (&str, &str),
    ) {
        let temp_dir = tempfile::tempdir().expect("Failed to create temporary directory");
        let cache_file_path = temp_dir.path().join("cache.hgnc");
        let client = CachedHGNCClient::new(cache_file_path.clone(), HGNCClient::default()).unwrap();
        let client = client
            .with_cache_dir(temp_dir.path().to_path_buf().join("test_cache"))
            .unwrap();
        let gene_doc = client.request_gene_identifier_pair(query).unwrap();

        assert_eq!(gene_doc.0, expected_pair.0);
        assert_eq!(gene_doc.1, expected_pair.1);
    }
}
