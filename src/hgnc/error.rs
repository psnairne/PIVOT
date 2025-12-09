#[cfg(feature = "caching")]
use crate::caching::error::CacherError;
#[cfg(feature = "caching")]
use redb::{CommitError, DatabaseError, StorageError, TableError, TransactionError};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum HGNCError {
    #[error(
        "Found '{n_found}' documents for '{identifier}' on HGNC, when '{n_expected}' were expected."
    )]
    UnexpectedNumberOfDocuments {
        identifier: String,
        n_found: usize,
        n_expected: usize,
    },
    #[error("No {desired_element} found in GeneDoc.")]
    MissingElementInDocument { desired_element: String },
    #[error("Cant establish caching dir {0}")]
    CannotEstablishCacheDir(String),
    #[cfg(feature = "caching")]
    #[error(transparent)]
    CacherError(#[from] CacherError),
    #[cfg(feature = "caching")]
    #[error(transparent)]
    CacheCommit(#[from] CommitError),
    #[cfg(feature = "caching")]
    #[error(transparent)]
    CacheStorage(#[from] StorageError),
    #[cfg(feature = "caching")]
    #[error(transparent)]
    CacheTransaction(#[from] TransactionError),
    #[cfg(feature = "caching")]
    #[error(transparent)]
    CacheDatabase(#[from] DatabaseError),
    #[cfg(feature = "caching")]
    #[error(transparent)]
    CacheTable(#[from] TableError),
    #[error(transparent)]
    Request(#[from] reqwest::Error),
}
