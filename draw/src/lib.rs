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
