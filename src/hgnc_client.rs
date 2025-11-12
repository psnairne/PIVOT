use crate::error::PivotError;

pub struct HgncClient;

impl HgncClient {
    pub fn get_gene_data_from_hgnc(&self, _gene: &str) -> Result<(&str, &str), PivotError> {
        todo!()
    }
}
