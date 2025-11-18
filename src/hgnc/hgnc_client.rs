use crate::hgnc::enums::GeneQuery;
use crate::hgnc::error::HGNCError;
use crate::hgnc::json_schema::{GeneDoc, GeneResponse};
use crate::hgnc::traits::HGNCData;
use ratelimit::Ratelimiter;
use reqwest::blocking::Client;
use std::fmt::{Debug, Formatter};
use std::thread::sleep;
use std::time::Duration;

pub struct HGNCClient {
    rate_limiter: Ratelimiter,
    api_url: String,
    client: Client,
}

impl HGNCClient {
    pub fn new(api_url: String) -> Self {
        let rate_limiter = Ratelimiter::builder(10, Duration::from_secs(1))
            .max_tokens(10)
            .build()
            .expect("Building rate limiter failed");

        HGNCClient {
            rate_limiter,
            api_url,
            client: Client::new(),
        }
    }

    fn fetch_request(&self, url: String) -> Result<Vec<GeneDoc>, HGNCError> {
        if let Err(duration) = self.rate_limiter.try_wait() {
            sleep(duration);
        }
        let response = self
            .client
            .get(url.clone())
            .header("User-Agent", "PIVOT")
            .header("Accept", "application/json")
            .send()?;

        let gene_response = response.json::<GeneResponse>()?;

        Ok(gene_response.response.docs)
    }
}

impl HGNCData for HGNCClient {
    fn request_gene_data(&self, query: &GeneQuery) -> Result<GeneDoc, HGNCError> {
        let fetch_url = match &query {
            GeneQuery::Symbol(symbol) => format!("{}fetch/symbol/{}", self.api_url, symbol),
            GeneQuery::HgncId(id) => format!("{}fetch/hgnc_id/{}", self.api_url, id),
        };
        let docs = self.fetch_request(fetch_url)?;

        let result = match query {
            GeneQuery::Symbol(symbol) => docs
                .into_iter()
                .find(|doc| doc.symbol.as_deref() == Some(symbol)),
            GeneQuery::HgncId(id) => docs
                .into_iter()
                .find(|doc| doc.hgnc_id.as_deref() == Some(id)),
        };

        result.ok_or_else(|| {
            HGNCError::NoDocumentFound(match query {
                GeneQuery::Symbol(s) => s.to_string(),
                GeneQuery::HgncId(id) => id.to_string(),
            })
        })
    }

    fn request_hgnc_id(&self, symbol: &str) -> Result<String, HGNCError> {
        let doc = self.request_gene_data(&GeneQuery::Symbol(symbol))?;
        match doc.hgnc_id {
            None => Err(HGNCError::NoDocumentFound(symbol.to_string())),
            Some(hg_id) => Ok(hg_id),
        }
    }

    fn request_gene_symbol(&self, hgnc_id: &str) -> Result<String, HGNCError> {
        let doc = self.request_gene_data(&GeneQuery::HgncId(hgnc_id))?;

        match doc.symbol {
            None => Err(HGNCError::NoDocumentFound(hgnc_id.to_string())),
            Some(symbol) => Ok(symbol),
        }
    }
}

impl Default for HGNCClient {
    fn default() -> Self {
        let rate_limiter = Ratelimiter::builder(10, Duration::from_secs(1))
            .max_tokens(10)
            .build()
            .expect("Building rate limiter failed");

        HGNCClient {
            rate_limiter,
            api_url: "https://rest.genenames.org/".to_string(),
            client: Client::new(),
        }
    }
}

impl Debug for HGNCClient {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("HGNCClient")
            .field("api_url", &self.api_url)
            .field("rate_limiter", &"<Ratelimiter>")
            .finish()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rstest::rstest;

    #[rstest]
    #[case(GeneQuery::Symbol("ZNF3"), Some("HGNC:13089"), Some("ZNF3"))]
    #[case(GeneQuery::HgncId("HGNC:13089"), Some("HGNC:13089"), Some("ZNF3"))]
    fn test_request_gene_data(
        #[case] query: GeneQuery,
        #[case] expected_hgnc_id: Option<&str>,
        #[case] expected_symbol: Option<&str>,
    ) {
        let client = HGNCClient::default();

        let gene_doc = client.request_gene_data(&query).unwrap();

        assert_eq!(gene_doc.hgnc_id.as_deref(), expected_hgnc_id);
        assert_eq!(gene_doc.symbol.as_deref(), expected_symbol);
    }

    #[rstest]
    fn test_request_hgnc_id() {
        let symbol = "CLOCK";
        let client = HGNCClient::default();

        let hgnc_id = client.request_hgnc_id(symbol).unwrap();

        assert_eq!(hgnc_id.as_str(), "HGNC:2082");
    }

    #[rstest]
    fn test_request_gene_symbol() {
        let hgnc_id = "HGNC:2082";
        let client = HGNCClient::default();

        let gene_symbol = client.request_gene_symbol(hgnc_id).unwrap();

        assert_eq!(gene_symbol.as_str(), "CLOCK");
    }
}
