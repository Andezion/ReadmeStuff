use reqwest::Client;
use serde::{Deserialize, Serialize, de::DeserializeOwned};
use serde_json::Value;
use thiserror::Error;

pub const GRAPHQL_URL: &str = "https://api.github.com/graphql";
pub const REST_URL: &str = "https://api.github.com";

#[derive(Debug, Error)]
pub enum GitHubError {
    #[error("HTTP request failed: {0}")]
    Http(#[from] reqwest::Error),

    #[error("GraphQL errors: {0}")]
    GraphQL(String),

    #[error("Missing GITHUB_TOKEN environment variable")]
    MissingToken,

    #[error("JSON deserialization failed: {0}")]
    Json(#[from] serde_json::Error),

    #[error("Resource not found: {0}")]
    NotFound(String),

    #[error("Rate limit exceeded, resets at: {reset_at}")]
    RateLimit { reset_at: String },
}

pub type Result<T> = std::result::Result<T, GitHubError>;

#[derive(Debug, Clone)]
pub struct GitHubClient {
    pub(crate) client: Client,
    pub(crate) token: String,
    pub(crate) graphql_url: String,
    pub(crate) rest_url: String,
}

#[derive(Debug, Deserialize)]
struct GraphQLResponse<T> {
    data: Option<T>,
    errors: Option<Vec<GqlError>>,
}

#[derive(Debug, Deserialize)]
struct GqlError {
    message: String,
}

#[derive(Debug, Serialize)]
struct GraphQLBody<'a> {
    query: &'a str,
    variables: Value,
}

impl GitHubClient {
    pub fn new(token: impl Into<String>) -> Result<Self> {
        let client = Client::builder()
            .user_agent("readme-stuff-api/1.0")
            .build()
            .map_err(GitHubError::Http)?;
        Ok(Self {
            client,
            token: token.into(),
            graphql_url: GRAPHQL_URL.to_string(),
            rest_url: REST_URL.to_string(),
        })
    }

    pub fn from_env() -> Result<Self> {
        let token = std::env::var("GITHUB_TOKEN").map_err(|_| GitHubError::MissingToken)?;
        Self::new(token)
    }

    pub async fn graphql<T: DeserializeOwned>(&self, query: &str, variables: Value) -> Result<T> {
        let body = GraphQLBody { query, variables };

        let resp = self
            .client
            .post(&self.graphql_url)
            .bearer_auth(&self.token)
            .header("X-GitHub-Api-Version", "2022-11-28")
            .json(&body)
            .send()
            .await?;

        if resp.status().as_u16() == 403 {
            let reset = resp
                .headers()
                .get("x-ratelimit-reset")
                .and_then(|v| v.to_str().ok())
                .unwrap_or("unknown")
                .to_string();
            return Err(GitHubError::RateLimit { reset_at: reset });
        }

        let gql: GraphQLResponse<T> = resp.error_for_status()?.json().await?;

        if let Some(errors) = gql.errors {
            let msg = errors
                .iter()
                .map(|e| e.message.as_str())
                .collect::<Vec<_>>()
                .join("; ");
            return Err(GitHubError::GraphQL(msg));
        }

        gql.data
            .ok_or_else(|| GitHubError::GraphQL("response contained no data".to_string()))
    }

    pub async fn rest_get<T: DeserializeOwned>(&self, path: &str) -> Result<T> {
        let url = format!(
            "{}/{}",
            self.rest_url.trim_end_matches('/'),
            path.trim_start_matches('/')
        );

        let resp = self
            .client
            .get(&url)
            .bearer_auth(&self.token)
            .header("Accept", "application/vnd.github+json")
            .header("X-GitHub-Api-Version", "2022-11-28")
            .send()
            .await?;

        if resp.status().as_u16() == 404 {
            return Err(GitHubError::NotFound(url));
        }

        if let Some(err) = rate_limit_error(&resp) {
            return Err(err);
        }

        Ok(resp.error_for_status()?.json().await?)
    }
}


fn rate_limit_error(resp: &reqwest::Response) -> Option<GitHubError> {
    if resp.status().as_u16() != 403 {
        return None;
    }

    let headers = resp.headers();
    let is_primary = headers
        .get("x-ratelimit-remaining")
        .and_then(|v| v.to_str().ok())
        == Some("0");
    let is_secondary = headers.contains_key("retry-after");

    if !is_primary && !is_secondary {
        return None;
    }

    let reset_at = headers
        .get("retry-after")
        .or_else(|| headers.get("x-ratelimit-reset"))
        .and_then(|v| v.to_str().ok())
        .unwrap_or("unknown")
        .to_string();
    Some(GitHubError::RateLimit { reset_at })
}
