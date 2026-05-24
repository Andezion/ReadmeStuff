use serde::Deserialize;
use std::time::Duration;

const DEFAULT_BASE_URL: &str = "https://www.codewars.com/api/v1/users/";

impl CodewarsApi {
    pub fn new(base_url: impl Into<String>) -> Self {
        CodewarsApi {
            base_url: base_url.into(),
            client: reqwest::blocking::Client::builder()
                .timeout(Duration::from_secs(10))
                .build()
                .expect("failed to build HTTP client"),
        }
    }
}