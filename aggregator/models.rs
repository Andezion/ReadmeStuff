use readme_stuff_api::{
    github_statistic::ProfileStats,
    github_streak::StreakStats,
    github_langs::AggregatedLangStats,
    codeforce::{User as CodeforcesUser, RatingChange},
    codewars::User as CodewarsUser,
    leetcode::{Solved, Language as LeetcodeLanguage},
    github_visitors::models::VisitorAnalytics,
};

pub struct CodeforcesData {
    pub user: CodeforcesUser,
    pub rating_history: Vec<RatingChange>,
}

pub struct LeetcodeData {
    pub solved: Solved,
    pub languages: Vec<LeetcodeLanguage>,
}

pub struct UserProfile {
    pub github:     Option<ProfileStats>,
    pub streak:     Option<StreakStats>,
    pub langs:      Option<AggregatedLangStats>,
    pub codeforces: Option<CodeforcesData>,
    pub codewars:   Option<CodewarsUser>,
    pub leetcode:   Option<LeetcodeData>,
    pub visitors:   Option<VisitorAnalytics>,
    pub sources:    SourceStatus,
}

pub struct SourceStatus {
    pub github:     Result<(), String>,
    pub codeforces: Result<(), String>,
    pub codewars:   Result<(), String>,
    pub leetcode:   Result<(), String>,
    pub visitors:   Result<(), String>,
}
