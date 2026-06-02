use crate::github_client::{GitHubClient, Result};
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LanguageEntry {
    pub name: String,
    pub color: Option<String>,
    pub bytes: u64,
    pub percentage: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RepoLanguageStats {
    pub repo_name: String,
    pub is_fork: bool,
    pub is_archived: bool,
    pub languages: Vec<LanguageEntry>,
    pub dominant_language: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LanguageStats {
    pub name: String,
    pub color: Option<String>,
    pub total_bytes: u64,
    pub percentage: f64,
    pub repo_count: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AggregatedLangStats {
    pub languages: Vec<LanguageStats>,
    pub most_used: Option<String>,
    pub total_bytes: u64,
    pub repos: Vec<RepoLanguageStats>,
}

#[derive(Debug, Clone)]
pub struct LangQueryOptions {
    pub exclude_forks: bool,
    pub exclude_archived: bool,
}

impl Default for LangQueryOptions {
    fn default() -> Self {
        Self {
            exclude_forks: true,
            exclude_archived: true,
        }
    }
}

#[derive(Deserialize)]
struct UserRepos {
    user: UserReposInner,
}

#[derive(Deserialize)]
struct UserReposInner {
    repositories: RepoConnection,
}

#[derive(Deserialize)]
struct RepoConnection {
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
    #[serde(rename = "isFork")]
    is_fork: bool,
    #[serde(rename = "isArchived")]
    is_archived: bool,
    languages: LangConnection,
}

#[derive(Deserialize)]
struct LangConnection {
    #[serde(rename = "totalSize")]
    total_size: u64,
    edges: Vec<LangEdge>,
}

#[derive(Deserialize)]
struct LangEdge {
    size: u64,
    node: LangNode,
}

#[derive(Deserialize)]
struct LangNode {
    name: String,
    color: Option<String>,
}

const LANG_QUERY: &str = r#"
query($login: String!, $after: String) {
  user(login: $login) {
    repositories(
      first: 100
      after: $after
      ownerAffiliations: OWNER
      orderBy: { field: UPDATED_AT, direction: DESC }
    ) {
      pageInfo {
        hasNextPage
        endCursor
      }
      nodes {
        name
        isFork
        isArchived
        languages(first: 20, orderBy: { field: SIZE, direction: DESC }) {
          totalSize
          edges {
            size
            node { name color }
          }
        }
      }
    }
  }
}
"#;

pub struct GitHubLangsApi {
    client: GitHubClient,
}

impl GitHubLangsApi {
    pub fn new(client: GitHubClient) -> Self {
        Self { client }
    }

    pub async fn fetch_lang_stats(
        &self,
        login: &str,
        opts: &LangQueryOptions,
    ) -> Result<AggregatedLangStats> {
        let repos = self.paginate_repos(login).await?;
        Ok(aggregate(repos, opts))
    }

    async fn paginate_repos(&self, login: &str) -> Result<Vec<RepoNode>> {
        let mut all = Vec::new();
        let mut cursor: Option<String> = None;

        loop {
            let data: UserRepos = self
                .client
                .graphql(LANG_QUERY, json!({ "login": login, "after": cursor }))
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

fn aggregate(repos: Vec<RepoNode>, opts: &LangQueryOptions) -> AggregatedLangStats {
    let mut global: HashMap<String, (u64, Option<String>, u32)> = HashMap::new();
    let mut repo_stats: Vec<RepoLanguageStats> = Vec::new();
    let mut total_bytes: u64 = 0;

    for repo in repos {
        if opts.exclude_forks && repo.is_fork {
            continue;
        }
        if opts.exclude_archived && repo.is_archived {
            continue;
        }
        if repo.languages.total_size == 0 {
            continue;
        }

        let repo_total = repo.languages.total_size;

        let mut entries: Vec<LanguageEntry> = repo
            .languages
            .edges
            .iter()
            .map(|e| LanguageEntry {
                name: e.node.name.clone(),
                color: e.node.color.clone(),
                bytes: e.size,
                percentage: (e.size as f64 / repo_total as f64) * 100.0,
            })
            .collect();

        entries.sort_by(|a, b| b.bytes.cmp(&a.bytes));
        let dominant = entries.first().map(|e| e.name.clone());

        for e in &repo.languages.edges {
            let entry = global
                .entry(e.node.name.clone())
                .or_insert((0, e.node.color.clone(), 0));
            entry.0 += e.size;
            entry.2 += 1;
            total_bytes += e.size;
        }

        repo_stats.push(RepoLanguageStats {
            repo_name: repo.name,
            is_fork: repo.is_fork,
            is_archived: repo.is_archived,
            languages: entries,
            dominant_language: dominant,
        });
    }

    let mut languages: Vec<LanguageStats> = global
        .into_iter()
        .map(|(name, (bytes, color, repo_count))| LanguageStats {
            name,
            color,
            total_bytes: bytes,
            percentage: if total_bytes > 0 {
                (bytes as f64 / total_bytes as f64) * 100.0
            } else {
                0.0
            },
            repo_count,
        })
        .collect();

    languages.sort_by(|a, b| b.total_bytes.cmp(&a.total_bytes));
    repo_stats.sort_by_key(|r| std::cmp::Reverse(r.languages.iter().map(|l| l.bytes).sum::<u64>()));

    let most_used = languages.first().map(|l| l.name.clone());

    AggregatedLangStats {
        most_used,
        total_bytes,
        languages,
        repos: repo_stats,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_repo(name: &str, is_fork: bool, is_archived: bool, langs: &[(&str, u64)]) -> RepoNode {
        let edges = langs
            .iter()
            .map(|(n, s)| LangEdge {
                size: *s,
                node: LangNode {
                    name: n.to_string(),
                    color: None,
                },
            })
            .collect::<Vec<_>>();
        let total = langs.iter().map(|(_, s)| s).sum();
        RepoNode {
            name: name.to_string(),
            is_fork,
            is_archived,
            languages: LangConnection {
                total_size: total,
                edges,
            },
        }
    }

    #[test]
    fn aggregate_excludes_forks_and_archived() {
        let repos = vec![
            make_repo("my-repo", false, false, &[("Rust", 5_000)]),
            make_repo("a-fork", true, false, &[("Rust", 3_000)]),
            make_repo("archived", false, true, &[("C", 2_000)]),
        ];
        let stats = aggregate(repos, &LangQueryOptions::default());
        assert_eq!(stats.total_bytes, 5_000);
        assert_eq!(stats.repos.len(), 1);
        assert_eq!(stats.most_used, Some("Rust".to_string()));
    }

    #[test]
    fn percentage_sums_to_100() {
        let repos = vec![make_repo(
            "mixed",
            false,
            false,
            &[("Rust", 600), ("C", 300), ("Python", 100)],
        )];
        let stats = aggregate(repos, &LangQueryOptions::default());
        let sum: f64 = stats.languages.iter().map(|l| l.percentage).sum();
        assert!(
            (sum - 100.0).abs() < 0.01,
            "percentages must sum to 100, got {sum}"
        );
    }

    #[test]
    fn dominant_language_is_largest() {
        let repos = vec![make_repo("r", false, false, &[("C", 100), ("Rust", 900)])];
        let stats = aggregate(repos, &LangQueryOptions::default());
        assert_eq!(stats.repos[0].dominant_language, Some("Rust".to_string()));
    }

    #[tokio::test]
    async fn live_langs_andezion() {
        let Ok(client) = GitHubClient::from_env() else {
            eprintln!("GITHUB_TOKEN not set — skipping live test");
            return;
        };
        let api = GitHubLangsApi::new(client);
        let stats = api
            .fetch_lang_stats("Andezion", &LangQueryOptions::default())
            .await
            .unwrap();

        println!("{stats:#?}");
        assert!(stats.total_bytes > 0);
        println!(
            "Top language: {:?}  total bytes: {}  repos analysed: {}",
            stats.most_used,
            stats.total_bytes,
            stats.repos.len()
        );
        let sum: f64 = stats.languages.iter().map(|l| l.percentage).sum();
        assert!((sum - 100.0).abs() < 0.5, "percentages sum={sum}");
    }
}
