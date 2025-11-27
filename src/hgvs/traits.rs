use crate::hgvs::validated_hgvs::ValidatedHgvs;

pub trait HGVSData {
    fn request_variant_data(&self, unvalidated_hgvs: &str) -> Result<ValidatedHgvs, HGVSError>;
    fn request_hgnc_id(&self, symbol: &str) -> Result<String, HGNCError>;
    fn request_gene_symbol(&self, hgnc_id: &str) -> Result<String, HGNCError>;
    fn request_gene_identifier_pair(&self, query: GeneQuery)
                                    -> Result<(String, String), HGNCError>;
}

