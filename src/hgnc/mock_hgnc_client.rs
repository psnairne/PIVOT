use crate::hgnc::enums::GeneQuery;
use crate::hgnc::error::HGNCError;
use crate::hgnc::json_schema::GeneDoc;
use crate::hgnc::traits::HGNCData;
use std::collections::HashMap;

/// A Mock client for the HGNC interface.
///
/// This struct is intended for use in unit testing. Instead of making live HTTP
/// requests to the HGNC API, it serves data from an internal `HashMap`.
/// This allows for deterministic testing of components that rely on `HGNCData`.
pub struct MockHGNClient {
    docs: HashMap<String, GeneDoc>,
}

impl MockHGNClient {
    pub fn new(docs: HashMap<String, GeneDoc>) -> MockHGNClient {
        MockHGNClient { docs }
    }
}

impl HGNCData for MockHGNClient {
    fn request_gene_data(&self, query: GeneQuery) -> Result<GeneDoc, HGNCError> {
        let identifier = query.inner();
        self.docs
            .get(identifier)
            .cloned()
            .ok_or(HGNCError::UnexpectedNumberOfDocuments {
                identifier: identifier.to_string(),
                n_found: 0,
                n_expected: 1,
            })
    }

    fn request_hgnc_id(&self, symbol: &str) -> Result<String, HGNCError> {
        let doc = self.request_gene_data(GeneQuery::Symbol(symbol))?;
        doc.hgnc_id.ok_or(HGNCError::UnexpectedNumberOfDocuments {
            identifier: symbol.to_string(),
            n_found: 0,
            n_expected: 1,
        })
    }

    fn request_gene_symbol(&self, hgnc_id: &str) -> Result<String, HGNCError> {
        let doc = self.request_gene_data(GeneQuery::HgncId(hgnc_id))?;
        doc.symbol.ok_or(HGNCError::UnexpectedNumberOfDocuments {
            identifier: hgnc_id.to_string(),
            n_found: 0,
            n_expected: 1,
        })
    }

    fn request_gene_identifier_pair(
        &self,
        query: GeneQuery,
    ) -> Result<(String, String), HGNCError> {
        let identifier_string = query.inner().to_string();

        let doc = self.request_gene_data(query)?;

        let hgnc_id = doc.hgnc_id.ok_or(HGNCError::UnexpectedNumberOfDocuments {
            identifier: identifier_string.clone(),
            n_found: 0,
            n_expected: 1,
        })?;

        let symbol = doc.symbol.ok_or(HGNCError::UnexpectedNumberOfDocuments {
            identifier: identifier_string,
            n_found: 0,
            n_expected: 1,
        })?;

        Ok((hgnc_id, symbol))
    }
}

impl Default for MockHGNClient {
    fn default() -> Self {
        let mut docs = HashMap::new();
        docs.insert(
            "BRCA1".to_string(),
            GeneDoc::default().hgnc_id("HGNC:1100").symbol("BRCA1"),
        );

        docs.insert(
            "HGNC:1100".to_string(),
            GeneDoc::default().hgnc_id("HGNC:1100").symbol("BRCA1"),
        );

        docs.insert(
            "HGNC:2082".to_string(),
            GeneDoc::default().hgnc_id("HGNC:2082").symbol("CLOCK"),
        );

        docs.insert(
            "CLOCK".to_string(),
            GeneDoc::default().hgnc_id("HGNC:2082").symbol("CLOCK"),
        );

        docs.insert(
            "HGNC:10848".to_string(),
            GeneDoc::default().hgnc_id("HGNC:10848").symbol("SHH"),
        );

        docs.insert(
            "SHH".to_string(),
            GeneDoc::default().hgnc_id("HGNC:10848").symbol("SHH"),
        );

        docs.insert(
            "HGNC:11251".to_string(),
            GeneDoc::default().hgnc_id("HGNC:11251").symbol("SPOCK1"),
        );

        docs.insert(
            "SPOCK1".to_string(),
            GeneDoc::default().hgnc_id("HGNC:11251").symbol("SPOCK1"),
        );

        MockHGNClient::new(docs)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    fn setup_mock() -> MockHGNClient {
        let mut docs = HashMap::new();

        docs.insert(
            "BRCA1".to_string(),
            GeneDoc::default().hgnc_id("HGNC:1100").symbol("BRCA1"),
        );

        docs.insert(
            "HGNC:1100".to_string(),
            GeneDoc::default().hgnc_id("HGNC:1100").symbol("BRCA1"),
        );

        docs.insert(
            "NO_SYMBOL_DATA".to_string(),
            GeneDoc::default().hgnc_id("HGNC:9999"),
        );

        MockHGNClient::new(docs)
    }

    #[test]
    fn test_request_gene_data_success() {
        let mock = setup_mock();
        let query = GeneQuery::Symbol("BRCA1");

        let result = mock.request_gene_data(query);
        assert!(result.is_ok());

        let doc = result.unwrap();
        assert_eq!(doc.symbol, Some("BRCA1".to_string()));
    }

    #[test]
    fn test_request_gene_data_not_found() {
        let mock = setup_mock();
        let query = GeneQuery::Symbol("UNKNOWN_GENE");

        let result = mock.request_gene_data(query);
        assert!(result.is_err());

        if let Err(HGNCError::UnexpectedNumberOfDocuments {
            identifier,
            n_found,
            ..
        }) = result
        {
            assert_eq!(identifier, "UNKNOWN_GENE");
            assert_eq!(n_found, 0);
        } else {
            panic!("Returned wrong error type");
        }
    }

    #[test]
    fn test_request_hgnc_id_success() {
        let mock = setup_mock();
        let result = mock.request_hgnc_id("BRCA1");

        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "HGNC:1100");
    }

    #[test]
    fn test_request_gene_symbol_success() {
        let mock = setup_mock();
        let result = mock.request_gene_symbol("HGNC:1100").unwrap();

        assert_eq!(result, "BRCA1");
    }

    #[test]
    fn test_request_pair_success() {
        let mock = setup_mock();
        let query = GeneQuery::Symbol("BRCA1");

        let result = mock.request_gene_identifier_pair(query);
        assert!(result.is_ok());

        let (id, symbol) = result.unwrap();
        assert_eq!(id, "HGNC:1100");
        assert_eq!(symbol, "BRCA1");
    }

    #[test]
    fn test_missing_fields_returns_error() {
        let mock = setup_mock();

        let result = mock.request_gene_symbol("NO_SYMBOL_DATA");

        assert!(result.is_err());
        if let Err(HGNCError::UnexpectedNumberOfDocuments { identifier, .. }) = result {
            assert_eq!(identifier, "NO_SYMBOL_DATA");
        } else {
            panic!("Should fail due to missing field in doc");
        }
    }
}
