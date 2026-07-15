use serde::Deserialize;
use std::time::Duration;

const DEFAULT_BASE_URL: &str = "https://alfa-leetcode-api.onrender.com";

type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Badge {
    pub id: String,
    pub display_name: String,
    pub icon: String,
    pub creation_date: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BadgesResponse {
    pub badges_count: u32,
    pub badges: Vec<Badge>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Language {
    #[serde(rename = "languageName")]
    pub name: String,
    #[serde(rename = "problemsSolved")]
    pub solved_amount: u32,
}

#[derive(Debug, Deserialize)]
struct LanguagesResponse {
    #[serde(rename = "languageProblemCount")]
    language_problem_count: Vec<Language>,
}

#[derive(Debug, Deserialize)]
pub struct Solved {
    #[serde(rename = "solvedProblem")]
    pub solved_problem: u32,
    #[serde(rename = "easySolved")]
    pub easy_solved: u32,
    #[serde(rename = "mediumSolved")]
    pub medium_solved: u32,
    #[serde(rename = "hardSolved")]
    pub hard_solved: u32,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Skill {
    #[serde(rename = "tagName")]
    pub name: String,
    #[serde(rename = "problemsSolved")]
    pub amount: u32,
}

#[derive(Debug, Deserialize)]
pub struct SkillsResponse {
    pub advanced: Vec<Skill>,
    pub intermediate: Vec<Skill>,
    pub fundamental: Vec<Skill>,
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

    pub fn badges(&self, username: &str) -> Result<BadgesResponse> {
        let url = format!("{}/{}/badges", self.base_url, username);
        let resp = self.client.get(&url).send()?.json::<BadgesResponse>()?;
        Ok(resp)
    }

    pub fn languages(&self, username: &str) -> Result<Vec<Language>> {
        let url = format!("{}/{}/language", self.base_url, username);
        let resp = self.client.get(&url).send()?.json::<LanguagesResponse>()?;
        Ok(resp.language_problem_count)
    }

    pub fn skills(&self, username: &str) -> Result<SkillsResponse> {
        let url = format!("{}/{}/skill", self.base_url, username);
        let resp = self.client.get(&url).send()?.json::<SkillsResponse>()?;
        Ok(resp)
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

    #[test]
    fn test_leetcode() -> Result<()> {
        let client = reqwest::blocking::Client::builder()
            .timeout(Duration::from_secs(10))
            .build()?;

        let solved: serde_json::Value = client
            .get(format!("{DEFAULT_BASE_URL}/Andezion/solved"))
            .send()?
            .json()?;
        println!("Solved:\n{}", serde_json::to_string_pretty(&solved)?);
        assert_eq!(solved["solvedProblem"], 611);

        let language: serde_json::Value = client
            .get(format!("{DEFAULT_BASE_URL}/Andezion/language"))
            .send()?
            .json()?;
        println!("Language:\n{}", serde_json::to_string_pretty(&language)?);

        let skill: serde_json::Value = client
            .get(format!("{DEFAULT_BASE_URL}/Andezion/skill"))
            .send()?
            .json()?;
        println!("Skill:\n{}", serde_json::to_string_pretty(&skill)?);

        let badges: serde_json::Value = client
            .get(format!("{DEFAULT_BASE_URL}/Andezion/badges"))
            .send()?
            .json()?;
        println!("Badges:\n{}", serde_json::to_string_pretty(&badges)?);

        Ok(())
    }
}

