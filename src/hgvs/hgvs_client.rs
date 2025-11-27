use std::thread::sleep;
use std::time::Duration;
use ratelimit::Ratelimiter;
use reqwest::blocking::Client;
use crate::hgnc::enums::GeneQuery;
use crate::hgnc::error::HGNCError;
use crate::hgnc::json_schema::{GeneDoc, GeneResponse};
use crate::hgvs::error::HGVSError;
use crate::hgvs::traits::HGVSData;
use crate::hgvs::validated_hgvs::ValidatedHgvs;

pub struct HGVSClient {
    rate_limiter: Ratelimiter,
    api_url: String,
    client: Client,
}

impl HGVSClient {
    pub fn new(api_url: String) -> Self {
        let rate_limiter = Ratelimiter::builder(10, Duration::from_secs(1))
            .max_tokens(10)
            .build()
            .expect("Building rate limiter failed");

        HGVSClient {
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

impl HGVSData for HGVSClient {
    fn request_variant_data(&self, unvalidated_hgvs: &str) -> Result<ValidatedHgvs, HGVSError> {

    }
}