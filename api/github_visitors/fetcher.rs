use crate::github_client::{GitHubClient, GitHubError};
use crate::github_visitors::models::{
    TrafficClones, TrafficDay, TrafficPath, TrafficReferrer, TrafficSnapshot, TrafficViews,
};
use chrono::DateTime;
use serde::Deserialize;
use serde_json::json;

#[derive(Deserialize)]
struct RestViews {
    count: u64,
    uniques: u64,
    views: Vec<RestDayEntry>,
}

#[derive(Deserialize)]
struct RestClones {
    count: u64,
    uniques: u64,
    clones: Vec<RestDayEntry>,
}

#[derive(Deserialize)]
struct RestDayEntry {
    timestamp: String,
    count: u64,
    uniques: u64,
}

#[derive(Deserialize)]
struct RestReferrer {
    referrer: String,
    count: u64,
    uniques: u64,
}

#[derive(Deserialize)]
struct RestPath {
    path: String,
    title: String,
    count: u64,
    uniques: u64,
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
    nodes: Vec<RepoNameNode>,
}

#[derive(Deserialize)]
struct PageInfo {
    #[serde(rename = "hasNextPage")]
    has_next_page: bool,
    #[serde(rename = "endCursor")]
    end_cursor: Option<String>,
}

#[derive(Deserialize)]
struct RepoNameNode {
    name: String,
}

const REPO_LIST_QUERY: &str = r#"
query($login: String!, $after: String) {
  user(login: $login) {
    repositories(
      first: 100
      after: $after
      ownerAffiliations: OWNER
      privacy: PUBLIC
      orderBy: { field: UPDATED_AT, direction: DESC }
    ) {
      pageInfo { hasNextPage endCursor }
      nodes { name }
    }
  }
}
"#;

pub struct TrafficFetcher {
    client: GitHubClient,
}

impl TrafficFetcher {
    pub fn new(client: GitHubClient) -> Self {
        Self { client }
    }

    pub async fn fetch_all_repo_traffic(
        &self,
        login: &str,
    ) -> Result<Vec<TrafficSnapshot>, GitHubError> {
        let names = self.enumerate_repo_names(login).await?;
        let mut snapshots = Vec::new();

        for name in &names {
            match self.fetch_repo_snapshot(login, name).await {
                Ok(snap) => snapshots.push(snap),
                Err(GitHubError::Http(ref e))
                    if e.status().map(|s| s.as_u16() == 403).unwrap_or(false) =>
                {
                    tracing::debug!("no push access to {login}/{name} - skipping traffic fetch");
                }
                Err(e) => return Err(e),
            }
        }

        snapshots.sort_by_key(|b| std::cmp::Reverse(b.views.count));
        Ok(snapshots)
    }

    pub async fn fetch_repo_snapshot(
        &self,
        owner: &str,
        repo: &str,
    ) -> Result<TrafficSnapshot, GitHubError> {
        let p_views = format!("repos/{owner}/{repo}/traffic/views");
        let p_clones = format!("repos/{owner}/{repo}/traffic/clones");
        let p_refs = format!("repos/{owner}/{repo}/traffic/popular/referrers");
        let p_paths = format!("repos/{owner}/{repo}/traffic/popular/paths");

        let (raw_views, raw_clones, raw_refs, raw_paths) = tokio::try_join!(
            self.client.rest_get::<RestViews>(&p_views),
            self.client.rest_get::<RestClones>(&p_clones),
            self.client.rest_get::<Vec<RestReferrer>>(&p_refs),
            self.client.rest_get::<Vec<RestPath>>(&p_paths),
        )?;

        let views = TrafficViews {
            count: raw_views.count,
            uniques: raw_views.uniques,
            days: raw_views.views.iter().filter_map(parse_day).collect(),
        };

        let clones = TrafficClones {
            count: raw_clones.count,
            uniques: raw_clones.uniques,
            days: raw_clones.clones.iter().filter_map(parse_day).collect(),
        };

        let referrers = raw_refs
            .into_iter()
            .map(|r| TrafficReferrer {
                referrer: r.referrer,
                count: r.count,
                uniques: r.uniques,
            })
            .collect();

        let top_paths = raw_paths
            .into_iter()
            .map(|p| TrafficPath {
                path: p.path,
                title: p.title,
                count: p.count,
                uniques: p.uniques,
            })
            .collect();

        Ok(TrafficSnapshot::new(
            format!("{owner}/{repo}"),
            views,
            clones,
            referrers,
            top_paths,
        ))
    }

    async fn enumerate_repo_names(&self, login: &str) -> Result<Vec<String>, GitHubError> {
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
}

fn parse_day(entry: &RestDayEntry) -> Option<TrafficDay> {
    let dt = DateTime::parse_from_rfc3339(&entry.timestamp).ok()?;
    Some(TrafficDay {
        date: dt.date_naive(),
        count: entry.count,
        uniques: entry.uniques,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::NaiveDate;

    #[test]
    fn parse_day_valid_timestamp() {
        let entry = RestDayEntry {
            timestamp: "2024-05-01T00:00:00Z".into(),
            count: 10,
            uniques: 7,
        };
        let day = parse_day(&entry).unwrap();
        assert_eq!(day.date, NaiveDate::from_ymd_opt(2024, 5, 1).unwrap());
        assert_eq!(day.count, 10);
        assert_eq!(day.uniques, 7);
    }

    #[test]
    fn parse_day_invalid_timestamp_returns_none() {
        let entry = RestDayEntry {
            timestamp: "not-a-date".into(),
            count: 1,
            uniques: 1,
        };
        assert!(parse_day(&entry).is_none());
    }

    #[tokio::test]
    async fn live_all_repo_traffic_andezion() {
        let Ok(client) = GitHubClient::from_env() else {
            eprintln!("GITHUB_TOKEN not set - skipping live test");
            return;
        };
        let fetcher = TrafficFetcher::new(client);
        let snapshots = fetcher.fetch_all_repo_traffic("Andezion").await.unwrap();

        println!("Fetched {} repo traffic snapshots", snapshots.len());
        for s in &snapshots {
            println!(
                "  {} - views: {}  unique: {}",
                s.repo, s.views.count, s.views.uniques
            );
        }
    }

    #[tokio::test]
    async fn live_single_repo_snapshot_andezion() {
        let Ok(client) = GitHubClient::from_env() else {
            eprintln!("GITHUB_TOKEN not set - skipping live test");
            return;
        };
        let fetcher = TrafficFetcher::new(client);
        match fetcher.fetch_repo_snapshot("Andezion", "ReadmeStuff").await {
            Ok(snap) => {
                println!("{snap:#?}");
                assert!(!snap.repo.is_empty());
            }
            Err(GitHubError::Http(e)) if e.status().map(|s| s.as_u16() == 403).unwrap_or(false) => {
                eprintln!("Token lacks push access to ReadmeStuff - expected for read-only tokens");
            }
            Err(e) => panic!("Unexpected error: {e}"),
        }
    }
}
