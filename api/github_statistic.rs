use crate::github_client::{GitHubClient, Result};
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProfileMetadata {
    pub login: String,
    pub name: Option<String>,
    pub bio: Option<String>,
    pub company: Option<String>,
    pub location: Option<String>,
    pub website_url: Option<String>,
    pub avatar_url: String,
    pub created_at: String,
    pub followers: u32,
    pub following: u32,
    pub public_repos: u32,
    pub public_gists: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrganizationInfo {
    pub login: String,
    pub name: Option<String>,
    pub avatar_url: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContributionSummary {
    pub total_commits: u32,
    pub total_pull_requests: u32,
    pub total_issues: u32,
    pub total_pull_request_reviews: u32,
    pub repos_contributed_to: u32,
    pub restricted_contributions: u32,
    /// Commits this year grouped by the repository's primary language.
    pub commits_by_language: HashMap<String, u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RepositoryStats {
    pub name: String,
    pub is_fork: bool,
    pub is_archived: bool,
    pub is_private: bool,
    pub stars: u32,
    pub forks: u32,
    pub watchers: u32,
    pub primary_language: Option<String>,
    pub description: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AggregatedRepoStats {
    pub total_repos: u32,
    pub total_stars: u64,
    pub total_forks: u64,
    pub total_watchers: u64,
    pub top_repos: Vec<RepositoryStats>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProfileStats {
    pub metadata: ProfileMetadata,
    pub organizations: Vec<OrganizationInfo>,
    pub contributions: ContributionSummary,
    pub repos: AggregatedRepoStats,
    pub rank: ProfileRank,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProfileRank {
    pub grade: String,
    pub score: f64,
    pub percentile: f64,
}

#[derive(Deserialize)]
struct ProfileRoot {
    user: GqlProfile,
}

#[derive(Deserialize)]
struct GqlProfile {
    login: String,
    name: Option<String>,
    bio: Option<String>,
    company: Option<String>,
    location: Option<String>,
    #[serde(rename = "websiteUrl")]
    website_url: Option<String>,
    #[serde(rename = "avatarUrl")]
    avatar_url: String,
    #[serde(rename = "createdAt")]
    created_at: String,
    followers: TotalCount,
    following: TotalCount,
    repositories: TotalCount,
    gists: TotalCount,
    organizations: OrgConnection,
    #[serde(rename = "contributionsCollection")]
    contributions_collection: GqlContribs,
}

#[derive(Deserialize)]
struct TotalCount {
    #[serde(rename = "totalCount")]
    total_count: u32,
}

#[derive(Deserialize)]
struct OrgConnection {
    nodes: Vec<GqlOrg>,
}

#[derive(Deserialize)]
struct GqlOrg {
    login: String,
    name: Option<String>,
    #[serde(rename = "avatarUrl")]
    avatar_url: String,
}

#[derive(Deserialize)]
struct GqlContribs {
    #[serde(rename = "totalCommitContributions")]
    total_commit_contributions: u32,
    #[serde(rename = "totalPullRequestContributions")]
    total_pull_request_contributions: u32,
    #[serde(rename = "totalIssueContributions")]
    total_issue_contributions: u32,
    #[serde(rename = "totalPullRequestReviewContributions")]
    total_pull_request_review_contributions: u32,
    #[serde(rename = "totalRepositoriesWithContributedCommits")]
    total_repositories_with_contributed_commits: u32,
    #[serde(rename = "restrictedContributionsCount")]
    restricted_contributions_count: u32,
    #[serde(rename = "commitContributionsByRepository", default)]
    commit_contributions_by_repository: Vec<GqlCommitByRepo>,
}

#[derive(Deserialize)]
struct GqlCommitByRepo {
    repository: GqlCommitRepoNode,
    contributions: TotalCount,
}

#[derive(Deserialize)]
struct GqlCommitRepoNode {
    #[serde(rename = "primaryLanguage")]
    primary_language: Option<LangName>,
}

#[derive(Deserialize)]
struct RepoStatsRoot {
    user: RepoStatsUser,
}

#[derive(Deserialize)]
struct RepoStatsUser {
    repositories: RepoStatsConn,
}

#[derive(Deserialize)]
struct RepoStatsConn {
    #[serde(rename = "pageInfo")]
    page_info: PageInfo,
    nodes: Vec<RepoStatsNode>,
}

#[derive(Deserialize)]
struct PageInfo {
    #[serde(rename = "hasNextPage")]
    has_next_page: bool,
    #[serde(rename = "endCursor")]
    end_cursor: Option<String>,
}

#[derive(Deserialize)]
struct RepoStatsNode {
    name: String,
    #[serde(rename = "isFork")]
    is_fork: bool,
    #[serde(rename = "isArchived")]
    is_archived: bool,
    #[serde(rename = "isPrivate")]
    is_private: bool,
    #[serde(rename = "stargazerCount")]
    stargazer_count: u32,
    #[serde(rename = "forkCount")]
    fork_count: u32,
    watchers: TotalCount,
    #[serde(rename = "primaryLanguage")]
    primary_language: Option<LangName>,
    description: Option<String>,
}

#[derive(Deserialize)]
struct LangName {
    name: String,
}

const PROFILE_QUERY: &str = r#"
query($login: String!) {
  user(login: $login) {
    login
    name
    bio
    company
    location
    websiteUrl
    avatarUrl
    createdAt
    followers     { totalCount }
    following     { totalCount }
    repositories(ownerAffiliations: OWNER) { totalCount }
    gists         { totalCount }
    organizations(first: 20) {
      nodes { login name avatarUrl }
    }
    contributionsCollection {
      totalCommitContributions
      totalPullRequestContributions
      totalIssueContributions
      totalPullRequestReviewContributions
      totalRepositoriesWithContributedCommits
      restrictedContributionsCount
      commitContributionsByRepository(maxRepositories: 100) {
        repository { primaryLanguage { name } }
        contributions { totalCount }
      }
    }
  }
}
"#;

const REPO_STATS_QUERY: &str = r#"
query($login: String!, $after: String) {
  user(login: $login) {
    repositories(
      first: 100
      after: $after
      ownerAffiliations: OWNER
      orderBy: { field: STARGAZERS, direction: DESC }
    ) {
      pageInfo { hasNextPage endCursor }
      nodes {
        name
        isFork
        isArchived
        isPrivate
        stargazerCount
        forkCount
        watchers { totalCount }
        primaryLanguage { name }
        description
      }
    }
  }
}
"#;

pub struct GitHubStatisticApi {
    client: GitHubClient,
}

impl GitHubStatisticApi {
    pub fn new(client: GitHubClient) -> Self {
        Self { client }
    }

    pub async fn fetch_profile_stats(&self, login: &str) -> Result<ProfileStats> {
        let (profile, repo_nodes) =
            tokio::try_join!(self.fetch_profile(login), self.paginate_repos(login))?;

        let metadata = ProfileMetadata {
            login: profile.login,
            name: profile.name,
            bio: profile.bio,
            company: profile.company,
            location: profile.location,
            website_url: profile.website_url,
            avatar_url: profile.avatar_url,
            created_at: profile.created_at,
            followers: profile.followers.total_count,
            following: profile.following.total_count,
            public_repos: profile.repositories.total_count,
            public_gists: profile.gists.total_count,
        };

        let organizations = profile
            .organizations
            .nodes
            .into_iter()
            .map(|o| OrganizationInfo {
                login: o.login,
                name: o.name,
                avatar_url: o.avatar_url,
            })
            .collect();

        let gc = profile.contributions_collection;
        let mut commits_by_language: HashMap<String, u32> = HashMap::new();
        for entry in &gc.commit_contributions_by_repository {
            if let Some(lang) = &entry.repository.primary_language {
                *commits_by_language.entry(lang.name.clone()).or_insert(0) +=
                    entry.contributions.total_count;
            }
        }
        let contributions = ContributionSummary {
            total_commits: gc.total_commit_contributions,
            total_pull_requests: gc.total_pull_request_contributions,
            total_issues: gc.total_issue_contributions,
            total_pull_request_reviews: gc.total_pull_request_review_contributions,
            repos_contributed_to: gc.total_repositories_with_contributed_commits,
            restricted_contributions: gc.restricted_contributions_count,
            commits_by_language,
        };

        let repos = build_repo_stats(repo_nodes);
        let rank = calculate_rank(&metadata, &contributions, &repos);

        Ok(ProfileStats {
            metadata,
            organizations,
            contributions,
            repos,
            rank,
        })
    }

    async fn fetch_profile(&self, login: &str) -> Result<GqlProfile> {
        let root: ProfileRoot = self
            .client
            .graphql(PROFILE_QUERY, json!({ "login": login }))
            .await?;
        Ok(root.user)
    }

    async fn paginate_repos(&self, login: &str) -> Result<Vec<RepoStatsNode>> {
        let mut all = Vec::new();
        let mut cursor: Option<String> = None;

        loop {
            let data: RepoStatsRoot = self
                .client
                .graphql(REPO_STATS_QUERY, json!({ "login": login, "after": cursor }))
                .await?;

            let conn = data.user.repositories;
            all.extend(conn.nodes);

            if conn.page_info.has_next_page {
                cursor = conn.page_info.end_cursor;
            } else {
                break;
            }
        }

        Ok(all)
    }
}

fn build_repo_stats(nodes: Vec<RepoStatsNode>) -> AggregatedRepoStats {
    let total_repos = nodes.len() as u32;
    let total_stars: u64 = nodes.iter().map(|r| r.stargazer_count as u64).sum();
    let total_forks: u64 = nodes.iter().map(|r| r.fork_count as u64).sum();
    let total_watchers: u64 = nodes.iter().map(|r| r.watchers.total_count as u64).sum();

    let mut list: Vec<RepositoryStats> = nodes
        .into_iter()
        .map(|r| RepositoryStats {
            name: r.name,
            is_fork: r.is_fork,
            is_archived: r.is_archived,
            is_private: r.is_private,
            stars: r.stargazer_count,
            forks: r.fork_count,
            watchers: r.watchers.total_count,
            primary_language: r.primary_language.map(|l| l.name),
            description: r.description,
        })
        .collect();

    list.sort_by_key(|b| std::cmp::Reverse(b.stars));

    AggregatedRepoStats {
        total_repos,
        total_stars,
        total_forks,
        total_watchers,
        top_repos: list.into_iter().take(10).collect(),
    }
}

pub fn calculate_rank(
    meta: &ProfileMetadata,
    contrib: &ContributionSummary,
    repos: &AggregatedRepoStats,
) -> ProfileRank {
    const COMMITS_W: f64 = 1.65;
    const PRS_W: f64 = 0.35;
    const ISSUES_W: f64 = 0.25;
    const REVIEWS_W: f64 = 0.20;
    const STARS_W: f64 = 0.45;
    const FOLLOWERS_W: f64 = 0.45;
    const CONTRIB_REPOS_W: f64 = 0.60;
    const NORMALIZER: f64 = 2_413.0;

    let score = (contrib.total_commits as f64) * COMMITS_W
        + (contrib.total_pull_requests as f64) * PRS_W
        + (contrib.total_issues as f64) * ISSUES_W
        + (contrib.total_pull_request_reviews as f64) * REVIEWS_W
        + (repos.total_stars as f64) * STARS_W
        + (meta.followers as f64) * FOLLOWERS_W
        + (contrib.repos_contributed_to as f64) * CONTRIB_REPOS_W;

    let percentile = 1.0 - (-score / NORMALIZER).exp();

    let grade = match percentile {
        p if p >= 0.99 => "S+",
        p if p >= 0.875 => "S",
        p if p >= 0.75 => "A++",
        p if p >= 0.625 => "A+",
        p if p >= 0.50 => "A",
        p if p >= 0.375 => "B+",
        p if p >= 0.25 => "B",
        p if p >= 0.125 => "C",
        _ => "C-",
    }
    .to_string();

    ProfileRank {
        grade,
        score,
        percentile,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn low_meta() -> ProfileMetadata {
        ProfileMetadata {
            login: "test".into(),
            name: None,
            bio: None,
            company: None,
            location: None,
            website_url: None,
            avatar_url: String::new(),
            created_at: String::new(),
            followers: 5,
            following: 0,
            public_repos: 3,
            public_gists: 0,
        }
    }

    fn low_contrib() -> ContributionSummary {
        ContributionSummary {
            total_commits: 50,
            total_pull_requests: 2,
            total_issues: 1,
            total_pull_request_reviews: 0,
            repos_contributed_to: 2,
            restricted_contributions: 0,
            commits_by_language: HashMap::new(),
        }
    }

    fn empty_repos() -> AggregatedRepoStats {
        AggregatedRepoStats {
            total_repos: 3,
            total_stars: 0,
            total_forks: 0,
            total_watchers: 0,
            top_repos: vec![],
        }
    }

    #[test]
    fn rank_higher_activity_scores_higher() {
        let mut meta_high = low_meta();
        meta_high.followers = 5_000;

        let mut contrib_high = low_contrib();
        contrib_high.total_commits = 10_000;
        contrib_high.total_pull_requests = 500;

        let repos_high = AggregatedRepoStats {
            total_stars: 8_000,
            ..empty_repos()
        };

        let low = calculate_rank(&low_meta(), &low_contrib(), &empty_repos());
        let high = calculate_rank(&meta_high, &contrib_high, &repos_high);

        assert!(high.score > low.score);
        assert!(high.percentile > low.percentile);

        println!("Low:  {} ({:.1}%)", low.grade, low.percentile * 100.0);
        println!("High: {} ({:.1}%)", high.grade, high.percentile * 100.0);
    }

    #[test]
    fn rank_percentile_in_unit_range() {
        let rank = calculate_rank(&low_meta(), &low_contrib(), &empty_repos());
        assert!((0.0..=1.0).contains(&rank.percentile));
    }

    #[test]
    fn grade_list_is_consistent() {
        let zero_meta = ProfileMetadata {
            followers: 0,
            public_repos: 0,
            ..low_meta()
        };
        let zero_contrib = ContributionSummary {
            total_commits: 0,
            total_pull_requests: 0,
            total_issues: 0,
            total_pull_request_reviews: 0,
            repos_contributed_to: 0,
            restricted_contributions: 0,
            commits_by_language: HashMap::new(),
        };
        let zero_rank = calculate_rank(&zero_meta, &zero_contrib, &empty_repos());
        assert_eq!(zero_rank.grade, "C-");

        let mut stellar_meta = low_meta();
        stellar_meta.followers = 100_000;
        let mut stellar_contrib = low_contrib();
        stellar_contrib.total_commits = 100_000;
        stellar_contrib.total_pull_requests = 10_000;
        let stellar_repos = AggregatedRepoStats {
            total_stars: 50_000,
            ..empty_repos()
        };
        let stellar = calculate_rank(&stellar_meta, &stellar_contrib, &stellar_repos);
        assert_eq!(stellar.grade, "S+");
    }

    #[tokio::test]
    async fn live_profile_stats_andezion() {
        let Ok(client) = GitHubClient::from_env() else {
            eprintln!("GITHUB_TOKEN not set - skipping live test");
            return;
        };
        let api = GitHubStatisticApi::new(client);
        let stats = api.fetch_profile_stats("Andezion").await.unwrap();

        println!("{stats:#?}");
        assert_eq!(stats.metadata.login, "Andezion");
        assert!(stats.repos.total_repos > 0);
        println!(
            "Rank: {}  score={:.1}  percentile={:.1}%",
            stats.rank.grade,
            stats.rank.score,
            stats.rank.percentile * 100.0
        );
        println!(
            "Stars: {}  Forks: {}",
            stats.repos.total_stars, stats.repos.total_forks
        );
    }
}
