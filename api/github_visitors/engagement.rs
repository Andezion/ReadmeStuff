use crate::github_client::{GitHubClient, GitHubError};
use crate::github_visitors::models::{EngagementSummary, RepoEngagement};
use chrono::{DateTime, Utc};
use serde::Deserialize;
use serde_json::json;

const RECENT_STARGAZERS_PER_REPO: usize = 5;
const TOP_REPOS_FOR_STARGAZER_DETAIL: usize = 5;

#[derive(Deserialize)]
struct EngagementListRoot {
    user: EngagementListUser,
}

#[derive(Deserialize)]
struct EngagementListUser {
    repositories: EngagementListConn,
}

#[derive(Deserialize)]
struct EngagementListConn {
    #[serde(rename = "pageInfo")]
    page_info: PageInfo,
    nodes: Vec<RepoEngagementNode>,
}

#[derive(Deserialize)]
struct PageInfo {
    #[serde(rename = "hasNextPage")]
    has_next_page: bool,
    #[serde(rename = "endCursor")]
    end_cursor: Option<String>,
}

#[derive(Deserialize)]
struct RepoEngagementNode {
    name: String,
    #[serde(rename = "stargazerCount")]
    stargazer_count: u64,
    #[serde(rename = "forkCount")]
    fork_count: u64,
    watchers: TotalCount,
}

#[derive(Deserialize)]
struct TotalCount {
    #[serde(rename = "totalCount")]
    total_count: u64,
}

const ENGAGEMENT_LIST_QUERY: &str = r#"
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
      nodes {
        name
        stargazerCount
        forkCount
        watchers { totalCount }
      }
    }
  }
}
"#;

#[derive(Deserialize)]
struct StargazersRoot {
    repository: StargazersRepo,
}

#[derive(Deserialize)]
struct StargazersRepo {
    stargazers: StargazersConn,
}

#[derive(Deserialize)]
struct StargazersConn {
    edges: Vec<StargazerEdge>,
}

#[derive(Deserialize)]
struct StargazerEdge {
    #[serde(rename = "starredAt")]
    starred_at: DateTime<Utc>,
    node: StargazerNode,
}

#[derive(Deserialize)]
struct StargazerNode {
    login: String,
}

const RECENT_STARGAZERS_QUERY: &str = r#"
query($owner: String!, $name: String!, $n: Int!) {
  repository(owner: $owner, name: $name) {
    stargazers(last: $n, orderBy: { field: STARRED_AT, direction: ASC }) {
      edges { starredAt node { login } }
    }
  }
}
"#;

pub struct EngagementFetcher {
    client: GitHubClient,
}

impl EngagementFetcher {
    pub fn new(client: GitHubClient) -> Self {
        Self { client }
    }

    pub async fn fetch_engagement(&self, login: &str) -> Result<EngagementSummary, GitHubError> {
        let mut repos = self.enumerate_repo_engagement(login).await?;
        repos.sort_by_key(|r| std::cmp::Reverse(r.stargazer_count));

        let detail_targets: Vec<String> = repos
            .iter()
            .filter(|r| r.stargazer_count > 0)
            .take(TOP_REPOS_FOR_STARGAZER_DETAIL)
            .map(|r| r.repo.clone())
            .collect();

        let mut recent: Vec<(DateTime<Utc>, String, String)> = Vec::new();
        for repo_name in &detail_targets {
            match self
                .fetch_recent_stargazers(login, repo_name, RECENT_STARGAZERS_PER_REPO)
                .await
            {
                Ok(names) => {
                    if let Some(r) = repos.iter_mut().find(|r| &r.repo == repo_name) {
                        r.recent_stargazers = names.iter().map(|(_, l)| l.clone()).collect();
                    }
                    for (ts, login) in names {
                        recent.push((ts, login, repo_name.clone()));
                    }
                }
                Err(GitHubError::Http(ref e))
                    if e.status().map(|s| s.as_u16() == 403).unwrap_or(false) =>
                {
                    tracing::debug!("no access to stargazers for {repo_name} - skipping");
                }
                Err(e) => return Err(e),
            }
        }
        recent.sort_by_key(|(ts, _, _)| std::cmp::Reverse(*ts));
        recent.truncate(20);

        let total_stars = repos.iter().map(|r| r.stargazer_count).sum();
        let total_forks = repos.iter().map(|r| r.fork_count).sum();
        let total_watchers = repos.iter().map(|r| r.watcher_count).sum();

        Ok(EngagementSummary {
            generated_at: Utc::now(),
            total_stars,
            total_forks,
            total_watchers,
            repositories: repos,
            recent_stargazers: recent
                .into_iter()
                .map(|(_, login, repo)| (login, repo))
                .collect(),
        })
    }

    async fn enumerate_repo_engagement(
        &self,
        login: &str,
    ) -> Result<Vec<RepoEngagement>, GitHubError> {
        let mut repos = Vec::new();
        let mut cursor: Option<String> = None;

        loop {
            let data: EngagementListRoot = self
                .client
                .graphql(
                    ENGAGEMENT_LIST_QUERY,
                    json!({ "login": login, "after": cursor }),
                )
                .await?;

            let conn = data.user.repositories;
            repos.extend(conn.nodes.into_iter().map(|n| RepoEngagement {
                repo: format!("{login}/{}", n.name),
                stargazer_count: n.stargazer_count,
                fork_count: n.fork_count,
                watcher_count: n.watchers.total_count,
                recent_stargazers: vec![],
            }));

            if conn.page_info.has_next_page {
                cursor = conn.page_info.end_cursor;
            } else {
                break;
            }
        }

        Ok(repos)
    }

    async fn fetch_recent_stargazers(
        &self,
        owner: &str,
        repo_full_name: &str,
        n: usize,
    ) -> Result<Vec<(DateTime<Utc>, String)>, GitHubError> {
        let name = repo_full_name.rsplit('/').next().unwrap_or(repo_full_name);
        let data: StargazersRoot = self
            .client
            .graphql(
                RECENT_STARGAZERS_QUERY,
                json!({ "owner": owner, "name": name, "n": n as i64 }),
            )
            .await?;

        Ok(data
            .repository
            .stargazers
            .edges
            .into_iter()
            .rev()
            .map(|e| (e.starred_at, e.node.login))
            .collect())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn live_engagement_andezion() {
        let Ok(client) = GitHubClient::from_env() else {
            eprintln!("GITHUB_TOKEN not set - skipping live test");
            return;
        };
        let fetcher = EngagementFetcher::new(client);
        let summary = fetcher.fetch_engagement("Andezion").await.unwrap();

        println!(
            "stars={} forks={} watchers={}",
            summary.total_stars, summary.total_forks, summary.total_watchers
        );
        for (login, repo) in &summary.recent_stargazers {
            println!("  {login} starred {repo}");
        }
    }
}
