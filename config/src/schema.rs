use crate::credential::Credential;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq)]
#[serde(default)]
pub struct ProfileConfig {
    pub github_login: Option<String>,
    pub github_token_env: Option<String>,
    pub codeforces_handle: Option<String>,
    pub codewars_username: Option<String>,
    pub leetcode_username: Option<String>,
}

impl ProfileConfig {
    pub fn available_credentials(&self) -> std::collections::HashSet<Credential> {
        let mut set = std::collections::HashSet::new();
        if self.github_login.is_some() && self.github_token_env.is_some() {
            set.insert(Credential::GitHubLogin);
            set.insert(Credential::GitHubToken);
        }
        if self.codeforces_handle.is_some() {
            set.insert(Credential::CodeforcesHandle);
        }
        if self.codewars_username.is_some() {
            set.insert(Credential::CodewarsUsername);
        }
        if self.leetcode_username.is_some() {
            set.insert(Credential::LeetcodeUsername);
        }
        set
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, Default, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub enum ThemeChoice {
    #[default]
    Matrix,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct PlacedWidget {
    pub id: String,
    #[serde(default)]
    pub x: u32,
    #[serde(default)]
    pub y: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq, Eq)]
#[serde(default)]
pub struct Row {
    pub widgets: Vec<PlacedWidget>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(default)]
pub struct Layout {
    pub canvas_width: u32,
    pub rows: Vec<Row>,
}

impl Default for Layout {
    fn default() -> Self {
        Layout {
            canvas_width: 990,
            rows: Vec::new(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq)]
#[serde(default)]
pub struct Config {
    pub profile: ProfileConfig,
    pub theme: ThemeChoice,
    pub layout: Layout,
}
