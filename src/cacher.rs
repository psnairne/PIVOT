use crate::hgvs::validated_c_hgvs::ValidatedCHgvs;
use redb::{
    Database as RedbDatabase, Database, DatabaseError, ReadableDatabase, TableDefinition, TypeName,
    Value,
};
use std::any::type_name;
use std::env::home_dir;
use std::fs;
use std::path::{Path, PathBuf};
use directories::ProjectDirs;
use crate::hgnc::enums::GeneQuery;
use crate::hgnc::error::HGNCError;
use crate::hgnc::json_schema::GeneDoc;

macro_rules! implement_value_for_local_type {
    ($type_name:ty) => {
        impl Value for $type_name {
            type SelfType<'a> = $type_name;
            type AsBytes<'a> = Vec<u8>;

            fn fixed_width() -> Option<usize> {
                None
            }

            fn from_bytes<'a>(data: &[u8]) -> Self::SelfType<'a>
            {
                serde_json::from_slice(data).expect("Could not convert to bytes.")
            }

            fn as_bytes<'a, 'b: 'a>(value: &'a Self::SelfType<'b>) -> Self::AsBytes<'a>
            where
                Self: 'b,
            {
                serde_json::to_vec(value).unwrap()
            }

            fn type_name() -> TypeName {
                TypeName::new(type_name::<$type_name>())
            }
        }
    };
}

implement_value_for_local_type!(ValidatedCHgvs);
implement_value_for_local_type!(GeneDoc);

pub struct Cacher<T: Value> {
    cache_file_path: PathBuf,
    type_to_cache: T,
}

impl<T: Value> Cacher<T> {

    fn init_cache(cache_dir: &Path) -> Result<(), HGNCError> {
        let cache = RedbDatabase::create(cache_dir)?;
        let table: TableDefinition<&str, T> = TableDefinition::new();

        let write_txn = cache.begin_write()?;
        {
            write_txn.open_table(crate::hgnc::cached_hgnc_client::TABLE)?;
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

    pub fn with_cache_dir(mut self, cache_dir: PathBuf) -> Result<Self, HGNCError> {
        Self::init_cache(&cache_dir)?;
        self.cache_file_path = cache_dir.clone();
        Ok(self)
    }

    fn open_cache(&self) -> Result<RedbDatabase, DatabaseError> {
        RedbDatabase::open(&self.cache_file_path)
    }
    pub(super) fn find_cache_entry(query: &GeneQuery, cache: &Database) -> Option<GeneDoc> {
        let cache_reader = cache.begin_read().ok()?;
        let table = cache_reader.open_table(crate::hgnc::cached_hgnc_client::TABLE).ok()?;

        if let Ok(Some(cache_entry)) = table.get(query.inner()) {
            return Some(cache_entry.value());
        }

        None
    }

    pub(super) fn cache_object(doc: &GeneDoc, cache: &Database) -> Result<(), HGNCError> {
        let cache_writer = cache.begin_write()?;
        {
            if let Some(symbol) = &doc.symbol {
                let mut table = cache_writer.open_table(crate::hgnc::cached_hgnc_client::TABLE)?;
                table.insert(symbol.as_str(), doc.clone())?;
            }

            if let Some(hgnc_id) = &doc.hgnc_id {
                let mut table = cache_writer.open_table(crate::hgnc::cached_hgnc_client::TABLE)?;
                table.insert(hgnc_id.as_str(), doc.clone())?;
            }
        }
        cache_writer.commit()?;
        Ok(())
    }

}
