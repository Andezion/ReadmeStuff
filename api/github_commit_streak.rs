use crate::github_client::{GitHubClient, GitHubError, Result};
use chrono::NaiveDate;
use serde::Deserialize;
use serde_json::json;
use std::collections::HashMap;
use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};
use tokio::task::JoinSet;

const REQUESTS_PER_PAUSE: u64 = 1000;
const PAUSE_DURATION: std::time::Duration = std::time::Duration::from_secs(1);

#[derive(Debug, Clone, Default)]
pub struct CommitStreakStats {
    pub total_commits: u64,
    pub days_with_commits: u32,
    pub current_streak: u32,
    pub current_streak_start: Option<NaiveDate>,
    pub longest_streak: u32,
    pub longest_streak_start: Option<NaiveDate>,
    pub longest_streak_end: Option<NaiveDate>,
}

#[derive(Deserialize)]
struct RepoListRoot {
    user: RepoListUser,
}

#[derive(Deserialize)]
struct RepoListUser {
    repositories: RepoListConn,
}

#[derive(Deserialize)]
struct RepoListConn {
    #[serde(rename = "pageInfo")]
    page_info: PageInfo,
    nodes: Vec<RepoNode>,
}

#[derive(Deserialize)]
struct PageInfo {
    #[serde(rename = "hasNextPage")]
    has_next_page: bool,
    #[serde(rename = "endCursor")]
    end_cursor: Option<String>,
}

#[derive(Deserialize)]
struct RepoNode {
    name: String,
}

#[derive(Deserialize)]
struct RestCommit {
    commit: RestCommitData,
}

#[derive(Deserialize)]
struct RestCommitData {
    author: RestCommitAuthor,
}

#[derive(Deserialize)]
struct RestCommitAuthor {
    date: Option<String>,
}

const REPO_LIST_QUERY: &str = r#"
query($login: String!, $after: String) {
  user(login: $login) {
    repositories(
      first: 100
      after: $after
      ownerAffiliations: OWNER
      isFork: false
      orderBy: { field: PUSHED_AT, direction: DESC }
    ) {
      pageInfo { hasNextPage endCursor }
      nodes { name }
    }
  }
}
"#;

pub struct GitHubCommitStreakApi {
    client: GitHubClient,
}

impl GitHubCommitStreakApi {
    pub fn new(client: GitHubClient) -> Self {
        Self { client }
    }

    pub async fn fetch_stats(&self, login: &str) -> Result<CommitStreakStats> {
        let repos = self.list_repos(login).await?;
        let dates = self.collect_commit_dates(login, repos).await;
        Ok(compute_stats(dates))
    }

    async fn list_repos(&self, login: &str) -> Result<Vec<String>> {
        let mut names = Vec::new();
        let mut cursor: Option<String> = None;

        loop {
            let data: RepoListRoot = self
                .client
                .graphql(REPO_LIST_QUERY, json!({ "login": login, "after": cursor }))
                .await?;

            let conn = data.user.repositories;
            names.extend(conn.nodes.into_iter().map(|n| n.name));

            if conn.page_info.has_next_page {
                cursor = conn.page_info.end_cursor;
            } else {
                break;
            }
        }

        Ok(names)
    }

    async fn collect_commit_dates(
        &self,
        login: &str,
        repos: Vec<String>,
    ) -> HashMap<NaiveDate, u64> {
        let sem = Arc::new(tokio::sync::Semaphore::new(5));
        let request_count = Arc::new(AtomicU64::new(0));
        let mut set: JoinSet<HashMap<NaiveDate, u64>> = JoinSet::new();

        for repo in repos {
            let client = self.client.clone();
            let login = login.to_owned();
            let sem = sem.clone();
            let request_count = request_count.clone();
            set.spawn(async move {
                let Ok(_permit) = sem.acquire_owned().await else {
                    return HashMap::new();
                };
                fetch_repo_dates(&client, &login, &repo, &request_count)
                    .await
                    .unwrap_or_default()
            });
        }

        let mut combined: HashMap<NaiveDate, u64> = HashMap::new();
        while let Some(Ok(map)) = set.join_next().await {
            for (date, count) in map {
                *combined.entry(date).or_insert(0) += count;
            }
        }
        combined
    }
}

const MAX_PAGE_RETRIES: u32 = 5;

