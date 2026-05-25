use serde::de::DeserializeOwned;
use serde::Deserialize;

const DEFAULT_BASE_URL: &str = "https://codeforces.com/api";

type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

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

pub struct Submission {
    pub id: i32, 	
    contestId 	Integer. Can be absent.
    creationTimeSeconds 	Integer. Time, when submission was created, in unix-format.
    relativeTimeSeconds 	Integer. Number of seconds, passed after the start of the contest (or a virtual start for virtual parties), before the submission.
    problem 	Problem object.
    author 	Party object.
    programmingLanguage 	String.
    verdict 	Enum: FAILED, OK, PARTIAL, COMPILATION_ERROR, RUNTIME_ERROR, WRONG_ANSWER, WRONG_ANSWER, TIME_LIMIT_EXCEEDED, MEMORY_LIMIT_EXCEEDED, IDLENESS_LIMIT_EXCEEDED, SECURITY_VIOLATED, CRASHED, INPUT_PREPARATION_CRASHED, CHALLENGED, SKIPPED, TESTING, REJECTED, SUBMITTED. Can be absent.
    testset 	Enum: SAMPLES, PRETESTS, TESTS, CHALLENGES, TESTS1, ..., TESTS10. Testset used for judging the submission.
    passedTestCount 	Integer. Number of passed tests.
    timeConsumedMillis 	Integer. Maximum time in milliseconds, consumed by solution for one test.
    memoryConsumedBytes 	Integer. Maximum memory in bytes, consumed by solution for one test.
    points 	Floating point number. Can be absent. Number of scored points for IOI-like contests.
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