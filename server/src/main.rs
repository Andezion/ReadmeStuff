mod handlers;

use axum::{Router, routing::get};
use readme_stuff_api::github_client::GitHubClient;
use readme_stuff_api::github_visitors::{GithubVisitorsService, StorageKind, filter::FilterConfig};
use readme_stuff_cache::DashboardCache;
use std::{net::SocketAddr, sync::Arc, time::Duration};

#[derive(Clone)]
pub struct AppState {
    pub cache: Arc<DashboardCache>,
    /// `None` when GITHUB_TOKEN isn't set or storage failed to open - the
    /// /track endpoint still serves the pixel, it just won't record.
    pub visitors: Option<Arc<GithubVisitorsService>>,
}

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .init();

    let ttl_secs = std::env::var("CACHE_TTL_SECS")
        .ok()
        .and_then(|v| v.parse::<u64>().ok())
        .unwrap_or(300);

    let visitors = init_visitors_service().await;

    let state = AppState {
        cache: Arc::new(DashboardCache::new(Duration::from_secs(ttl_secs))),
        visitors,
    };

    let app = Router::new()
        .route("/github.svg", get(handlers::github))
        .route("/streak.svg", get(handlers::streak))
        .route("/langs.svg", get(handlers::langs))
        .route("/competitive.svg", get(handlers::competitive))
        .route("/track", get(handlers::track))
        .with_state(state);

    let addr = "0.0.0.0:3000";
    tracing::info!("listening on {addr}");
    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(
        listener,
        app.into_make_service_with_connect_info::<SocketAddr>(),
    )
    .await
    .unwrap();
}

async fn init_visitors_service() -> Option<Arc<GithubVisitorsService>> {
    let client = match GitHubClient::from_env() {
        Ok(c) => c,
        Err(_) => {
            tracing::warn!("GITHUB_TOKEN not set - /track will serve the pixel but not record visits");
            return None;
        }
    };

    let db_path = std::env::var("VISITORS_DB_PATH").unwrap_or_else(|_| "visitors.db".to_string());
    let filter_config = FilterConfig {
        owner_ip_hash: std::env::var("OWNER_IP_HASH").ok(),
        ..FilterConfig::default()
    };

    match GithubVisitorsService::new(
        client,
        StorageKind::Sqlite {
            path: db_path.into(),
        },
        filter_config,
    )
    .await
    {
        Ok(svc) => Some(Arc::new(svc)),
        Err(e) => {
            tracing::warn!("failed to open visitor tracking storage: {e}");
            None
        }
    }
}
