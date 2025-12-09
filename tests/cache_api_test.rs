#![cfg(feature = "caching")]
use rstest::rstest;
use pivot::hgnc::{GeneQuery, HGNCClient};
use pivot::hgvs::HGVSClient;
use pivot::hgvs::CachedHGVSClient;
use pivot::hgnc::CachedHGNCClient;
use pivot::hgvs::HGVSData;
use pivot::hgnc::HGNCData;
#[rstest]
fn test_cached_hgnc_client() {
    let temp_dir = tempfile::tempdir().expect("Failed to create temporary directory");
    let cache_file_path = temp_dir.path().join("cache.hgnc");

    let client = CachedHGNCClient::new(cache_file_path, HGNCClient::default()).unwrap();
    let gene_doc = client
        .request_gene_data(GeneQuery::HgncId("HGNC:13089"))
        .unwrap();
    let expected_location = Some("7q22.1".to_string());
    assert_eq!(gene_doc.location, expected_location);
}


#[rstest]
fn test_cached_hgvs_client() {
    let temp_dir = tempfile::tempdir().expect("Failed to create temporary directory");
    let cache_file_path = temp_dir.path().join("cache.hgvs");

    let client = CachedHGVSClient::new(cache_file_path, HGVSClient::default()).unwrap();

    let hgvs_variant = client
        .request_and_validate_hgvs("NR_002196.1:n.601G>T")
        .unwrap();
    let expected_gene = "H19".to_string();
    assert_eq!(hgvs_variant.gene_symbol(), expected_gene);
}
