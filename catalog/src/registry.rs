use readme_stuff_aggregator::models::UserProfile;
use readme_stuff_aggregator::widgets::*;
use readme_stuff_config::{Credential, Requirement};
use readme_stuff_draw::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WidgetGroup {
    GitHub,
    Codeforces,
    Codewars,
    LeetCode,
    Combined,
}

pub struct WidgetSpec {
    pub id: &'static str,
    pub label: &'static str,
    pub group: WidgetGroup,
    pub requires: Requirement,
    pub size: (u32, u32),
    pub render: fn(&UserProfile, Theme) -> Option<String>,
}

macro_rules! widget {
    ($id:literal, $label:literal, $group:expr, $requires:expr, $size:expr, $builder:expr, $render:expr) => {
        WidgetSpec {
            id: $id,
            label: $label,
            group: $group,
            requires: $requires,
            size: $size,
            render: |p, theme| $builder(p).map(|w| $render(&w, theme)),
        }
    };
}

const GITHUB: Requirement = Requirement::All(&[Credential::GitHubToken, Credential::GitHubLogin]);
const CODEFORCES: Requirement = Requirement::All(&[Credential::CodeforcesHandle]);
const CODEWARS: Requirement = Requirement::All(&[Credential::CodewarsUsername]);
const LEETCODE: Requirement = Requirement::All(&[Credential::LeetcodeUsername]);
const ANY_COMPETITIVE: Requirement = Requirement::AnyOf(&[
    Credential::CodeforcesHandle,
    Credential::CodewarsUsername,
    Credential::LeetcodeUsername,
]);

static WIDGETS: &[WidgetSpec] = &[
    widget!(
        "github-stats",
        "GitHub Stats",
        WidgetGroup::GitHub,
        GITHUB,
        sizes::GITHUB_STATS,
        github_stats_widget,
        render_github_stats
    ),
    widget!(
        "github-repos",
        "GitHub Repos",
        WidgetGroup::GitHub,
        GITHUB,
        sizes::GITHUB_REPOS,
        github_repos_widget,
        render_github_repos
    ),
    widget!(
        "github-contributions",
        "GitHub Contributions",
        WidgetGroup::GitHub,
        GITHUB,
        sizes::GITHUB_CONTRIBUTIONS,
        github_contributions_widget,
        render_github_contributions
    ),
    widget!(
        "github-social",
        "GitHub Social",
        WidgetGroup::GitHub,
        GITHUB,
        sizes::GITHUB_SOCIAL,
        github_social_widget,
        render_github_social
    ),
    widget!(
        "github-heatmap",
        "GitHub Weekday Heatmap",
        WidgetGroup::GitHub,
        GITHUB,
        sizes::GITHUB_HEATMAP,
        github_heatmap_widget,
        render_github_heatmap
    ),
    widget!(
        "github-monthly",
        "GitHub Monthly Activity",
        WidgetGroup::GitHub,
        GITHUB,
        sizes::GITHUB_MONTHLY,
        github_monthly_widget,
        render_github_monthly
    ),
    widget!(
        "streak",
        "GitHub Contribution Streak",
        WidgetGroup::GitHub,
        GITHUB,
        sizes::STREAK,
        streak_widget,
        render_streak
    ),
    WidgetSpec {
        id: "langs",
        label: "Top Languages",
        group: WidgetGroup::GitHub,
        requires: GITHUB,
        size: sizes::LANGS,
        render: |p, theme| langs_widget(p, 6).map(|w| render_langs(&w, theme)),
    },
    widget!(
        "cf-rating",
        "Codeforces Rating",
        WidgetGroup::Codeforces,
        CODEFORCES,
        sizes::CF_RATING,
        cf_rating_widget,
        render_cf_rating
    ),
    widget!(
        "cf-stats",
        "Codeforces Stats",
        WidgetGroup::Codeforces,
        CODEFORCES,
        sizes::CF_STATS,
        cf_stats_widget,
        render_cf_stats
    ),
    widget!(
        "cw-rank",
        "Codewars Rank",
        WidgetGroup::Codewars,
        CODEWARS,
        sizes::CW_RANK,
        cw_rank_widget,
        render_cw_rank
    ),
    widget!(
        "cw-kata",
        "Codewars Kata",
        WidgetGroup::Codewars,
        CODEWARS,
        sizes::CW_KATA,
        cw_kata_widget,
        render_cw_kata
    ),
    widget!(
        "cw-languages",
        "Codewars by Language",
        WidgetGroup::Codewars,
        CODEWARS,
        sizes::CW_LANGUAGES,
        cw_languages_widget,
        render_cw_languages
    ),
    widget!(
        "lc-solved",
        "LeetCode Solved",
        WidgetGroup::LeetCode,
        LEETCODE,
        sizes::LC_SOLVED,
        lc_solved_widget,
        render_lc_solved
    ),
    widget!(
        "lc-skills",
        "LeetCode Top Skills",
        WidgetGroup::LeetCode,
        LEETCODE,
        sizes::LC_SKILLS,
        lc_skills_widget,
        render_lc_skills
    ),
    widget!(
        "lc-languages",
        "LeetCode Languages",
        WidgetGroup::LeetCode,
        LEETCODE,
        sizes::LC_LANGUAGES,
        lc_languages_widget,
        render_lc_languages
    ),
    widget!(
        "lc-badges",
        "LeetCode Badges",
        WidgetGroup::LeetCode,
        LEETCODE,
        sizes::LC_BADGES,
        lc_badges_widget,
        render_lc_badges
    ),
    widget!(
        "competitive",
        "Competitive Programming",
        WidgetGroup::Combined,
        ANY_COMPETITIVE,
        sizes::COMPETITIVE,
        competitive_widget,
        render_competitive
    ),
    widget!(
        "github-visitors",
        "GitHub Traffic",
        WidgetGroup::GitHub,
        GITHUB,
        sizes::GITHUB_VISITORS,
        github_visitors_widget,
        render_github_visitors
    ),
    widget!(
        "github-engagement",
        "GitHub Engagement",
        WidgetGroup::GitHub,
        GITHUB,
        sizes::GITHUB_ENGAGEMENT,
        github_engagement_widget,
        render_github_engagement
    ),
    widget!(
        "github-commit-streak",
        "Commit Streak",
        WidgetGroup::GitHub,
        GITHUB,
        sizes::GITHUB_COMMIT_STREAK,
        commit_streak_widget,
        render_github_commit_streak
    ),
];

pub fn all_widgets() -> &'static [WidgetSpec] {
    WIDGETS
}

pub fn find(id: &str) -> Option<&'static WidgetSpec> {
    WIDGETS.iter().find(|w| w.id == id)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashSet;

    #[test]
    fn every_widget_has_a_unique_id() {
        let mut seen = HashSet::new();
        for w in all_widgets() {
            assert!(seen.insert(w.id), "duplicate widget id: {}", w.id);
        }
    }

    #[test]
    fn every_widget_has_a_non_zero_size() {
        for w in all_widgets() {
            assert!(w.size.0 > 0 && w.size.1 > 0, "{} has zero size {:?}", w.id, w.size);
        }
    }

    #[test]
    fn find_looks_up_by_id() {
        assert!(find("github-stats").is_some());
        assert!(find("does-not-exist").is_none());
    }
}
