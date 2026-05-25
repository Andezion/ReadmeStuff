use serde::de::DeserializeOwned;
use serde::Deserialize;

const DEFAULT_BASE_URL: &str = "https://codeforces.com/api";

type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

#[derive(Debug, Deserialize)]
pub enum Verdict {
    FAILED,
    OK,
    PARTIAL,
    COMPILATION_ERROR,
    RUNTIME_ERROR,
    WRONG_ANSWER,
    TIME_LIMIT_EXCEEDED,
    MEMORY_LIMIT_EXCEEDED,
    IDLENESS_LIMIT_EXCEEDED,
    SECURITY_VIOLATED,
    CRASHED,
    INPUT_PREPARATION_CRASHED,
    CHALLENGED,
    SKIPPED,
    TESTING,
    REJECTED,
    SUBMITTED,
}

#[derive(Debug, Deserialize)]
pub enum Testset {
    SAMPLES,
    PRETESTS,
    TESTS,
    CHALLENGES,
    TESTS1,
    TESTS2,
    TESTS3,
    TESTS4,
    TESTS5,
    TESTS6,
    TESTS7,
    TESTS8,
    TESTS9,
    TESTS10,
}

pub enum Type {
    PROGRAMMING, 
    QUESTION
}

#[derive(Debug, Deserialize)]
pub struct Problem {
    pub contestId: Option<i32>, 	
    pub problemsetName: String, 
    pub index: String,
    pub name: String,
    pub type_of: Type,
    pub points: Option<f64>, Floating point number. Can be absent. Maximum amount of points for the problem.
    pub rating: Option<i32>,Integer. Can be absent. Problem rating (difficulty).
    pub tags: Vec<String>String list. Problem tags.
}

#[derive(Debug, Deserialize)]
pub struct Party { 

}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Submission {
    pub id: i32, 	
    pub contestId: Option<i32>, 	
    pub creationTimeSeconds: i64, 	
    pub relativeTimeSeconds: i64, 	
    pub problem: Problem,
    pub author: Party,
    pub programmingLanguage: String,
    pub verdict: Option<Verdict>,
    pub testset: Testset,
    pub passedTestCount: i32, 
    pub timeConsumedMillis: i64, 
    pub memoryConsumedBytes: i64, 
    pub points: Option<f64> 	
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct User {
    pub handle: String,
    pub email: Option<String>,
    pub vk_id: Option<String>,
    pub open_id: Option<String>,
    pub first_name: Option<String>,
    pub last_name: Option<String>,
    pub country: Option<String>,
    pub city: Option<String>,
    pub organization: Option<String>,
    pub contribution: i32,
    pub rank: String,
    pub rating: i32,
    pub max_rank: String,
    pub max_rating: i32,
    pub last_online_time_seconds: i64,
    pub registration_time_seconds: i64,
    pub friend_of_count: i64,
    pub avatar: String,
    pub title_photo: String,
}

#[derive(Debug, Deserialize)]
struct ApiResponse<T> {
    status: String,
    comment: Option<String>,
    result: Option<T>,
}

pub struct CodeforcesApi {
    base_url: String,
    client: reqwest::blocking::Client,
}

impl CodeforcesApi {
    pub fn new(base_url: impl Into<String>) -> Self {
        CodeforcesApi {
            base_url: base_url.into(),
            client: reqwest::blocking::Client::builder()
                .timeout(std::time::Duration::from_secs(10))
                .build()
                .expect("failed to build HTTP client"),
        }
    }

    fn get<T: DeserializeOwned>(&self, method: &str, params: &[(&str, String)]) -> Result<T> {
        let url = format!("{}/{}", self.base_url.trim_end_matches('/'), method);
        let response = self
            .client
            .get(&url)
            .query(params)
            .send()?
            .error_for_status()?
            .json::<ApiResponse<T>>()?;

        match response.status.as_str() {
            "OK" => response
                .result
                .ok_or_else(|| "Codeforces response did not include a result field".into()),
            _ => Err(response
                .comment
                .unwrap_or_else(|| "Codeforces request failed".to_string())
                .into()),
        }
    }

    pub fn user_info(&self, handles: impl Into<String>) -> Result<Vec<User>> {
        self.get("user.info", &[("handles", handles.into())])
    }

    
}

impl Default for CodeforcesApi {
    fn default() -> Self {
        Self::new(DEFAULT_BASE_URL)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_live_user_info_logs_full_response() {
        let handle = "Andezion".to_string();
        let api = CodeforcesApi::default();
        let users = api.user_info(handle.as_str()).unwrap();

        println!("{users:#?}");

        assert_eq!(users.len(), 1);
        assert_eq!(users[0].handle, handle);
    }
}