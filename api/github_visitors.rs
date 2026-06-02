use crate::github_client::{GitHubClient, GitHubError, Result};
use serde::{Deserialize, Serialize};
use serde_json::json;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrafficDay {
    pub timestamp: String,
    pub count: u64,
    pub uniques: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrafficViews {
    pub count: u64,
    pub uniques: u64,
    pub views: Vec<TrafficDay>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrafficClones {
    pub count: u64,
    pub uniques: u64,
    pub clones: Vec<TrafficDay>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrafficReferrer {
    pub referrer: String,
    pub count: u64,
    pub uniques: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrafficPath {
    pub path: String,
    pub title: String,
    pub count: u64,
    pub uniques: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RepoTraffic {
    pub repo_name: String,
    pub views: TrafficViews,
    pub clones: TrafficClones,
    pub referrers: Vec<TrafficReferrer>,
    pub top_paths: Vec<TrafficPath>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProfileTraffic {
    pub profile_views: Option<u64>,
    pub total_repo_views: u64,
    pub total_unique_visitors: u64,
    pub total_clones: u64,
    pub total_unique_cloners: u64,
    pub repos: Vec<RepoTraffic>,
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

#[derive(Deserialize)]
struct RestViews {
    count: u64,
    uniques: u64,
    views: Vec<RestDay>,
}

#[derive(Deserialize)]
struct RestClones {
    count: u64,
    uniques: u64,
    clones: Vec<RestDay>,
}

#[derive(Deserialize)]
struct RestDay {
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

pub struct GitHubVisitorsApi {
    client: GitHubClient,
}

impl GitHubVisitorsApi {
    pub fn new(client: GitHubClient) -> Self {
        Self { client }
    }

    pub async fn fetch_profile_traffic(&self, login: &str) -> Result<ProfileTraffic> {
        let names = self.enumerate_repo_names(login).await?;

        let mut repos: Vec<RepoTraffic> = Vec::new();
        let mut total_views = 0u64;
        let mut total_uniques = 0u64;
        let mut total_clones = 0u64;
        let mut total_unique_cloners = 0u64;

        for name in &names {
            match self.fetch_repo_traffic(login, name).await {
                Ok(t) => {
                    total_views += t.views.count;
                    total_uniques += t.views.uniques;
                    total_clones += t.clones.count;
                    total_unique_cloners += t.clones.uniques;
                    repos.push(t);
                }
                Err(GitHubError::Http(ref e))
                    if e.status().map(|s| s.as_u16() == 403).unwrap_or(false) => {}
                Err(e) => return Err(e),
            }
        }

        repos.sort_by_key(|b| std::cmp::Reverse(b.views.count));

        Ok(ProfileTraffic {
            profile_views: None,
            total_repo_views: total_views,
            total_unique_visitors: total_uniques,
            total_clones,
            total_unique_cloners,
            repos,
        })
    }

    pub async fn fetch_repo_traffic(&self, owner: &str, repo: &str) -> Result<RepoTraffic> {
        let path_views = format!("repos/{owner}/{repo}/traffic/views");
        let path_clones = format!("repos/{owner}/{repo}/traffic/clones");
        let path_refs = format!("repos/{owner}/{repo}/traffic/popular/referrers");
        let path_paths = format!("repos/{owner}/{repo}/traffic/popular/paths");

        let (views_raw, clones_raw, refs_raw, paths_raw) = tokio::try_join!(
            self.client.rest_get::<RestViews>(&path_views),
            self.client.rest_get::<RestClones>(&path_clones),
            self.client.rest_get::<Vec<RestReferrer>>(&path_refs),
            self.client.rest_get::<Vec<RestPath>>(&path_paths),
        )?;

        Ok(RepoTraffic {
            repo_name: repo.to_string(),
            views: TrafficViews {
                count: views_raw.count,
                uniques: views_raw.uniques,
                views: views_raw.views.into_iter().map(into_day).collect(),
            },
            clones: TrafficClones {
                count: clones_raw.count,
                uniques: clones_raw.uniques,
                clones: clones_raw.clones.into_iter().map(into_day).collect(),
            },
            referrers: refs_raw
                .into_iter()
                .map(|r| TrafficReferrer {
                    referrer: r.referrer,
                    count: r.count,
                    uniques: r.uniques,
                })
                .collect(),
            top_paths: paths_raw
                .into_iter()
                .map(|p| TrafficPath {
                    path: p.path,
                    title: p.title,
                    count: p.count,
                    uniques: p.uniques,
                })
                .collect(),
        })
    }

    async fn enumerate_repo_names(&self, login: &str) -> Result<Vec<String>> {
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

fn into_day(r: RestDay) -> TrafficDay {
    TrafficDay {
        timestamp: r.timestamp,
        count: r.count,
        uniques: r.uniques,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn live_traffic_andezion() {
        let Ok(client) = GitHubClient::from_env() else {
            eprintln!("GITHUB_TOKEN not set - skipping live test");
            return;
        };
        let api = GitHubVisitorsApi::new(client);
        let traffic = api.fetch_profile_traffic("Andezion").await.unwrap();

        println!("{traffic:#?}");
        println!(
            "14-day totals - views: {}  unique visitors: {}  clones: {}",
            traffic.total_repo_views, traffic.total_unique_visitors, traffic.total_clones,
        );
        assert!(traffic.profile_views.is_none());
    }

    #[tokio::test]
    async fn live_single_repo_traffic_andezion() {
        let Ok(client) = GitHubClient::from_env() else {
            eprintln!("GITHUB_TOKEN not set - skipping live test");
            return;
        };
        let api = GitHubVisitorsApi::new(client);
        match api.fetch_repo_traffic("Andezion", "ReadmeStuff").await {
            Ok(t) => println!("{t:#?}"),
            Err(GitHubError::Http(e)) if e.status().map(|s| s.as_u16() == 403).unwrap_or(false) => {
                eprintln!(
                    "No push access to ReadmeStuff - this is expected with a read-only token"
                );
            }
            Err(e) => panic!("Unexpected error: {e}"),
        }
    }
}
