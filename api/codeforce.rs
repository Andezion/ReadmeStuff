use serde::Deserialize;

const DEFAULT_BASE_URL: &str = "https://www.codewars.com/api/";

#[derive(Deserialize)]
pub struct User {
    pub handle: String,
    pub email: String,
    pub vkid: String,
    #[serde(rename = "openId")]
    pub openid: String,
    #[serde(rename = "firstName")]
    pub first_name: String,
    #[serde(rename = "lastName")]
    pub last_name: String,
    pub country: String,
    pub city: String,
    pub organization: String,
    pub contribution: i32,
    pub rank: String,
    pub rating: i32,
    #[serde(rename = "maxRank")]
    pub max_rank: String,
    #[serde(rename = "maxRating")]
    pub max_rating: i32,
    #[serde(rename = "lastOnlineTime")]
    pub last_online_time: i64,
    #[serde(rename = "registrationTime")]
    pub registration_time: i64,
    #[serde(rename = "friendOf")]
    pub friend_of: i64,
    pub avatar: String,
    #[serde(rename = "titlePhoto")]
    pub title_photo: String,
}

impl CodeforcesApi {
    
}