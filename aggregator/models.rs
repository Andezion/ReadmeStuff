use readme_stuff_api::{
    github_statistic::ProfileStats,
    github_streak::StreakStats,
    github_langs::AggregatedLangStats,
    codeforce::{User as CfUser, RatingChange},
    codewars::User as CodewarsUser,
    leetcode::{Solved, Language as LcLanguage},
    github_visitors::models::VisitorAnalytics,
};

pub struct CodeforcesData {
    pub user: CfUser,
    pub rating_history: Vec<RatingChange>,
}

pub struct LeetcodeData {
    pub solved: Solved,
    pub languages: Vec<LcLanguage>,
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
