use serde::Deserialize;
use serde::de::DeserializeOwned;

const DEFAULT_BASE_URL: &str = "https://codeforces.com/api";

type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

#[derive(Debug, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum Verdict {
    Failed,
    Ok,
    Partial,
    CompilationError,
    RuntimeError,
    WrongAnswer,
    TimeLimitExceeded,
    MemoryLimitExceeded,
    IdlenessLimitExceeded,
    SecurityViolated,
    Crashed,
    InputPreparationCrashed,
    Challenged,
    Skipped,
    Testing,
    Rejected,
    Submitted,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum Testset {
    Samples,
    Pretests,
    Tests,
    Challenges,
    Tests1,
    Tests2,
    Tests3,
    Tests4,
    Tests5,
    Tests6,
    Tests7,
    Tests8,
    Tests9,
    Tests10,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum Type {
    Programming,
    Question,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Problem {
    pub contest_id: Option<i32>,
    pub problemset_name: Option<String>,
    pub index: String,
    pub name: String,
    #[serde(rename = "type")]
    pub kind: Type,
    pub points: Option<f64>,
    pub rating: Option<i32>,
    pub tags: Vec<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Member {
    pub handle: String,
    pub name: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ParticipantType {
    Contestant,
    Practice,
    Virtual,
    Manager,
    OutOfCompetition,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RatingChange {
    pub contest_id: i32,
    pub contest_name: String,
    pub handle: String,
    pub rank: i32,
    #[serde(rename = "ratingUpdateTimeSeconds")]
    pub rating_update_time_seconds: i32,
    pub old_rating: i32,
    pub new_rating: i32,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Party {
    pub contest_id: Option<i32>,
    pub members: Vec<Member>,
    pub participant_type: ParticipantType,
    pub team_id: Option<i32>,
    pub team_name: Option<String>,
    pub ghost: bool,
    pub room: Option<i32>,
    pub start_time_seconds: Option<i32>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Submission {
    pub id: i32,
    pub contest_id: Option<i32>,
    pub creation_time_seconds: i64,
    pub relative_time_seconds: i64,
    pub problem: Problem,
    pub author: Party,
    pub programming_language: String,
    pub verdict: Option<Verdict>,
    pub testset: Testset,
    pub passed_test_count: i32,
    pub time_consumed_millis: i64,
    pub memory_consumed_bytes: i64,
    pub points: Option<f64>,
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

    pub fn user_status(
        &self,
        handle: impl Into<String>,
        from: Option<i32>,
        count: Option<i32>,
        include_sources: Option<bool>,
    ) -> Result<Vec<Submission>> {
        let mut params: Vec<(&str, String)> = Vec::new();
        params.push(("handle", handle.into()));
        if let Some(f) = from {
            params.push(("from", f.to_string()));
        }
        if let Some(c) = count {
            params.push(("count", c.to_string()));
        }
        if let Some(s) = include_sources {
            params.push(("includeSources", s.to_string()));
        }

        self.get("user.status", &params)
    }

    pub fn user_rated_list(
        &self,
        active_only: Option<bool>,
        include_retired: Option<bool>,
        contest_id: Option<i32>,
    ) -> Result<Vec<User>> {
        let mut params: Vec<(&str, String)> = Vec::new();
        if let Some(a) = active_only {
            params.push(("activeOnly", a.to_string()));
        }
        if let Some(ir) = include_retired {
            params.push(("includeRetired", ir.to_string()));
        }
        if let Some(cid) = contest_id {
            params.push(("contestId", cid.to_string()));
        }

        self.get("user.ratedList", &params)
    }

    pub fn user_rating(&self, handle: impl Into<String>) -> Result<Vec<RatingChange>> {
        self.get("user.rating", &[("handle", handle.into())])
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

    /*

    [
        User {
            handle: "Andezion",
            email: None,
            vk_id: None,
            open_id: None,
            first_name: None,
            last_name: None,
            country: None,
            city: None,
            organization: None,
            contribution: 0,
            rank: "newbie",
            rating: 1148,
            max_rank: "newbie",
            max_rating: 1148,
            last_online_time_seconds: 1776974379,
            registration_time_seconds: 1729697721,
            friend_of_count: 0,
            avatar: "https://userpic.codeforces.org/4365023/avatar/b64550ec3c9d18f6.jpg",
            title_photo: "https://userpic.codeforces.org/4365023/title/760c02ca07760e71.jpg",
        },
    ]

     */

    #[test]
    fn test_live_user_status_logs_full_response() {
        let handle = "Andezion";
        let api = CodeforcesApi::default();
        let subs = api
            .user_status(handle, Some(1), Some(20), Some(false))
            .unwrap();

        println!("{subs:#?}");

        assert!(!subs.is_empty());
    }

    #[test]
    fn test_live_user_rated_list_logs_full_response() {
        let api = CodeforcesApi::default();
        let list = api.user_rated_list(Some(true), Some(false), None).unwrap();
        let found = list.iter().any(|u| u.handle == "Andezion");
        println!("Found Andezion in rated list: {}", found);

        assert!(!list.is_empty());
    }

    #[test]
    fn test_live_user_rating_logs_full_response() {
        let handle = "Andezion";
        let api = CodeforcesApi::default();
        let ratings = api.user_rating(handle).unwrap();

        println!("{ratings:#?}");

        assert!(!ratings.is_empty());
    }

    /*

    [
        RatingChange {
            contest_id: 2114,
            contest_name: "Codeforces Round 1027 (Div. 3)",
            handle: "Andezion",
            rank: 17471,
            rating_update_time_seconds: 1748278200,
            old_rating: 0,
            new_rating: 385,
        },
        RatingChange {
            contest_id: 2117,
            contest_name: "Codeforces Round 1029 (Div. 3)",
            handle: "Andezion",
            rank: 17699,
            rating_update_time_seconds: 1749401400,
            old_rating: 385,
            new_rating: 622,
        },
        RatingChange {
            contest_id: 2113,
            contest_name: "Codeforces Round 1031 (Div. 2)",
            handle: "Andezion",
            rank: 9578,
            rating_update_time_seconds: 1749985500,
            old_rating: 622,
            new_rating: 779,
        },
        RatingChange {
            contest_id: 2160,
            contest_name: "Codeforces Round 1058 (Div. 2)",
            handle: "Andezion",
            rank: 6857,
            rating_update_time_seconds: 1760288700,
            old_rating: 779,
            new_rating: 952,
        },
        RatingChange {
            contest_id: 2161,
            contest_name: "Pinely Round 5 (Div. 1 + Div. 2)",
            handle: "Andezion",
            rank: 2895,
            rating_update_time_seconds: 1761852900,
            old_rating: 952,
            new_rating: 1148,
        },
    ]

     */
}
