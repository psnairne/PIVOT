//! # PIVOT
//!
//! A library for getting validated variant data using the VariantValidator API. One can request the full data from VariantValidator, or an abbreviated form.
//!
//! ## Objects
//!
//! # [`SingleVariantResponse`]
//!
//! This struct contains all the information provided by VariantValidator on a single variant.
//!
//! # [`HgvsVariant`]
//!
//! This is a condensed form of [`SingleVariantResponse`].
//!
//! # [`HGVSData`]
//!
//! A trait consisting of the following methods:
//!
//! - `get_full_validated_hgvs_data(&self, unvalidated_hgvs: &str) -> Result<SingleVariantResponse, HGVSError>` — validates that the HGVS is accurate and, if so, returns the [`SingleVariantResponse`] object.
//! - `get_abbreviated_validated_hgvs_data(&self, unvalidated_hgvs: &str) -> Result<HgvsVariant, HGVSError>` — validates that the HGVS is accurate and, if so, returns the condensed [`HgvsVariant`] object.
//!
//! The get_abbreviated_validated_hgvs_data method is formed automatically.
//!
//! # [`HGVSClient`]
//!
//! The basic implementation of the [`HGVSData`] trait. Make a request to the VariantValidator API and receive either a [`SingleVariantResponse`] or [`HgvsVariant`] object if the &str was a valid hgvs.c or hgvs.n variant string.
//!
//! # [`HGVSError`]
//!
//! An enum for errors returned by the API.
//!
//! ## Use
//!
//! ### Get full data from VariantValidator
//!
//! ```rust
//! use pivot::{GenomeAssembly, HGVSClient, HGVSData};
//!
//! let client = HGVSClient::default();
//!
//! let hgvs_data = client.get_full_validated_hgvs_data("NM_001173464.1:c.2860C>T").unwrap();
//!
//! assert_eq!(hgvs_data.flag, "gene_variant");
//! ```
//!
//! ### Get condensed data from VariantValidator
//!
//! The [`GenomeAssembly`] argument determines the g_hgvs attribute of [`HgvsVariant`].
//!
//! ```rust
//! use pivot::{GenomeAssembly, HGVSClient, HGVSData};
//!
//! let client = HGVSClient::default();
//!
//! let hgvs_variant = client.get_abbreviated_validated_hgvs_data("NM_001173464.1:c.2860C>T", GenomeAssembly::Hg38).unwrap();
//!
//! assert_eq!(hgvs_variant.gene_symbol(), "KIF21A");
//! ```
//!
//! ### Convert [`SingleVariantResponse`] into the more condensed [`HgvsVariant`].
//!
//! ```rust
//! use pivot::{GenomeAssembly, HGVSClient, HGVSData};
//!
//! let client = HGVSClient::default();
//!
//! let hgvs_data = client.get_full_validated_hgvs_data("NM_001173464.1:c.2860C>T").unwrap();
//! let hgvs_variant = hgvs_data.abbreviate_response(GenomeAssembly::Hg38).unwrap();
//!
//! assert_eq!(hgvs_variant.gene_symbol(), "KIF21A");
//! ```
//!
pub use enums::GenomeAssembly;
pub use error::HGVSError;
pub use hgvs_client::HGVSClient;
pub use hgvs_variant::HgvsVariant;
pub use single_variant_response::SingleVariantResponse;
pub use traits::HGVSData;

mod enums;
mod error;
mod hgvs_client;
mod hgvs_variant;
mod json_schema;
mod single_variant_response;
mod traits;
mod utils;
