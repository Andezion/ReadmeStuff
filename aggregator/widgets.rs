use std::collections::{HashMap, HashSet};

use readme_stuff_api::codeforce::Verdict;
use readme_stuff_api::github_visitors::models::TrendHighlight;

use crate::models::UserProfile;

pub struct GithubVisitorsWidget {
    pub total_views: u64,
    pub total_unique: u64,
    pub total_clones: u64,
    pub total_unique_cloners: u64,
    pub top_repos: Vec<(String, u64, f64)>,
    pub top_referrer: Option<(String, u64)>,
    pub growth_rate_pct: f64,
    pub is_growing: bool,
    pub peak_day: Option<chrono::NaiveDate>,
    pub peak_value: u64,
    pub highlight: Option<TrendHighlight>,
    pub weekly_views: Vec<(chrono::NaiveDate, u64)>,
    pub referrer_trend: Vec<(String, String, u64)>,
}

pub fn github_visitors_widget(p: &UserProfile) -> Option<GithubVisitorsWidget> {
    let v = p.visitors.as_ref()?;
    let total_views: u64 = v.repositories.iter().map(|r| r.total_views_all_time).sum();
    let total_unique: u64 = v
        .repositories
        .iter()
        .map(|r| r.total_unique_visitors_all_time)
        .sum();
    if total_views == 0 {
        return None;
    }
    let top_repos = v
        .repositories
        .iter()
        .filter(|r| r.total_views_all_time > 0)
        .map(|r| {
            (
                r.repo.clone(),
                r.total_views_all_time,
                r.trend.growth_rate_pct,
            )
        })
        .collect();
    Some(GithubVisitorsWidget {
        total_views,
        total_unique,
        total_clones: v.total_clones_all_time,
        total_unique_cloners: v.total_unique_cloners_all_time,
        top_repos,
        top_referrer: v
            .top_referrers
            .first()
            .map(|r| (r.referrer.clone(), r.count)),
        growth_rate_pct: v.trend.growth_rate_pct,
        is_growing: v.trend.is_growing,
        peak_day: v.trend.peak_day,
        peak_value: v.trend.peak_value,
        highlight: v.trend.highlight.clone(),
        weekly_views: v.weekly_views.clone(),
        referrer_trend: v.referrer_trend.clone(),
    })
}

pub struct EngagementWidget {
    pub total_stars: u64,
    pub total_forks: u64,
    pub total_watchers: u64,
    pub recent_stargazers: Vec<(String, String)>,
}

pub fn github_engagement_widget(p: &UserProfile) -> Option<EngagementWidget> {
    let e = p.engagement.as_ref()?;
    if e.total_stars == 0 {
        return None;
    }
    Some(EngagementWidget {
        total_stars: e.total_stars,
        total_forks: e.total_forks,
        total_watchers: e.total_watchers,
        recent_stargazers: e.recent_stargazers.clone(),
    })
}

pub struct CommitStreakWidget {
    pub total_commits: u64,
    pub days_with_commits: u32,
    pub current_streak: u32,
    pub longest_streak: u32,
    pub longest_streak_start: Option<chrono::NaiveDate>,
    pub longest_streak_end: Option<chrono::NaiveDate>,
}

pub fn commit_streak_widget(p: &UserProfile) -> Option<CommitStreakWidget> {
    let s = p.commit_streak.as_ref()?;
    Some(CommitStreakWidget {
        total_commits: s.total_commits,
        days_with_commits: s.days_with_commits,
        current_streak: s.current_streak,
        longest_streak: s.longest_streak,
        longest_streak_start: s.longest_streak_start,
        longest_streak_end: s.longest_streak_end,
    })
}

