use serde::Deserialize;
use std::time::Duration;

const DEFAULT_BASE_URL: &str = "https://www.codewars.com/api/v1/users/";

pub struct CodeChallenge {
    pub total_authored: u32, 
    pub total_completed: u32,
}

pub struct Language {
    pub name: String,
    pub rank: u32,
    pub rank_name: String,
    pub color: String,
    pub score: u32,
}

pub struct Rank {
    pub name: String,
    pub rank: u32,
    pub rank_name: String,
    pub color: String,
    pub score: u32,
    pub languages: Vec<Language>
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

    pub fn amount_of_completed_challenges(&self, username: &str) -> Result<CodeChallenge> {
        let url = format!("{}/{}/codeChallenges", self.base_url, username);
        let challenge = self.client.get(&url).send()?.json::<CodeChallenge>()?;
        Ok(challenge)
    }

    pub fn rank(&self, username: &str) -> Result<Rank> {
        let url = format!("{}/{}/ranks", self.base_url, username);
        let rank = self.client.get(&url).send()?.json::<Rank>()?;
        Ok(rank)
    }

    pub fn skills(&self, username: &str) -> Result<Vec<String>> {
        let url = format!("{}/{}/skills", self.base_url, username);
        let skills = self.client.get(&url).send()?.json::<Vec<String>>()?;
        Ok(skills)
    }

    pub fn honor(&self, username: &str) -> Result<u32> {
        let url = format!("{}/{}/honor", self.base_url, username);
        let honor = self.client.get(&url).send()?.json::<u32>()?;
        Ok(honor)
    }

    pub fn leaderboard_position(&self, username: &str) -> Result<u32> {
        let url = format!("{}/{}/leaderboardPosition", self.base_url, username);
        let position = self.client.get(&url).send()?.json::<u32>()?;
        Ok(position)
    }
}