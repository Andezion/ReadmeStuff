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
