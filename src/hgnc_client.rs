use crate::error::PivotError;

pub struct HgncClient;

impl HgncClient {
    /// This takes a gene symbol or HGNC ID and returns the (symbol, ID) pair
    pub fn get_gene_data_from_hgnc(&self, _gene: &str) -> Result<(&str, &str), PivotError> {
        todo!()
    }
}
