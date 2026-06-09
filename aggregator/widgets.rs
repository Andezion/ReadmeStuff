use crate::models::UserProfile;

pub struct StreakWidget {
    pub current_streak: u32,
    pub longest_streak: u32,
    pub total_contributions: u32,
    pub average_daily: f64,
}

pub struct LangBar {
    pub name: String,
    pub percentage: f64,
    pub color: Option<String>,
}

pub struct LangsWidget {
    pub top: Vec<LangBar>,
}

pub struct GithubStatsWidget {
    pub login: String,
    pub name: Option<String>,
    pub stars: u64,
    pub commits: u32,
    pub prs: u32,
    pub issues: u32,
    pub followers: u32,
    pub rank: String,
    pub rank_percentile: f64,
}

pub struct CompetitiveWidget {
    pub cf_rating: Option<i32>,
    pub cf_rank: Option<String>,
    pub cw_rank: Option<String>,
    pub cw_honor: Option<u32>,
    pub lc_solved: Option<u32>,
    pub lc_easy: Option<u32>,
    pub lc_medium: Option<u32>,
    pub lc_hard: Option<u32>,
}

pub fn streak_widget(p: &UserProfile) -> Option<StreakWidget> {
    let s = p.streak.as_ref()?;
    Some(StreakWidget {
        current_streak: s.current_streak,
        longest_streak: s.longest_streak,
        total_contributions: s.total_contributions,
        average_daily: s.average_daily_contributions,
    })
}

pub fn langs_widget(p: &UserProfile, top_n: usize) -> Option<LangsWidget> {
    let l = p.langs.as_ref()?;
    let top = l
        .languages
        .iter()
        .take(top_n)
        .map(|lang| LangBar {
            name: lang.name.clone(),
            percentage: lang.percentage,
            color: lang.color.clone(),
        })
        .collect();
    Some(LangsWidget { top })
}

pub fn github_stats_widget(p: &UserProfile) -> Option<GithubStatsWidget> {
    let g = p.github.as_ref()?;
    Some(GithubStatsWidget {
        login: g.metadata.login.clone(),
        name: g.metadata.name.clone(),
        stars: g.repos.total_stars,
        commits: g.contributions.total_commits,
        prs: g.contributions.total_pull_requests,
        issues: g.contributions.total_issues,
        followers: g.metadata.followers,
        rank: g.rank.grade.clone(),
        rank_percentile: g.rank.percentile,
    })
}

pub fn competitive_widget(p: &UserProfile) -> Option<CompetitiveWidget> {
    if p.codeforces.is_none() && p.codewars.is_none() && p.leetcode.is_none() {
        return None;
    }
    Some(CompetitiveWidget {
        cf_rating: p.codeforces.as_ref().map(|d| d.user.rating),
        cf_rank: p.codeforces.as_ref().map(|d| d.user.rank.clone()),
        cw_rank: p.codewars.as_ref().map(|u| u.ranks.overall.name.clone()),
        cw_honor: p.codewars.as_ref().map(|u| u.honor),
        lc_solved: p.leetcode.as_ref().map(|d| d.solved.solved_problem),
        lc_easy: p.leetcode.as_ref().map(|d| d.solved.easy_solved),
        lc_medium: p.leetcode.as_ref().map(|d| d.solved.medium_solved),
        lc_hard: p.leetcode.as_ref().map(|d| d.solved.hard_solved),
    })
}
