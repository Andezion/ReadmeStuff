use serde::Deserialize;
use std::collections::HashMap;
use std::time::Duration;

const DEFAULT_BASE_URL: &str = "https://www.codewars.com/api/v1";

type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

#[derive(Deserialize)]
pub struct RankEntry {
    pub rank: i32,
    pub name: String,
    pub color: String,
    pub score: u32,
}

#[derive(Deserialize)]
pub struct Ranks {
    pub overall: RankEntry,
    pub languages: HashMap<String, RankEntry>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CodeChallenges {
    pub total_authored: u32,
    pub total_completed: u32,
}

#[derive(Deserialize)]
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
    use httpmock::prelude::*;
    use serde_json::json;

    fn api(server: &MockServer) -> CodewarsApi {
        CodewarsApi::new(server.base_url())
    }

    #[test]
    fn test_user() {
        let server = MockServer::start();
        let mock = server.mock(|when, then| {
            when.method(GET).path("/users/testuser");
            then.status(200).json_body(json!({
                "username": "testuser",
                "name": "Test User",
                "honor": 544,
                "clan": null,
                "leaderboardPosition": 134,
                "skills": ["rust", "python"],
                "ranks": {
                    "overall": { "rank": -3, "name": "3 kyu", "color": "blue", "score": 2116 },
                    "languages": {
                        "rust": { "rank": -3, "name": "3 kyu", "color": "blue", "score": 1819 }
                    }
                },
                "codeChallenges": { "totalAuthored": 3, "totalCompleted": 230 }
            }));
        });

        let user = api(&server).user("testuser").unwrap();

        assert_eq!(user.username, "testuser");
        assert_eq!(user.honor, 544);
        assert_eq!(user.ranks.overall.rank, -3);
        assert_eq!(user.ranks.overall.name, "3 kyu");
        assert_eq!(user.ranks.languages["rust"].score, 1819);
        assert_eq!(user.code_challenges.total_completed, 230);
        assert_eq!(user.leaderboard_position, Some(134));
        mock.assert();
    }

    #[test]
    fn test_network_error_returns_err() {
        let api = CodewarsApi::new("http://127.0.0.1:1");
        assert!(api.user("testuser").is_err());
    }
}
