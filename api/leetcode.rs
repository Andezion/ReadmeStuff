use serde::Deserialize;
use std::time::Duration;

const DEFAULT_BASE_URL: &str = "https://alfa-leetcode-api.onrender.com";

type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

#[derive(Deserialize)]
pub struct Badge {
    pub id: u32,
    pub name: String,
    pub icon_url: String,
    pub date: String,
}

#[derive(Deserialize)]
pub struct BadgeStorage {
    pub counter: u32,
    pub badges: Vec<Badge>,
}

#[derive(Deserialize)]
pub struct Language {
    pub name: String,
    pub solved_amount: u32,
}

#[derive(Deserialize)]
pub struct Solved {
    pub full_amount: u32,
    pub easy_amount: u32,
    pub medium_amount: u32,
    pub hard_amount: u32,
}

#[derive(Deserialize)]
pub struct Skill {
    pub name: String,
    pub amount: u32,
}

pub struct LeetcodeApi {
    base_url: String,
    client: reqwest::blocking::Client,
}

impl LeetcodeApi {
    pub fn new(base_url: impl Into<String>) -> Self {
        LeetcodeApi {
            base_url: base_url.into(),
            client: reqwest::blocking::Client::builder()
                .timeout(Duration::from_secs(10))
                .build()
                .expect("failed to build HTTP client"),
        }
    }

    pub fn amount_of_solved_problems(&self, username: &str) -> Result<Solved> {
        let url = format!("{}/{}/solved", self.base_url, username);
        let solved = self.client.get(&url).send()?.json::<Solved>()?;
        Ok(solved)
    }

    pub fn badges(&self, username: &str) -> Result<Vec<BadgeStorage>> {
        let url = format!("{}/{}/badge", self.base_url, username);
        let badges = self.client.get(&url).send()?.json::<Vec<BadgeStorage>>()?;
        Ok(badges)
    }

    pub fn languages(&self, username: &str) -> Result<Vec<Language>> {
        let url = format!("{}/{}/language", self.base_url, username);
        let languages = self.client.get(&url).send()?.json::<Vec<Language>>()?;
        Ok(languages)
    }

    pub fn skills(&self, username: &str) -> Result<Vec<Skill>> {
        let url = format!("{}/{}/skill", self.base_url, username);
        let skills = self.client.get(&url).send()?.json::<Vec<Skill>>()?;
        Ok(skills)
    }
}

impl Default for LeetcodeApi {
    fn default() -> Self {
        Self::new(DEFAULT_BASE_URL)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use httpmock::prelude::*;
    use serde_json::json;

    fn api(server: &MockServer) -> LeetcodeApi {
        LeetcodeApi::new(server.base_url())
    }

    #[test]
    fn test_solved_problems() {
        let server = MockServer::start();
        let mock = server.mock(|when, then| {
            when.method(GET).path("/testuser/solved");
            then.status(200).json_body(json!({
                "full_amount": 150,
                "easy_amount": 80,
                "medium_amount": 50,
                "hard_amount": 20
            }));
        });

        let result = api(&server).amount_of_solved_problems("testuser").unwrap();

        assert_eq!(result.full_amount, 150);
        assert_eq!(result.easy_amount, 80);
        assert_eq!(result.medium_amount, 50);
        assert_eq!(result.hard_amount, 20);
        mock.assert();
    }

    #[test]
    fn test_languages() {
        let server = MockServer::start();
        let mock = server.mock(|when, then| {
            when.method(GET).path("/testuser/language");
            then.status(200).json_body(json!([
                { "name": "Rust", "solved_amount": 42 },
                { "name": "Python", "solved_amount": 10 }
            ]));
        });

        let result = api(&server).languages("testuser").unwrap();

        assert_eq!(result.len(), 2);
        assert_eq!(result[0].name, "Rust");
        assert_eq!(result[0].solved_amount, 42);
        mock.assert();
    }

    #[test]
    fn test_skills() {
        let server = MockServer::start();
        let mock = server.mock(|when, then| {
            when.method(GET).path("/testuser/skill");
            then.status(200).json_body(json!([
                { "name": "Array", "amount": 30 },
                { "name": "Dynamic Programming", "amount": 15 }
            ]));
        });

        let result = api(&server).skills("testuser").unwrap();

        assert_eq!(result.len(), 2);
        assert_eq!(result[1].name, "Dynamic Programming");
        assert_eq!(result[1].amount, 15);
        mock.assert();
    }

    #[test]
    fn test_badges() {
        let server = MockServer::start();
        let mock = server.mock(|when, then| {
            when.method(GET).path("/testuser/badge");
            then.status(200).json_body(json!([{
                "counter": 2,
                "badges": [
                    { "id": 1, "name": "Annual Badge", "icon_url": "https://example.com/badge.png", "date": "2024-01-01" }
                ]
            }]));
        });

        let result = api(&server).badges("testuser").unwrap();

        assert_eq!(result.len(), 1);
        assert_eq!(result[0].counter, 2);
        assert_eq!(result[0].badges[0].name, "Annual Badge");
        mock.assert();
    }

    #[test]
    fn test_network_error_returns_err() {
        let api = LeetcodeApi::new("http://127.0.0.1:1");
        assert!(api.amount_of_solved_problems("testuser").is_err());
    }
}
