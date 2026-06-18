use readme_stuff_api::{
    codeforce::{RatingChange, Submission, User as CodeforcesUser},
    codewars::User as CodewarsUser,
    github_commit_streak::CommitStreakStats,
    github_langs::AggregatedLangStats,
    github_statistic::ProfileStats,
    github_streak::StreakStats,
    github_visitors::models::VisitorAnalytics,
    leetcode::{BadgesResponse, Language as LeetcodeLanguage, Skill as LeetcodeSkill, Solved},
};

pub struct CodeforcesData {
    pub user: CodeforcesUser,
    pub rating_history: Vec<RatingChange>,
    pub submissions: Vec<Submission>,
}

pub struct LeetcodeData {
    pub solved: Solved,
    pub languages: Vec<LeetcodeLanguage>,
    pub skills_advanced: Vec<LeetcodeSkill>,
    pub skills_intermediate: Vec<LeetcodeSkill>,
    pub skills_fundamental: Vec<LeetcodeSkill>,
    pub badges: BadgesResponse,
}

pub struct UserProfile {
    pub github: Option<ProfileStats>,
    pub streak: Option<StreakStats>,
    pub commit_streak: Option<CommitStreakStats>,
    pub langs: Option<AggregatedLangStats>,
    pub codeforces: Option<CodeforcesData>,
    pub codewars: Option<CodewarsUser>,
    pub leetcode: Option<LeetcodeData>,
    pub visitors: Option<VisitorAnalytics>,
    pub sources: SourceStatus,
}

pub struct SourceStatus {
    pub github: Result<(), String>,
    pub codeforces: Result<(), String>,
    pub codewars: Result<(), String>,
    pub leetcode: Result<(), String>,
    pub visitors: Result<(), String>,
    pub commit_streak: Result<(), String>,
}