pub struct StreakWidget {
    pub current_streak: u32,
    pub longest_streak: u32,
    pub total_contributions: u32,
    pub average_daily: f64,
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

pub struct LangBar {
    pub name: String,
    pub percentage: f64,
    pub color: Option<String>,
}

pub struct LangsWidget {
    pub top: Vec<LangBar>,
    pub source: &'static str,
}

static LANG_COLORS: &[(&str, &str)] = &[
    ("Rust", "#dea584"),
    ("C++", "#f34b7d"),
    ("C", "#555555"),
    ("C#", "#178600"),
    ("Python", "#3572A5"),
    ("JavaScript", "#f1e05a"),
    ("TypeScript", "#3178c6"),
    ("Java", "#b07219"),
    ("Go", "#00ADD8"),
    ("Kotlin", "#A97BFF"),
    ("Swift", "#F05138"),
    ("Ruby", "#701516"),
    ("PHP", "#4F5D95"),
    ("Dart", "#00B4AB"),
    ("Scala", "#c22d40"),
    ("Shell", "#89e051"),
    ("HTML", "#e34c26"),
    ("CSS", "#563d7c"),
    ("Zig", "#ec915c"),
    ("Haskell", "#5e5086"),
    ("Lua", "#000080"),
    ("R", "#198CE7"),
    ("Perl", "#0298c3"),
    ("Elixir", "#6e4a7e"),
    ("OCaml", "#3be133"),
    ("CMake", "#DA3434"),
];

fn lang_fallback_color(name: &str) -> Option<String> {
    LANG_COLORS
        .iter()
        .find(|(n, _)| n.eq_ignore_ascii_case(name))
        .map(|(_, c)| c.to_string())
}

pub fn langs_widget(p: &UserProfile, top_n: usize) -> Option<LangsWidget> {
    if let Some(g) = p.github.as_ref() {
        let cbl = &g.contributions.commits_by_language;
        if !cbl.is_empty() {
            let total: u32 = cbl.values().sum();
            if total > 0 {
                let color_lookup: HashMap<&str, &str> = p
                    .langs
                    .as_ref()
                    .map(|l| {
                        l.languages
                            .iter()
                            .filter_map(|ls| ls.color.as_deref().map(|c| (ls.name.as_str(), c)))
                            .collect()
                    })
                    .unwrap_or_default();

                let mut langs: Vec<(String, u32)> =
                    cbl.iter().map(|(k, v)| (k.clone(), *v)).collect();
                langs.sort_by(|a, b| b.1.cmp(&a.1));

                let top = langs
                    .into_iter()
                    .take(top_n)
                    .map(|(name, count)| {
                        let color = color_lookup
                            .get(name.as_str())
                            .map(|s| s.to_string())
                            .or_else(|| lang_fallback_color(&name));
                        LangBar {
                            percentage: count as f64 / total as f64 * 100.0,
                            color,
                            name,
                        }
                    })
                    .collect();

                return Some(LangsWidget {
                    top,
                    source: "commits (this year)",
                });
            }
        }
    }

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
    Some(LangsWidget {
        top,
        source: "bytes",
    })
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

pub struct GithubReposWidget {
    pub total_repos: u32,
    pub total_stars: u64,
    pub total_forks: u64,
    pub total_watchers: u64,
}

pub fn github_repos_widget(p: &UserProfile) -> Option<GithubReposWidget> {
    let g = p.github.as_ref()?;
    Some(GithubReposWidget {
        total_repos: g.repos.total_repos,
        total_stars: g.repos.total_stars,
        total_forks: g.repos.total_forks,
        total_watchers: g.repos.total_watchers,
    })
}

pub struct GithubContributionsWidget {
    pub total_commits: u32,
    pub total_prs: u32,
    pub total_issues: u32,
    pub total_reviews: u32,
    pub repos_contributed_to: u32,
}

pub fn github_contributions_widget(p: &UserProfile) -> Option<GithubContributionsWidget> {
    let g = p.github.as_ref()?;
    let c = &g.contributions;
    Some(GithubContributionsWidget {
        total_commits: c.total_commits,
        total_prs: c.total_pull_requests,
        total_issues: c.total_issues,
        total_reviews: c.total_pull_request_reviews,
        repos_contributed_to: c.repos_contributed_to,
    })
}

pub struct GithubSocialWidget {
    pub followers: u32,
    pub following: u32,
}

pub fn github_social_widget(p: &UserProfile) -> Option<GithubSocialWidget> {
    let g = p.github.as_ref()?;
    Some(GithubSocialWidget {
        followers: g.metadata.followers,
        following: g.metadata.following,
    })
}

pub struct GithubHeatmapWidget {
    pub weekday_distribution: [u32; 7],
}

pub fn github_heatmap_widget(p: &UserProfile) -> Option<GithubHeatmapWidget> {
    let s = p.streak.as_ref()?;
    if s.weekday_distribution.iter().all(|&v| v == 0) {
        return None;
    }
    Some(GithubHeatmapWidget {
        weekday_distribution: s.weekday_distribution,
    })
}

pub struct GithubMonthlyWidget {
    pub months: Vec<(String, u32)>,
}

pub fn github_monthly_widget(p: &UserProfile) -> Option<GithubMonthlyWidget> {
    let s = p.streak.as_ref()?;
    if s.monthly_totals.is_empty() {
        return None;
    }
    let mut pairs: Vec<(String, u32)> = s
        .monthly_totals
        .iter()
        .map(|(k, v)| (k.clone(), *v))
        .collect();
    pairs.sort_by(|a, b| a.0.cmp(&b.0));

    let months = pairs
        .into_iter()
        .rev()
        .take(12)
        .rev()
        .map(|(key, count)| {
            let parts: Vec<&str> = key.split('-').collect();
            let label = if parts.len() == 2 {
                let m = match parts[1] {
                    "01" => "Jan",
                    "02" => "Feb",
                    "03" => "Mar",
                    "04" => "Apr",
                    "05" => "May",
                    "06" => "Jun",
                    "07" => "Jul",
                    "08" => "Aug",
                    "09" => "Sep",
                    "10" => "Oct",
                    "11" => "Nov",
                    "12" => "Dec",
                    _ => "?",
                };
                format!("{} {}", m, &parts[0][2..])
            } else {
                key.clone()
            };
            (label, count)
        })
        .collect();

    Some(GithubMonthlyWidget { months })
}

pub struct CfRatingWidget {
    pub rating: i32,
    pub rank: String,
    pub max_rating: i32,
    pub max_rank: String,
    pub contest_count: usize,
}

pub fn cf_rating_widget(p: &UserProfile) -> Option<CfRatingWidget> {
    let cf = p.codeforces.as_ref()?;
    Some(CfRatingWidget {
        rating: cf.user.rating,
        rank: cf.user.rank.clone(),
        max_rating: cf.user.max_rating,
        max_rank: cf.user.max_rank.clone(),
        contest_count: cf.rating_history.len(),
    })
}

pub struct CfStatsWidget {
    pub problems_solved: usize,
    pub contest_count: usize,
    pub contribution: i32,
    pub friend_of_count: i64,
}

pub fn cf_stats_widget(p: &UserProfile) -> Option<CfStatsWidget> {
    let cf = p.codeforces.as_ref()?;

    let mut seen: HashSet<(Option<i32>, String)> = HashSet::new();
    for sub in &cf.submissions {
        if sub.verdict.as_ref() == Some(&Verdict::Ok) {
            seen.insert((sub.problem.contest_id, sub.problem.index.clone()));
        }
    }

    Some(CfStatsWidget {
        problems_solved: seen.len(),
        contest_count: cf.rating_history.len(),
        contribution: cf.user.contribution,
        friend_of_count: cf.user.friend_of_count,
    })
}

pub struct CwRankWidget {
    pub rank_name: String,
    pub rank_color: String,
    pub score: u32,
    pub honor: u32,
    pub leaderboard_position: Option<u32>,
    pub clan: Option<String>,
}

pub fn cw_rank_widget(p: &UserProfile) -> Option<CwRankWidget> {
    let cw = p.codewars.as_ref()?;
    Some(CwRankWidget {
        rank_name: cw.ranks.overall.name.clone(),
        rank_color: cw.ranks.overall.color.clone(),
        score: cw.ranks.overall.score,
        honor: cw.honor,
        leaderboard_position: cw.leaderboard_position,
        clan: cw.clan.clone(),
    })
}

pub struct CwKataWidget {
    pub total_completed: u32,
    pub total_authored: u32,
}

pub fn cw_kata_widget(p: &UserProfile) -> Option<CwKataWidget> {
    let cw = p.codewars.as_ref()?;
    Some(CwKataWidget {
        total_completed: cw.code_challenges.total_completed,
        total_authored: cw.code_challenges.total_authored,
    })
}

pub struct CwLangEntry {
    pub lang: String,
    pub rank_name: String,
    pub rank_color: String,
    pub score: u32,
}

pub struct CwLanguagesWidget {
    pub languages: Vec<CwLangEntry>,
}

pub fn cw_languages_widget(p: &UserProfile) -> Option<CwLanguagesWidget> {
    let cw = p.codewars.as_ref()?;
    if cw.ranks.languages.is_empty() {
        return None;
    }
    let mut languages: Vec<CwLangEntry> = cw
        .ranks
        .languages
        .iter()
        .map(|(lang, entry)| CwLangEntry {
            lang: lang.clone(),
            rank_name: entry.name.clone(),
            rank_color: entry.color.clone(),
            score: entry.score,
        })
        .collect();
    languages.sort_by(|a, b| b.score.cmp(&a.score));
    Some(CwLanguagesWidget { languages })
}

pub struct LcSolvedWidget {
    pub total: u32,
    pub easy: u32,
    pub medium: u32,
    pub hard: u32,
}

pub fn lc_solved_widget(p: &UserProfile) -> Option<LcSolvedWidget> {
    let lc = p.leetcode.as_ref()?;
    Some(LcSolvedWidget {
        total: lc.solved.solved_problem,
        easy: lc.solved.easy_solved,
        medium: lc.solved.medium_solved,
        hard: lc.solved.hard_solved,
    })
}

pub struct SkillEntry {
    pub name: String,
    pub amount: u32,
    pub category: &'static str,
}

pub struct LcSkillsWidget {
    pub skills: Vec<SkillEntry>,
}

pub fn lc_skills_widget(p: &UserProfile) -> Option<LcSkillsWidget> {
    let lc = p.leetcode.as_ref()?;
    let mut all: Vec<SkillEntry> = lc
        .skills_advanced
        .iter()
        .map(|s| SkillEntry {
            name: s.name.clone(),
            amount: s.amount,
            category: "adv",
        })
        .chain(lc.skills_intermediate.iter().map(|s| SkillEntry {
            name: s.name.clone(),
            amount: s.amount,
            category: "int",
        }))
        .chain(lc.skills_fundamental.iter().map(|s| SkillEntry {
            name: s.name.clone(),
            amount: s.amount,
            category: "fun",
        }))
        .collect();
    if all.is_empty() {
        return None;
    }
    all.sort_by(|a, b| b.amount.cmp(&a.amount));
    all.truncate(15);
    Some(LcSkillsWidget { skills: all })
}

pub struct LcLangEntry {
    pub name: String,
    pub solved: u32,
}

pub struct LcLanguagesWidget {
    pub languages: Vec<LcLangEntry>,
}

pub fn lc_languages_widget(p: &UserProfile) -> Option<LcLanguagesWidget> {
    let lc = p.leetcode.as_ref()?;
    if lc.languages.is_empty() {
        return None;
    }
    let mut languages: Vec<LcLangEntry> = lc
        .languages
        .iter()
        .map(|l| LcLangEntry {
            name: l.name.clone(),
            solved: l.solved_amount,
        })
        .collect();
    languages.sort_by(|a, b| b.solved.cmp(&a.solved));
    Some(LcLanguagesWidget { languages })
}

pub struct BadgeEntry {
    pub name: String,
    pub date: String,
}

pub struct LcBadgesWidget {
    pub badges: Vec<BadgeEntry>,
    pub total: u32,
}

pub fn lc_badges_widget(p: &UserProfile) -> Option<LcBadgesWidget> {
    let lc = p.leetcode.as_ref()?;
    let total = lc.badges.badges_count;
    if lc.badges.badges.is_empty() {
        return None;
    }
    let badges = lc
        .badges
        .badges
        .iter()
        .map(|b| BadgeEntry {
            name: b.display_name.clone(),
            date: b.creation_date.clone(),
        })
        .collect();
    Some(LcBadgesWidget { badges, total })
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
