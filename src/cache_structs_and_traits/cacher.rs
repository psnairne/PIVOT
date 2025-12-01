use crate::cache_structs_and_traits::error::CacherError;
use crate::hgnc::json_schema::GeneDoc;
use crate::hgvs::validated_c_hgvs::ValidatedCHgvs;
use directories::ProjectDirs;
use redb::{
    Database as RedbDatabase, Database, DatabaseError, ReadableDatabase, TableDefinition, TypeName,
    Value,
};
use std::any::type_name;
use std::borrow::Borrow;
use std::env::home_dir;
use std::fs;
use std::marker::PhantomData;
use std::path::PathBuf;

macro_rules! implement_value_for_local_type {
    ($type_name:ty) => {
        impl Value for $type_name {
            type SelfType<'a> = $type_name;
            type AsBytes<'a> = Vec<u8>;

            fn fixed_width() -> Option<usize> {
                None
            }

            fn from_bytes<'a>(data: &[u8]) -> Self::SelfType<'a> {
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

pub trait Cacheable: Sized + Clone + Value + 'static
where
    for<'a> Self: From<Self::SelfType<'a>>, // required so that cache_entry.value().into() works
    for<'a> Self: Borrow<Self::SelfType<'a>>,
{
    fn keys(&self) -> Vec<&str>;

    fn table_definition() -> TableDefinition<'static, &'static str, Self> {
        TableDefinition::new(type_name::<Self>())
    }
}

impl Cacheable for ValidatedCHgvs {
    fn keys(&self) -> Vec<&str> {
        vec![self.c_hgvs()]
    }
}

impl Cacheable for GeneDoc {
    fn keys(&self) -> Vec<&str> {
        vec![self.symbol(), self.hgnc_id()]
    }
}

pub struct Cacher<T: Cacheable> {
    cache_file_path: PathBuf,
    _phantom: PhantomData<T>,
}

impl<T: Cacheable> Default for Cacher<T> {
    fn default() -> Self {
        let pkg_name = env!("CARGO_PKG_NAME");

        let phenox_cache_dir = ProjectDirs::from("", "", pkg_name)
            .map(|project_dir| project_dir.cache_dir().to_path_buf())
            .or_else(|| home_dir().map(|home| home.join(pkg_name)))
            .unwrap_or_else(|| panic!("Could not find cache directory or home directory."));

        if !phenox_cache_dir.exists() {
            fs::create_dir_all(&phenox_cache_dir)
                .expect("Failed to create default cache directory.");
        }

        Cacher::new(phenox_cache_dir.join(type_name::<T>()))
    }
}

impl<T: Cacheable> Cacher<T> {
    pub fn new(cache_file_path: PathBuf) -> Self {
        Cacher {
            cache_file_path,
            _phantom: PhantomData,
        }
    }

    pub fn cache_file_path(&self) -> &PathBuf {
        &self.cache_file_path
    }

    pub fn init_cache(&self) -> Result<(), CacherError> {
        let cache = RedbDatabase::create(self.cache_file_path.clone())?;

        let write_txn = cache.begin_write()?;
        {
            write_txn.open_table(T::table_definition())?;
        }
        write_txn.commit()?;
        Ok(())
    }

    pub fn with_cache_dir(mut self, cache_dir: PathBuf) -> Result<Self, CacherError> {
        self.cache_file_path = cache_dir.clone();
        self.init_cache()?;
        Ok(self)
    }

    pub fn open_cache(&self) -> Result<RedbDatabase, DatabaseError> {
        RedbDatabase::open(&self.cache_file_path)
    }
    pub fn find_cache_entry(&self, query: &str, cache: &Database) -> Option<T> {
        let cache_reader = cache.begin_read().ok()?;
        let table = cache_reader.open_table(T::table_definition()).ok()?;

        if let Ok(Some(cache_entry)) = table.get(query) {
            return Some(cache_entry.value().into());
        }

        None
    }

    pub fn cache_object(&self, object_to_cache: T, cache: &Database) -> Result<(), CacherError> {
        let cache_writer = cache.begin_write()?;
        {
            let mut table = cache_writer.open_table(T::table_definition())?;
            for key in object_to_cache.keys() {
                table.insert(key, object_to_cache.clone())?;
            }
        }
        cache_writer.commit()?;
        Ok(())
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
    fn test(temp_dir: TempDir) {
        let cache_file_path = temp_dir.path().join("cache.hgnc");
        let hgvs_cacher = Cacher {
            cache_file_path: cache_file_path,
            _phantom: PhantomData::<ValidatedCHgvs>,
        };
    }
}
