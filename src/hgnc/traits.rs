use crate::hgnc::enums::GeneQuery;
use crate::hgnc::error::HGNCError;
use crate::hgnc::json_schema::GeneDoc;

pub trait HGNCData {
    fn request_gene_data(&self, query: GeneQuery) -> Result<GeneDoc, HGNCError>;
    fn request_hgnc_id(&self, symbol: &str) -> Result<String, HGNCError> {
        let doc = self.request_gene_data(GeneQuery::Symbol(symbol))?;
        Ok(doc.hgnc_id_owned())
    }
    fn request_gene_symbol(&self, hgnc_id: &str) -> Result<String, HGNCError> {
        let doc = self.request_gene_data(GeneQuery::HgncId(hgnc_id))?;
        Ok(doc.symbol_owned())
    }
    fn request_gene_identifier_pair(
        &self,
        query: GeneQuery,
    ) -> Result<(String, String), HGNCError> {
        let doc = self.request_gene_data(query)?;
        Ok(doc.symbol_id_pair())
    }
}
