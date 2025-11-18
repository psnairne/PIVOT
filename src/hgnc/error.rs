use redb::{CommitError, DatabaseError, StorageError, TableError, TransactionError};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum HGNCError {
    #[error("Not document found for {0}")]
    NoDocumentFound(String),
    #[error("Cant establish caching dir {0}")]
    CannotEstablishCacheDir(String),
    #[error(transparent)]
    CacheCommit(#[from] CommitError),
    #[error(transparent)]
    CacheStorage(#[from] StorageError),
    #[error(transparent)]
    CacheTransaction(#[from] TransactionError),
    #[error(transparent)]
    CacheDatabase(#[from] DatabaseError),
    #[error(transparent)]
    CacheTable(#[from] TableError),
    #[error(transparent)]
    Request(#[from] reqwest::Error),
}
