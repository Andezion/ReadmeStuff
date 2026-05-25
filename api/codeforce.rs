use serde::Deserialize;

const DEFAULT_BASE_URL: &str = "https://www.codewars.com/api/";

#[derive(Deserialize)]
pub struct User {
    pub handle: String,
    pub email: String,
    pub vkid: String,
    #[serde(rename = "openId")]
    pub openid: String,
    pub firstName: String,
    pub lastName: String,   
    pub country: String,
    pub city: String,
    pub organization: String,
    pub contribution: i32,
    pub rank: String,
    pub rating: i32,
    pub maxRank: String,
    pub maxRating: i32,
    pub lastOnlineTime: i64,
    pub registrationTime: i64,
    pub friendOf: i64,
    pub avatar: String,
    pub titlePhoto: String,
}

impl CodeforcesApi {
    
}