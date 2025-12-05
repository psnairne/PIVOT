//! # PIVOT
//!
//! A library for getting gene data from HGNC and variant data from VariantValidator.
//!
//! ## HGNC
//!
//! - Given a gene symbol or an HGNC ID, get a GeneDoc containing a multitude of information about that gene.
//!
//! - If you use CachedHGNCClient, the GeneDocs will be cached and can thereafter be accessed without an API call.
//!
//! ## HGVS
//!
//! - Validate a hgvs.c (e.g. NM_000138.5:c.8242G>T) or hgvs.n (NR_002196.1:n.601G>T) string, and get a HgvsVariant object with information on the relevant (coding or non-coding) gene, and the amino acid change if applicable.
//!
//! - If you use CachedHGVSClient, the HgvsVariant objects will be cached and can thereafter be accessed without an API call.
//!
//! - There is also functionality for creating a Phenopacket VariantInterpretation from a HgvsVariant object and data on allele count and chromosomal sex.

mod cache_structs_and_traits;
pub mod hgnc;
pub mod hgvs;
