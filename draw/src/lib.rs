pub mod theme;
mod helpers;
mod matrix;

// GitHub
mod github_stats;
mod github_repos;
mod github_contributions;
mod github_social;
mod github_heatmap;
mod github_monthly;

// Codeforces
mod cf_rating;
mod cf_stats;

// Codewars
mod cw_rank;
mod cw_kata;
mod cw_languages;

// LeetCode
mod lc_solved;
mod lc_skills;
mod lc_languages;
mod lc_badges;

// Legacy combined card
mod streak;
mod langs;
mod competitive;

pub use theme::Theme;

// GitHub
pub use github_stats::render_github_stats;
pub use github_repos::render_github_repos;
pub use github_contributions::render_github_contributions;
pub use github_social::render_github_social;
pub use github_heatmap::render_github_heatmap;
pub use github_monthly::render_github_monthly;

// GitHub streak / langs (kept)
pub use streak::render_streak;
pub use langs::render_langs;

// Codeforces
pub use cf_rating::render_cf_rating;
pub use cf_stats::render_cf_stats;

// Codewars
pub use cw_rank::render_cw_rank;
pub use cw_kata::render_cw_kata;
pub use cw_languages::render_cw_languages;

// LeetCode
pub use lc_solved::render_lc_solved;
pub use lc_skills::render_lc_skills;
pub use lc_languages::render_lc_languages;
pub use lc_badges::render_lc_badges;

// Legacy
pub use competitive::render_competitive;