async fn fetch_repo_dates(
    client: &GitHubClient,
    login: &str,
    repo: &str,
    request_count: &Arc<AtomicU64>,
) -> Result<HashMap<NaiveDate, u64>> {
    let mut map: HashMap<NaiveDate, u64> = HashMap::new();
    let mut page = 1u32;
    let mut retries = 0u32;

    loop {
        let path = format!("/repos/{login}/{repo}/commits?author={login}&per_page=100&page={page}");

        let commits: Vec<RestCommit> = match client.rest_get(&path).await {
            Ok(c) => c,
            
            Err(GitHubError::RateLimit { .. }) if retries < MAX_PAGE_RETRIES => {
                retries += 1;
                tokio::time::sleep(PAUSE_DURATION * retries).await;
                continue;
            }
            Err(_) => break,
        };
        retries = 0;

        if (request_count.fetch_add(1, Ordering::Relaxed) + 1) % REQUESTS_PER_PAUSE == 0 {
            tokio::time::sleep(PAUSE_DURATION).await;
        }

        let done = commits.len() < 100;
        for c in commits {
            if let Some(date_str) = c.commit.author.date.as_deref()
                && let Ok(dt) = chrono::DateTime::parse_from_rfc3339(date_str)
            {
                *map.entry(dt.date_naive()).or_insert(0) += 1;
            }
        }

        if done {
            break;
        }
        page += 1;
    }

    Ok(map)
}


pub fn compute_stats(dates: HashMap<NaiveDate, u64>) -> CommitStreakStats {
    if dates.is_empty() {
        return CommitStreakStats::default();
    }

    let total_commits: u64 = dates.values().sum();
    let days_with_commits = dates.len() as u32;

    let mut sorted: Vec<NaiveDate> = dates.keys().copied().collect();
    sorted.sort();

    let mut run = 1u32;
    let mut run_start = sorted[0];

    let mut longest = 1u32;
    let mut longest_start = sorted[0];
    let mut longest_end = sorted[0];

    for pair in sorted.windows(2) {
        let (prev, cur) = (pair[0], pair[1]);
        if cur.signed_duration_since(prev).num_days() == 1 {
            run += 1;
        } else {
            run = 1;
            run_start = cur;
        }
        if run > longest {
            longest = run;
            longest_start = run_start;
            longest_end = cur;
        }
    }

    CommitStreakStats {
        total_commits,
        days_with_commits,
        current_streak: run,
        current_streak_start: Some(run_start),
        longest_streak: longest,
        longest_streak_start: Some(longest_start),
        longest_streak_end: Some(longest_end),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn dates(pairs: &[(&str, u64)]) -> HashMap<NaiveDate, u64> {
        pairs
            .iter()
            .map(|(s, c)| (NaiveDate::parse_from_str(s, "%Y-%m-%d").unwrap(), *c))
            .collect()
    }

    #[test]
    fn three_consecutive_days() {
        let d = dates(&[("2024-01-01", 3), ("2024-01-02", 5), ("2024-01-03", 2)]);
        let s = compute_stats(d);
        assert_eq!(s.longest_streak, 3);
        assert_eq!(s.current_streak, 3);
        assert_eq!(s.total_commits, 10);
    }

    #[test]
    fn streak_broken_by_gap() {
        let d = dates(&[
            ("2024-01-01", 5),
            ("2024-01-02", 4),
            ("2024-01-04", 3),
            ("2024-01-05", 2),
        ]);
        let s = compute_stats(d);
        assert_eq!(s.longest_streak, 2);
        assert_eq!(s.current_streak, 2);
        assert_eq!(s.days_with_commits, 4);
    }

    #[test]
    fn current_streak_anchors_on_latest_commit_even_when_dated_in_the_future() {
       
        let d = dates(&[("2027-03-01", 1), ("2027-03-02", 1), ("2027-03-03", 1)]);
        let s = compute_stats(d);
        assert_eq!(s.current_streak, 3);
        assert_eq!(s.current_streak_start, NaiveDate::from_ymd_opt(2027, 3, 1));
    }

    #[test]
    fn no_commits() {
        let s = compute_stats(HashMap::new());
        assert_eq!(s.total_commits, 0);
        assert_eq!(s.longest_streak, 0);
        assert_eq!(s.current_streak, 0);
    }

    #[test]
    fn many_commits_same_day_count_once_for_streak() {
        let d = dates(&[("2024-03-01", 50)]);
        let s = compute_stats(d);
        assert_eq!(s.total_commits, 50);
        assert_eq!(s.longest_streak, 1);
        assert_eq!(s.days_with_commits, 1);
    }

    #[tokio::test]
    async fn live_commit_streak_andezion() {
        let Ok(client) = GitHubClient::from_env() else {
            eprintln!("GITHUB_TOKEN not set - skipping live test");
            return;
        };
        let api = GitHubCommitStreakApi::new(client);
        let stats = api.fetch_stats("Andezion").await.unwrap();

        println!("total_commits       : {}", stats.total_commits);
        println!("days_with_commits   : {}", stats.days_with_commits);
        println!("current_streak      : {} days", stats.current_streak);
        println!("current_streak_start: {:?}", stats.current_streak_start);
        println!("longest_streak      : {} days", stats.longest_streak);
        println!("longest_streak_start: {:?}", stats.longest_streak_start);
        println!("longest_streak_end  : {:?}", stats.longest_streak_end);

        assert!(stats.total_commits > 0 || stats.days_with_commits == 0);
    }
}
