use crate::github_client::{GitHubClient, Result};
use chrono::{NaiveDate, Utc};
use serde::Deserialize;
use serde_json::json;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::task::JoinSet;

#[derive(Debug, Clone)]
#[derive(Default)]
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
        Ok(compute_stats(dates, Utc::now().date_naive()))
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
        let mut set: JoinSet<HashMap<NaiveDate, u64>> = JoinSet::new();

        for repo in repos {
            let client = self.client.clone();
            let login = login.to_owned();
            let sem = sem.clone();
            set.spawn(async move {
                let Ok(_permit) = sem.acquire_owned().await else {
                    return HashMap::new();
                };
                fetch_repo_dates(&client, &login, &repo)
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

async fn fetch_repo_dates(
    client: &GitHubClient,
    login: &str,
    repo: &str,
) -> Result<HashMap<NaiveDate, u64>> {
    let mut map: HashMap<NaiveDate, u64> = HashMap::new();
    let mut page = 1u32;

    loop {
        let path = format!("/repos/{login}/{repo}/commits?author={login}&per_page=100&page={page}");

        let commits: Vec<RestCommit> = match client.rest_get(&path).await {
            Ok(c) => c,
            Err(_) => break,
        };

        let done = commits.len() < 100;
        for c in commits {
            if let Some(date_str) = c.commit.author.date.as_deref()
                && let Ok(dt) = chrono::DateTime::parse_from_rfc3339(date_str) {
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

pub fn compute_stats(dates: HashMap<NaiveDate, u64>, today: NaiveDate) -> CommitStreakStats {
    if dates.is_empty() {
        return CommitStreakStats::default();
    }

    let total_commits: u64 = dates.values().sum();
    let min_date = *dates.keys().min().unwrap();

    let mut run = 0u32;
    let mut run_start: Option<NaiveDate> = None;
    let mut run_end: Option<NaiveDate> = None;

    let mut longest = 0u32;
    let mut longest_start: Option<NaiveDate> = None;
    let mut longest_end: Option<NaiveDate> = None;
    let mut days_with_commits = 0u32;

    let mut cur = min_date;
    while cur <= today {
        let count = dates.get(&cur).copied().unwrap_or(0);

        if count > 0 {
            days_with_commits += 1;
            let continues = run_end
                .map(|last| cur.signed_duration_since(last).num_days() == 1)
                .unwrap_or(false);

            if continues {
                run += 1;
            } else {
                run = 1;
                run_start = Some(cur);
            }
            run_end = Some(cur);

            if run > longest {
                longest = run;
                longest_start = run_start;
                longest_end = run_end;
            }
        } else {
            run = 0;
            run_start = None;
            run_end = None;
        }

        match cur.succ_opt() {
            Some(d) => cur = d,
            None => break,
        }
    }

    let (current_streak, current_streak_start) = calc_current_streak(&dates, today);

    CommitStreakStats {
        total_commits,
        days_with_commits,
        current_streak,
        current_streak_start,
        longest_streak: longest,
        longest_streak_start: longest_start,
        longest_streak_end: longest_end,
    }
}

fn calc_current_streak(
    dates: &HashMap<NaiveDate, u64>,
    today: NaiveDate,
) -> (u32, Option<NaiveDate>) {
    let anchor = if dates.get(&today).copied().unwrap_or(0) > 0 {
        today
    } else if let Some(y) = today.pred_opt() {
        if dates.get(&y).copied().unwrap_or(0) > 0 {
            y
        } else {
            return (0, None);
        }
    } else {
        return (0, None);
    };

    let mut streak = 0u32;
    let mut streak_start: Option<NaiveDate> = None;
    let mut cur = anchor;

    loop {
        if dates.get(&cur).copied().unwrap_or(0) > 0 {
            streak += 1;
            streak_start = Some(cur);
            match cur.pred_opt() {
                Some(d) => cur = d,
                None => break,
            }
        } else {
            break;
        }
    }

    (streak, streak_start)
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
        let today = NaiveDate::from_ymd_opt(2024, 1, 3).unwrap();
        let d = dates(&[("2024-01-01", 3), ("2024-01-02", 5), ("2024-01-03", 2)]);
        let s = compute_stats(d, today);
        assert_eq!(s.longest_streak, 3);
        assert_eq!(s.current_streak, 3);
        assert_eq!(s.total_commits, 10);
    }

    #[test]
    fn streak_broken_by_gap() {
        let today = NaiveDate::from_ymd_opt(2024, 1, 5).unwrap();
        let d = dates(&[
            ("2024-01-01", 5),
            ("2024-01-02", 4),
            ("2024-01-04", 3),
            ("2024-01-05", 2),
        ]);
        let s = compute_stats(d, today);
        assert_eq!(s.longest_streak, 2);
        assert_eq!(s.current_streak, 2);
        assert_eq!(s.days_with_commits, 4);
    }

    #[test]
    fn current_streak_uses_yesterday_when_today_empty() {
        let today = NaiveDate::from_ymd_opt(2024, 6, 10).unwrap();
        let d = dates(&[("2024-06-08", 1), ("2024-06-09", 2)]);
        let s = compute_stats(d, today);
        assert_eq!(s.current_streak, 2);
        assert_eq!(s.current_streak_start, NaiveDate::from_ymd_opt(2024, 6, 8));
    }

    #[test]
    fn no_commits() {
        let today = NaiveDate::from_ymd_opt(2024, 1, 1).unwrap();
        let s = compute_stats(HashMap::new(), today);
        assert_eq!(s.total_commits, 0);
        assert_eq!(s.longest_streak, 0);
        assert_eq!(s.current_streak, 0);
    }

    #[test]
    fn many_commits_same_day_count_once_for_streak() {
        let today = NaiveDate::from_ymd_opt(2024, 3, 1).unwrap();
        let d = dates(&[("2024-03-01", 50)]);
        let s = compute_stats(d, today);
        assert_eq!(s.total_commits, 50);
        assert_eq!(s.longest_streak, 1);
        assert_eq!(s.days_with_commits, 1);
    }

    #[tokio::test]
    async fn live_commit_streak_andezion() {
        let Ok(client) = GitHubClient::from_env() else {
            eprintln!("GITHUB_TOKEN not set — skipping live test");
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
