mod handlers;

use axum::{Router, routing::get};
use readme_stuff_cache::DashboardCache;
use std::{sync::Arc, time::Duration};

#[derive(Clone)]
pub struct AppState {
    pub cache: Arc<DashboardCache>,
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

    let state = AppState {
        cache: Arc::new(DashboardCache::new(Duration::from_secs(ttl_secs))),
    };

    let app = Router::new()
        .route("/github.svg", get(handlers::github))
        .route("/streak.svg", get(handlers::streak))
        .route("/langs.svg", get(handlers::langs))
        .route("/competitive.svg", get(handlers::competitive))
        .with_state(state);

    let addr = "0.0.0.0:3000";
    tracing::info!("listening on {addr}");
    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
