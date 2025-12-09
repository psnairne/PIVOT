//! Here we test that the public-facing gives us what we want and nothing more

use pivot::hgnc::{
     GeneDoc, GeneQuery, HGNCClient, HGNCData, HGNCError, MockHGNCClient,
};
use pivot::hgvs::{
    AlleleCount, ChromosomalSex, HGVSClient, HGVSData, HGVSError, HgvsVariant,
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

