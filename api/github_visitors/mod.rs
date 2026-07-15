pub mod analytics;
pub mod engagement;
pub mod fetcher;
pub mod filter;
pub mod models;
pub mod storage;

pub use models::*;
pub use storage::{InMemoryStorage, JsonStorage, SqliteStorage, StorageError, VisitorStorage};

use crate::github_client::GitHubClient;
use analytics::{
    compute_analytics, daily_active_visitors, repo_popularity_ranking, unique_visitor_stats,
};
use chrono::{NaiveDate, Utc};
use engagement::EngagementFetcher;
use fetcher::TrafficFetcher;
use filter::{FilterConfig, VisitorFilter};
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::Mutex;

pub enum StorageKind {
    InMemory,
    Sqlite { path: PathBuf },
    Json { dir: PathBuf },
}

#[derive(Debug, thiserror::Error)]
pub enum VisitorsError {
    #[error("GitHub API error: {0}")]
    GitHub(#[from] crate::github_client::GitHubError),
    #[error("Storage error: {0}")]
    Storage(#[from] StorageError),
}

pub type Result<T> = std::result::Result<T, VisitorsError>;

pub struct GithubVisitorsService {
    fetcher: TrafficFetcher,
    engagement_fetcher: EngagementFetcher,
    filter: Arc<Mutex<VisitorFilter>>,
    storage: Arc<dyn VisitorStorage>,
}

impl GithubVisitorsService {
    pub async fn new(
        client: GitHubClient,
        kind: StorageKind,
        filter_config: FilterConfig,
    ) -> Result<Self> {
        let storage: Arc<dyn VisitorStorage> = match kind {
            StorageKind::InMemory => Arc::new(InMemoryStorage::new()),
            StorageKind::Sqlite { path } => Arc::new(
                SqliteStorage::open(path)
                    .await
                    .map_err(VisitorsError::Storage)?,
            ),
            StorageKind::Json { dir } => Arc::new(JsonStorage::new(dir)),
        };

        Ok(Self {
            fetcher: TrafficFetcher::new(client.clone()),
            engagement_fetcher: EngagementFetcher::new(client),
            filter: Arc::new(Mutex::new(VisitorFilter::new(filter_config))),
            storage,
        })
    }

    pub async fn refresh_from_github(&self, login: &str) -> Result<Vec<TrafficSnapshot>> {
        let snapshots = self.fetcher.fetch_all_repo_traffic(login).await?;
        for snap in &snapshots {
            self.storage
                .save_snapshot(snap)
                .await
                .map_err(VisitorsError::Storage)?;
        }
        tracing::info!(
            login,
            count = snapshots.len(),
            "refreshed traffic snapshots from GitHub"
        );
        Ok(snapshots)
    }

    pub async fn fetch_repo_snapshot(&self, owner: &str, repo: &str) -> Result<TrafficSnapshot> {
        Ok(self.fetcher.fetch_repo_snapshot(owner, repo).await?)
    }

    pub async fn engagement(&self, login: &str) -> Result<EngagementSummary> {
        Ok(self.engagement_fetcher.fetch_engagement(login).await?)
    }

    pub async fn record_visit(
        &self,
        target: VisitTarget,
        user_agent: Option<&str>,
        hashed_identity: Option<&str>,
    ) -> Result<VisitorEvent> {
        let target_key = target.name();
        let now = Utc::now();

        let filter_result = {
            let mut f = self.filter.lock().await;
            f.evaluate(user_agent, hashed_identity, &target_key, now)
        };

        let event = VisitorEvent::new(
            target,
            user_agent.map(|s| s.to_string()),
            hashed_identity.map(|s| s.to_string()),
            EventSource::CustomPixel,
            filter_result,
        );

        self.storage
            .record_event(&event)
            .await
            .map_err(VisitorsError::Storage)?;
        Ok(event)
    }

    pub async fn analytics(&self, username: &str) -> Result<VisitorAnalytics> {
        let events = self
            .storage
            .get_events(&EventQuery::default())
            .await
            .map_err(VisitorsError::Storage)?;
        let snapshots = self
            .storage
            .get_snapshots(None, 10_000)
            .await
            .map_err(VisitorsError::Storage)?;

        Ok(compute_analytics(username, &events, &snapshots))
    }

    pub async fn unique_visitor_stats(&self) -> Result<UniqueVisitorStats> {
        let events = self
            .storage
            .get_events(&EventQuery::default())
            .await
            .map_err(VisitorsError::Storage)?;
        Ok(unique_visitor_stats(&events))
    }

    pub async fn repo_ranking(&self) -> Result<Vec<(String, u64, u64)>> {
        let snapshots = self
            .storage
            .get_snapshots(None, 10_000)
            .await
            .map_err(VisitorsError::Storage)?;
        Ok(repo_popularity_ranking(&snapshots))
    }

    pub async fn daily_active_visitors(
        &self,
    ) -> Result<std::collections::BTreeMap<NaiveDate, u64>> {
        let events = self
            .storage
            .get_events(&EventQuery::default())
            .await
            .map_err(VisitorsError::Storage)?;
        Ok(daily_active_visitors(&events))
    }

    pub async fn export_json(&self, path: &std::path::Path) -> Result<()> {
        self.storage
            .export_json(path)
            .await
            .map_err(VisitorsError::Storage)?;
        Ok(())
    }

    pub async fn gc_filter(&self) {
        self.filter.lock().await.gc();
    }
    pub fn tracking_pixel_url(base_url: &str, username: &str) -> String {
        format!("{}/track?u={}", base_url.trim_end_matches('/'), username)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use filter::FilterConfig;

    async fn make_service() -> GithubVisitorsService {
        let client = GitHubClient::new("dummy-token-for-unit-tests").unwrap();
        GithubVisitorsService::new(client, StorageKind::InMemory, FilterConfig::default())
            .await
            .unwrap()
    }

    #[tokio::test]
    async fn record_and_retrieve_visit() {
        let svc = make_service().await;
        let target = VisitTarget::Profile {
            username: "Andezion".into(),
        };

        let event = svc
            .record_visit(
                target.clone(),
                Some("Mozilla/5.0 (X11; Linux x86_64) Chrome/120.0"),
                Some("hashed-ip-abc"),
            )
            .await
            .unwrap();

        assert!(event.filter_result.passed);

        let stats = svc.unique_visitor_stats().await.unwrap();
        assert_eq!(stats.counted_events, 1);
        assert_eq!(stats.distinct_identities, 1);
    }

    #[tokio::test]
    async fn bot_visit_filtered() {
        let svc = make_service().await;
        let target = VisitTarget::Profile {
            username: "Andezion".into(),
        };

        let event = svc
            .record_visit(target, Some("Googlebot/2.1"), Some("bot-ip"))
            .await
            .unwrap();

        assert!(!event.filter_result.passed);

        let stats = svc.unique_visitor_stats().await.unwrap();
        assert_eq!(stats.total_events, 1);
        assert_eq!(stats.counted_events, 0);
    }

    #[tokio::test]
    async fn second_visit_within_window_deduplicated() {
        let svc = make_service().await;
        let target = VisitTarget::Profile {
            username: "Andezion".into(),
        };

        let e1 = svc
            .record_visit(target.clone(), Some("Mozilla/5.0"), Some("id-xyz"))
            .await
            .unwrap();
        let e2 = svc
            .record_visit(target, Some("Mozilla/5.0"), Some("id-xyz"))
            .await
            .unwrap();

        assert!(e1.filter_result.passed);
        assert!(!e2.filter_result.passed);
        assert!(
            e2.filter_result
                .reasons
                .contains(&FilterReason::DeduplicatedByWindow)
        );
    }

    #[tokio::test]
    async fn camo_proxy_request_filtered() {
        let svc = make_service().await;
        let event = svc
            .record_visit(
                VisitTarget::Profile { username: "Andezion".into() },
                Some("camo-rs (https://raw.githubusercontent.com; +https://github.com/atmos/camo) abc"),
                None,
            )
            .await
            .unwrap();

        assert!(!event.filter_result.passed);
        assert!(
            event
                .filter_result
                .reasons
                .contains(&FilterReason::GithubCamoProxy)
        );
    }

    #[tokio::test]
    async fn analytics_on_empty_data() {
        let svc = make_service().await;
        let a = svc.analytics("Andezion").await.unwrap();
        assert_eq!(a.username, "Andezion");
        assert!(a.repositories.is_empty());
        assert_eq!(a.filter_summary.total_raw, 0);
    }

    #[tokio::test]
    async fn tracking_pixel_url_format() {
        let url = GithubVisitorsService::tracking_pixel_url("https://my-server.com", "Andezion");
        assert_eq!(url, "https://my-server.com/track?u=Andezion");
    }

    #[tokio::test]
    async fn spam_refresh_simulation() {
        let config = FilterConfig {
            max_requests_per_window: 20,
            rate_window_secs: 60,
            dedup_window_secs: 1,
            filter_bots: false,
            filter_camo_proxy: false,
            filter_github_actions: false,
            filter_empty_user_agents: false,
            owner_ip_hash: None,
        };
        let client = GitHubClient::new("dummy").unwrap();
        let svc = GithubVisitorsService::new(client, StorageKind::InMemory, config)
            .await
            .unwrap();

        let mut passed = 0u32;
        let mut rate_limited = 0u32;
        for i in 0..25u32 {
            let id = format!("id-{i}");
            let target = VisitTarget::Profile {
                username: "Andezion".into(),
            };
            let event = svc
                .record_visit(target, Some("Mozilla/5.0"), Some(&id))
                .await
                .unwrap();
            if event.filter_result.passed {
                passed += 1;
            } else if event
                .filter_result
                .reasons
                .contains(&FilterReason::RateLimitExceeded)
            {
                rate_limited += 1;
            }
        }

        assert_eq!(passed, 20, "exactly 20 should pass before rate limit");
        assert_eq!(rate_limited, 5, "the remaining 5 should be rate-limited");
    }

    #[tokio::test]
    async fn live_refresh_from_github_andezion() {
        let Ok(client) = GitHubClient::from_env() else {
            eprintln!("GITHUB_TOKEN not set - skipping live test");
            return;
        };
        let svc =
            GithubVisitorsService::new(client, StorageKind::InMemory, FilterConfig::default())
                .await
                .unwrap();

        let snapshots = svc.refresh_from_github("Andezion").await.unwrap();
        println!("Refreshed {} snapshots", snapshots.len());

        let analytics = svc.analytics("Andezion").await.unwrap();
        println!("{analytics:#?}");
        println!("Top repos: {:?}", analytics.top_repos_by_views);
    }

    #[tokio::test]
    async fn live_sqlite_persistence() {
        let Ok(client) = GitHubClient::from_env() else {
            eprintln!("GITHUB_TOKEN not set - skipping live test");
            return;
        };
        let db_path = std::env::temp_dir().join("andezion_visitors_test.db");
        let svc = GithubVisitorsService::new(
            client,
            StorageKind::Sqlite {
                path: db_path.clone(),
            },
            FilterConfig::default(),
        )
        .await
        .unwrap();

        let snapshots = svc.refresh_from_github("Andezion").await.unwrap();
        println!("Stored {} snapshots in SQLite", snapshots.len());

        let ranking = svc.repo_ranking().await.unwrap();
        for (repo, views, unique) in &ranking {
            println!("  {repo}: {views} views ({unique} unique)");
        }

        tokio::fs::remove_file(&db_path).await.ok();
    }
}
