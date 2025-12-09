//! Here we test that the public-facing gives us what we want and nothing more

use pivot::hgnc::{
    CachedHGNCClient, GeneDoc, GeneQuery, HGNCClient, HGNCData, HGNCError, MockHGNCClient,
};
use pivot::hgvs::{
    AlleleCount, CachedHGVSClient, ChromosomalSex, HGVSClient, HGVSData, HGVSError, HgvsVariant,
};
use rstest::rstest;
use std::collections::HashMap;

#[rstest]
fn test_hgnc_client() {
    let client = HGNCClient::default();

    fn manipulate_hgnc_err(hgnc_err: HGNCError) -> String {
        format!("{:?}", hgnc_err)
    }

    let gene_doc_result = client.request_gene_data(GeneQuery::Symbol("CLOCK"));
    let gene_doc = gene_doc_result.map_err(manipulate_hgnc_err).unwrap();

    let different_gene_doc = GeneDoc::default();
    assert_ne!(gene_doc, different_gene_doc);
}

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
fn test_mock_hgnc_client() {
    let mut docs = HashMap::new();

    docs.insert(
        "BRCA1".to_string(),
        GeneDoc::default()
            .with_hgnc_id("HGNC:1100")
            .with_symbol("BRCA1"),
    );

    let mock = MockHGNCClient::new(docs);

    let gene_doc = mock.request_gene_data(GeneQuery::Symbol("BRCA1")).unwrap();
    assert_eq!(gene_doc.symbol, Some("BRCA1".to_string()));
}

#[rstest]
fn test_hgvs_client() {
    let client = HGVSClient::default();

    fn manipulate_hgvs_err(hgvs_err: HGVSError) -> String {
        format!("{:?}", hgvs_err)
    }

    let hgvs_variant_result = client.request_and_validate_hgvs("NM_001173464.1:c.2860C>T");
    let hgvs_variant = hgvs_variant_result.map_err(manipulate_hgvs_err).unwrap();

    let different_hgvs_variant = HgvsVariant::default();
    assert_ne!(hgvs_variant, different_hgvs_variant);

    hgvs_variant.validate_against_gene("KIF21A").unwrap();

    let vi = hgvs_variant
        .create_variant_interpretation(AlleleCount::Single, ChromosomalSex::XX)
        .unwrap();
    assert_eq!(
        vi.variation_descriptor
            .unwrap()
            .allelic_state
            .unwrap()
            .label,
        "heterozygous".to_string()
    );
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
