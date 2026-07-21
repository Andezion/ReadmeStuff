use crate::schema::{Config, Layout, PlacedWidget, ProfileConfig, Row, ThemeChoice};


pub fn default_config() -> Config {
    Config {
        profile: ProfileConfig {
            github_login: Some("Andezion".to_string()),
            github_token_env: Some("GITHUB_TOKEN".to_string()),
            codeforces_handle: Some("Andezion".to_string()),
            codewars_username: Some("Andezion".to_string()),
            leetcode_username: Some("Andezion".to_string()),
        },
        theme: ThemeChoice::Matrix,
        layout: Layout {
            canvas_width: 990,
            rows: default_rows(),
        },
    }
}

fn widget(id: &str, x: u32, y: u32) -> PlacedWidget {
    PlacedWidget {
        id: id.to_string(),
        x,
        y,
    }
}

fn default_rows() -> Vec<Row> {
    vec![
        // github-repos / github-social / github-stats / langs
        Row {
            widgets: vec![
                widget("github-repos", 0, 100),
                widget("github-social", 495, 100),
                widget("github-stats", 0, 220),
                widget("langs", 495, 220),
            ],
        },
        // github-commit-streak / github-contributions
        Row {
            widgets: vec![
                widget("github-commit-streak", 495, 0),
                widget("github-contributions", 495, 150),
            ],
        },
        // github-heatmap / github-monthly
        Row {
            widgets: vec![
                widget("github-heatmap", 0, 80),
                widget("github-monthly", 495, 80),
            ],
        },
        // github-visitors / github-engagement
        Row {
            widgets: vec![
                widget("github-visitors", 0, 0),
                widget("github-engagement", 495, 0),
            ],
        },
        // competitive
        Row {
            widgets: vec![widget("competitive", 495, 0)],
        },
        // lc-languages / lc-solved / lc-skills / lc-badges
        Row {
            widgets: vec![
                widget("lc-languages", 0, 181),
                widget("lc-solved", 0, 398),
                widget("lc-skills", 495, 0),
                widget("lc-badges", 495, 355),
            ],
        },
        // cf-rating / cf-stats
        Row {
            widgets: vec![widget("cf-rating", 495, 0), widget("cf-stats", 495, 165)],
        },
        // cw-rank / cw-kata / cw-languages
        Row {
            widgets: vec![
                widget("cw-rank", 495, 0),
                widget("cw-kata", 0, 165),
                widget("cw-languages", 495, 165),
            ],
        },
    ]
}
