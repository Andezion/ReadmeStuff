use serde::Deserialize;
use std::collections::HashMap;
use std::time::Duration;

const DEFAULT_BASE_URL: &str = "https://www.codewars.com/api/v1";

type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

#[derive(Debug, Deserialize)]
pub struct RankEntry {
    pub rank: i32,
    pub name: String,
    pub color: String,
    pub score: u32,
}

#[derive(Debug, Deserialize)]
pub struct Ranks {
    pub overall: RankEntry,
    pub languages: HashMap<String, RankEntry>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CodeChallenges {
    pub total_authored: u32,
    pub total_completed: u32,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct User {
    pub username: String,
    pub name: Option<String>,
    pub honor: u32,
    pub clan: Option<String>,
    pub leaderboard_position: Option<u32>,
    pub skills: Option<Vec<String>>,
    pub ranks: Ranks,
    pub code_challenges: CodeChallenges,
}

pub struct CodewarsApi {
    base_url: String,
    client: reqwest::blocking::Client,
}

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

    pub fn user(&self, username: &str) -> Result<User> {
        let url = format!("{}/users/{}", self.base_url, username);
        let user = self.client.get(&url).send()?.json::<User>()?;
        Ok(user)
    }
}

impl Default for CodewarsApi {
    fn default() -> Self {
        Self::new(DEFAULT_BASE_URL)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const LIVE_TESTS_ENV: &str = "RUN_LIVE_CODEWARS_TESTS";

    #[test]
    fn test_live_user_logs_full_response() {
        if std::env::var(LIVE_TESTS_ENV).is_err() {
            eprintln!("skipping live Codewars test; set {LIVE_TESTS_ENV}=1 to run it");
            return;
        }

        let api = CodewarsApi::default();
        let user = api.user("Andezion").unwrap();

        println!("{user:#?}");

        assert_eq!(user.username, "Andezion");
    }
}
