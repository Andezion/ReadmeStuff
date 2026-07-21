pub mod compose;
mod helpers;
mod matrix;
pub mod theme;

// Custom text-to-SVG rendering
mod text_card;
mod text_glyph;
mod text_input;

// GitHub
mod github_contributions;
mod github_heatmap;
mod github_monthly;
mod github_repos;
mod github_social;
mod github_stats;
mod github_visitors;

// Codeforces
mod cf_rating;
mod cf_stats;

// Codewars
mod cw_kata;
mod cw_languages;
mod cw_rank;

// LeetCode
mod lc_badges;
mod lc_languages;
mod lc_skills;
mod lc_solved;

// Legacy combined card
mod competitive;
mod langs;
mod streak;

pub use compose::{Tile, compose};
pub use theme::Theme;

pub mod sizes {
    pub const GITHUB_STATS: (u32, u32) = crate::github_stats::SIZE;
    pub const GITHUB_REPOS: (u32, u32) = crate::github_repos::SIZE;
    pub const GITHUB_CONTRIBUTIONS: (u32, u32) = crate::github_contributions::SIZE;
    pub const GITHUB_SOCIAL: (u32, u32) = crate::github_social::SIZE;
    pub const GITHUB_HEATMAP: (u32, u32) = crate::github_heatmap::SIZE;
    pub const GITHUB_MONTHLY: (u32, u32) = crate::github_monthly::SIZE;
    pub const GITHUB_VISITORS: (u32, u32) = crate::github_visitors::VISITORS_SIZE;
    pub const GITHUB_ENGAGEMENT: (u32, u32) = crate::github_visitors::ENGAGEMENT_SIZE;
    pub const GITHUB_COMMIT_STREAK: (u32, u32) = crate::github_visitors::COMMIT_STREAK_SIZE;

    pub const STREAK: (u32, u32) = crate::streak::SIZE;
    pub const LANGS: (u32, u32) = crate::langs::SIZE;

    pub const CF_RATING: (u32, u32) = crate::cf_rating::SIZE;
    pub const CF_STATS: (u32, u32) = crate::cf_stats::SIZE;

    pub const CW_RANK: (u32, u32) = crate::cw_rank::SIZE;
    pub const CW_KATA: (u32, u32) = crate::cw_kata::SIZE;
    pub const CW_LANGUAGES: (u32, u32) = crate::cw_languages::SIZE;

    pub const LC_SOLVED: (u32, u32) = crate::lc_solved::SIZE;
    pub const LC_SKILLS: (u32, u32) = crate::lc_skills::SIZE;
    pub const LC_LANGUAGES: (u32, u32) = crate::lc_languages::SIZE;
    pub const LC_BADGES: (u32, u32) = crate::lc_badges::SIZE;

    pub const COMPETITIVE: (u32, u32) = crate::competitive::SIZE;
}

// GitHub
pub use github_contributions::render_github_contributions;
pub use github_heatmap::render_github_heatmap;
pub use github_monthly::render_github_monthly;
pub use github_repos::render_github_repos;
pub use github_social::render_github_social;
pub use github_stats::render_github_stats;
pub use github_visitors::render_github_commit_streak;
pub use github_visitors::render_github_engagement;
pub use github_visitors::render_github_visitors;

// GitHub streak / langs (kept)
pub use langs::render_langs;
pub use streak::render_streak;

// Codeforces
pub use cf_rating::render_cf_rating;
pub use cf_stats::render_cf_stats;

// Codewars
pub use cw_kata::render_cw_kata;
pub use cw_languages::render_cw_languages;
pub use cw_rank::render_cw_rank;

// LeetCode
pub use lc_badges::render_lc_badges;
pub use lc_languages::render_lc_languages;
pub use lc_skills::render_lc_skills;
pub use lc_solved::render_lc_solved;

// Legacy
pub use competitive::render_competitive;

// Custom text-to-SVG rendering
pub use text_card::{DEFAULT_HEIGHT, DEFAULT_WIDTH, render_text_card};
pub use text_glyph::{Align, HAlign, TextLine, VAlign};
pub use text_input::parse_lines;
