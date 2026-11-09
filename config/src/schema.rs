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
    pub linkedin_name: Option<String>,
    pub linkedin_url: Option<String>,
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
        if self.linkedin_url.is_some() {
            set.insert(Credential::LinkedIn);
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

impl ThemeChoice {
    pub const ALL: [ThemeChoice; 1] = [ThemeChoice::Matrix];

    pub fn slug(self) -> &'static str {
        match self {
            ThemeChoice::Matrix => "matrix",
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, Default, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub enum ColorChoice {
    #[default]
    Green,
    Cyan,
    Blue,
    Purple,
    Pink,
    Red,
    Orange,
    Yellow,
    White,
    Rainbow,
}

impl ColorChoice {
    pub const ALL: [ColorChoice; 10] = [
        ColorChoice::Green,
        ColorChoice::Cyan,
        ColorChoice::Blue,
        ColorChoice::Purple,
        ColorChoice::Pink,
        ColorChoice::Red,
        ColorChoice::Orange,
        ColorChoice::Yellow,
        ColorChoice::White,
        ColorChoice::Rainbow,
    ];

    pub fn slug(self) -> &'static str {
        match self {
            ColorChoice::Green => "green",
            ColorChoice::Cyan => "cyan",
            ColorChoice::Blue => "blue",
            ColorChoice::Purple => "purple",
            ColorChoice::Pink => "pink",
            ColorChoice::Red => "red",
            ColorChoice::Orange => "orange",
            ColorChoice::Yellow => "yellow",
            ColorChoice::White => "white",
            ColorChoice::Rainbow => "rainbow",
        }
    }
}

pub fn mosaic_file_name(theme: ThemeChoice, color: ColorChoice) -> String {
    format!("profile-mosaic_{}_{}.svg", theme.slug(), color.slug())
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, Default, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub enum FontChoice {
    #[default]
    Monospace,
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
pub struct TextCardConfig {
    pub file: Option<String>,
    pub output: Option<String>,
    pub align: Option<String>,
    pub centered: bool,
    pub width: Option<u32>,
    pub height: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq)]
#[serde(default)]
pub struct Config {
    pub profile: ProfileConfig,
    pub theme: ThemeChoice,
    pub color: ColorChoice,
    pub font: FontChoice,
    pub layout: Layout,
    pub text_card: TextCardConfig,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn mosaic_name_carries_theme_and_color() {
        assert_eq!(
            mosaic_file_name(ThemeChoice::Matrix, ColorChoice::Green),
            "profile-mosaic_matrix_green.svg"
        );
        assert_eq!(
            mosaic_file_name(ThemeChoice::Matrix, ColorChoice::Rainbow),
            "profile-mosaic_matrix_rainbow.svg"
        );
    }

    #[test]
    fn slugs_match_serde_names() {
        for color in ColorChoice::ALL {
            let toml = toml::to_string(&Config {
                color,
                ..Default::default()
            })
            .unwrap();
            assert!(toml.contains(&format!("color = \"{}\"", color.slug())));
        }
        for theme in ThemeChoice::ALL {
            let toml = toml::to_string(&Config {
                theme,
                ..Default::default()
            })
            .unwrap();
            assert!(toml.contains(&format!("theme = \"{}\"", theme.slug())));
        }
    }

    #[test]
    fn config_without_color_defaults_to_green() {
        let cfg: Config = toml::from_str("theme = \"matrix\"").unwrap();
        assert_eq!(cfg.color, ColorChoice::Green);
    }
}
