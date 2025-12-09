//! Here we test that the public-facing gives us what we want and nothing more
use pivot::{GenomeAssembly, HGVSClient, HGVSData, HGVSError, HgvsVariant, SingleVariantResponse};
use rstest::rstest;

#[rstest]
fn test_hgvs_client() {
    let client = HGVSClient::default();

    fn manipulate_hgvs_err(hgvs_err: HGVSError) -> String {
        format!("{:?}", hgvs_err)
    }

    let hgvs_data_result = client.get_full_validated_hgvs_data("NM_001173464.1:c.2860C>T");
    let hgvs_data = hgvs_data_result.map_err(manipulate_hgvs_err).unwrap();

    let different_hgvs_data = SingleVariantResponse::default();
    assert_ne!(
        hgvs_data.transcript_hgvs,
        different_hgvs_data.transcript_hgvs
    );

    let hgvs_variant = hgvs_data.abbreviate_response(GenomeAssembly::Hg38).unwrap();

    let different_hgvs_variant = HgvsVariant::default();
    assert_ne!(hgvs_variant, different_hgvs_variant);
}

#[rstest]
fn test_hgvs_client2() {
    use pivot::{GenomeAssembly, HGVSClient, HGVSData};

    let client = HGVSClient::default();

    let hgvs_data = client
        .get_abbreviated_validated_hgvs_data("NM_001173464.1:c.2860C>T", GenomeAssembly::Hg38)
        .unwrap();

    assert_eq!(hgvs_data.gene_symbol(), "blah");
}
