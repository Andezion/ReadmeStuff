use readme_stuff_aggregator::models::UserProfile;
use std::{
    collections::HashMap,
    sync::Arc,
    time::{Duration, Instant},
};
use tokio::sync::RwLock;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct CacheKey {
    pub github_login: String,
    pub cf_handle: String,
    pub cw_username: String,
    pub lc_username: String,
}

impl CacheKey {
    pub fn new(
        github_login: impl Into<String>,
        cf_handle: impl Into<String>,
        cw_username: impl Into<String>,
        lc_username: impl Into<String>,
    ) -> Self {
        Self {
            github_login: github_login.into(),
            cf_handle: cf_handle.into(),
            cw_username: cw_username.into(),
            lc_username: lc_username.into(),
        }
    }
}

struct CachedEntry {
    profile: Arc<UserProfile>,
    fetched_at: Instant,
}

pub struct DashboardCache {
    store: RwLock<HashMap<CacheKey, CachedEntry>>,
    ttl: Duration,
}

impl DashboardCache {
    pub fn new(ttl: Duration) -> Self {
        Self {
            store: RwLock::new(HashMap::new()),
            ttl,
        }
    }

    pub async fn get(&self, key: &CacheKey) -> Option<Arc<UserProfile>> {
        let store = self.store.read().await;
        let entry = store.get(key)?;
        if entry.fetched_at.elapsed() < self.ttl {
            Some(Arc::clone(&entry.profile))
        } else {
            None
        }
    }

    pub async fn set(&self, key: CacheKey, profile: UserProfile) -> Arc<UserProfile> {
        let arc = Arc::new(profile);
        let mut store = self.store.write().await;
        store.insert(
            key,
            CachedEntry {
                profile: Arc::clone(&arc),
                fetched_at: Instant::now(),
            },
        );
        arc
    }
}
