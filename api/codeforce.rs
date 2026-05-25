use serde::Deserialize;

const DEFAULT_BASE_URL: &str = "https://www.codewars.com/api/";

#[derive(Deserialize)]
pub struct User {
    pub handle: String,
    pub email: String,
    pub vkid: String,
    #[serde(rename = "openId")]
    pub openid: String,

}

impl CodeforcesApi {
    
}