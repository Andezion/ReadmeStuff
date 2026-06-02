use crate::github_visitors::models::{EventQuery, TrafficSnapshot, VisitorEvent};
use async_trait::async_trait;
use chrono::Utc;
use serde_json;
use sqlx::sqlite::{SqlitePool, SqlitePoolOptions};
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tokio::sync::RwLock;

pub type StorageResult<T> = std::result::Result<T, StorageError>;

#[derive(Debug, thiserror::Error)]
pub enum StorageError {
    #[error("SQLite error: {0}")]
    Sqlite(#[from] sqlx::Error),

    #[error("JSON serialization error: {0}")]
    Json(#[from] serde_json::Error),

    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Backend not initialised — call `init()` first")]
    NotInitialised,
}

#[async_trait]
pub trait VisitorStorage: Send + Sync {
    async fn record_event(&self, event: &VisitorEvent) -> StorageResult<()>;
    async fn get_events(&self, query: &EventQuery) -> StorageResult<Vec<VisitorEvent>>;
    async fn save_snapshot(&self, snapshot: &TrafficSnapshot) -> StorageResult<()>;

    async fn get_snapshots(
        &self,
        repo: Option<&str>,
        limit: u32,
    ) -> StorageResult<Vec<TrafficSnapshot>>;

    async fn export_json(&self, path: &Path) -> StorageResult<()>;
}

#[derive(Debug, Default, Clone)]
pub struct InMemoryStorage {
    events: Arc<RwLock<Vec<VisitorEvent>>>,
    snapshots: Arc<RwLock<Vec<TrafficSnapshot>>>,
}

impl InMemoryStorage {
    pub fn new() -> Self {
        Self::default()
    }
}

#[async_trait]
impl VisitorStorage for InMemoryStorage {
    async fn record_event(&self, event: &VisitorEvent) -> StorageResult<()> {
        self.events.write().await.push(event.clone());
        Ok(())
    }

    async fn get_events(&self, query: &EventQuery) -> StorageResult<Vec<VisitorEvent>> {
        let events = self.events.read().await;
        let filtered: Vec<VisitorEvent> = events
            .iter()
            .filter(|e| {
                if let Some(ref t) = query.target
                    && &e.target != t {
                        return false;
                    }
                if let Some(src) = query.source
                    && e.source != src {
                        return false;
                    }
                if let Some(from) = query.from
                    && e.timestamp < from {
                        return false;
                    }
                if let Some(to) = query.to
                    && e.timestamp > to {
                        return false;
                    }
                if query.passed_only && !e.filter_result.passed {
                    return false;
                }
                true
            })
            .take(query.limit.unwrap_or(u64::MAX) as usize).cloned()
            .collect();
        Ok(filtered)
    }

    async fn save_snapshot(&self, snapshot: &TrafficSnapshot) -> StorageResult<()> {
        self.snapshots.write().await.push(snapshot.clone());
        Ok(())
    }

    async fn get_snapshots(
        &self,
        repo: Option<&str>,
        limit: u32,
    ) -> StorageResult<Vec<TrafficSnapshot>> {
        let snaps = self.snapshots.read().await;
        let result: Vec<TrafficSnapshot> = snaps
            .iter()
            .filter(|s| repo.map(|r| s.repo == r).unwrap_or(true))
            .rev()
            .take(limit as usize)
            .cloned()
            .collect();
        Ok(result)
    }

    async fn export_json(&self, path: &Path) -> StorageResult<()> {
        use tokio::io::AsyncWriteExt;
        let events = self.events.read().await;
        let snaps = self.snapshots.read().await;

        let mut file = tokio::fs::File::create(path).await?;
        for e in events.iter() {
            let line = serde_json::to_string(e)? + "\n";
            file.write_all(line.as_bytes()).await?;
        }
        for s in snaps.iter() {
            let line = serde_json::to_string(s)? + "\n";
            file.write_all(line.as_bytes()).await?;
        }
        Ok(())
    }
}

pub struct SqliteStorage {
    pool: SqlitePool,
}

impl SqliteStorage {
    pub async fn open(path: impl AsRef<Path>) -> StorageResult<Self> {
        let url = format!("sqlite://{}?mode=rwc", path.as_ref().display());
        let pool = SqlitePoolOptions::new()
            .max_connections(4)
            .connect(&url)
            .await?;
        let storage = Self { pool };
        storage.migrate().await?;
        Ok(storage)
    }

    pub async fn open_in_memory() -> StorageResult<Self> {
        let pool = SqlitePoolOptions::new()
            .max_connections(1)
            .connect("sqlite::memory:")
            .await?;
        let storage = Self { pool };
        storage.migrate().await?;
        Ok(storage)
    }

    async fn migrate(&self) -> StorageResult<()> {
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS visitor_events (
                id              TEXT PRIMARY KEY,
                timestamp       TEXT NOT NULL,
                target_type     TEXT NOT NULL,
                target_name     TEXT NOT NULL,
                hashed_identity TEXT,
                user_agent      TEXT,
                source          TEXT NOT NULL,
                passed_filter   INTEGER NOT NULL DEFAULT 1,
                filter_data     TEXT NOT NULL DEFAULT '{}',
                created_at      TEXT NOT NULL
            );
            CREATE INDEX IF NOT EXISTS idx_ve_ts   ON visitor_events(timestamp);
            CREATE INDEX IF NOT EXISTS idx_ve_tgt  ON visitor_events(target_name, timestamp);

            CREATE TABLE IF NOT EXISTS traffic_snapshots (
                id          TEXT PRIMARY KEY,
                captured_at TEXT NOT NULL,
                repo        TEXT NOT NULL,
                views_json  TEXT NOT NULL,
                clones_json TEXT NOT NULL,
                refs_json   TEXT NOT NULL,
                paths_json  TEXT NOT NULL,
                created_at  TEXT NOT NULL
            );
            CREATE INDEX IF NOT EXISTS idx_ts_repo ON traffic_snapshots(repo, captured_at);
            "#,
        )
        .execute(&self.pool)
        .await?;
        Ok(())
    }
}

#[async_trait]
impl VisitorStorage for SqliteStorage {
    async fn record_event(&self, event: &VisitorEvent) -> StorageResult<()> {
        let id = event.id.to_string();
        let ts = event.timestamp.to_rfc3339();
        let target_type = event.target.kind();
        let target_name = event.target.name();
        let source = event.source.to_string();
        let passed = event.filter_result.passed as i32;
        let filter_data = serde_json::to_string(&event.filter_result)?;
        let now = Utc::now().to_rfc3339();

        sqlx::query(
            "INSERT OR IGNORE INTO visitor_events
             (id, timestamp, target_type, target_name, hashed_identity, user_agent,
              source, passed_filter, filter_data, created_at)
             VALUES (?,?,?,?,?,?,?,?,?,?)",
        )
        .bind(&id)
        .bind(&ts)
        .bind(target_type)
        .bind(&target_name)
        .bind(&event.hashed_identity)
        .bind(&event.user_agent)
        .bind(&source)
        .bind(passed)
        .bind(&filter_data)
        .bind(&now)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    async fn get_events(&self, query: &EventQuery) -> StorageResult<Vec<VisitorEvent>> {
        let mut sql = "SELECT id, timestamp, target_type, target_name, \
                             hashed_identity, user_agent, source, filter_data \
                       FROM visitor_events WHERE 1=1"
            .to_string();

        if let Some(ref target) = query.target {
            let _ = target;
            sql += " AND target_name = ?";
        }
        if let Some(src) = query.source {
            let _ = src;
            sql += " AND source = ?";
        }
        if query.from.is_some() {
            sql += " AND timestamp >= ?";
        }
        if query.to.is_some() {
            sql += " AND timestamp <= ?";
        }
        if query.passed_only {
            sql += " AND passed_filter = 1";
        }
        sql += " ORDER BY timestamp DESC";
        if let Some(lim) = query.limit {
            sql += &format!(" LIMIT {lim}");
        }

        let rows = sqlx::query(&sql).fetch_all(&self.pool).await?;
        let events: Vec<VisitorEvent> = rows
            .into_iter()
            .filter_map(|row| deserialize_event_row(&row).ok())
            .collect();

        Ok(events)
    }

    async fn save_snapshot(&self, snapshot: &TrafficSnapshot) -> StorageResult<()> {
        let id = snapshot.id.to_string();
        let captured_at = snapshot.captured_at.to_rfc3339();
        let views_json = serde_json::to_string(&snapshot.views)?;
        let clones_json = serde_json::to_string(&snapshot.clones)?;
        let refs_json = serde_json::to_string(&snapshot.referrers)?;
        let paths_json = serde_json::to_string(&snapshot.top_paths)?;
        let now = Utc::now().to_rfc3339();

        sqlx::query(
            "INSERT OR IGNORE INTO traffic_snapshots
             (id, captured_at, repo, views_json, clones_json, refs_json, paths_json, created_at)
             VALUES (?,?,?,?,?,?,?,?)",
        )
        .bind(&id)
        .bind(&captured_at)
        .bind(&snapshot.repo)
        .bind(&views_json)
        .bind(&clones_json)
        .bind(&refs_json)
        .bind(&paths_json)
        .bind(&now)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    async fn get_snapshots(
        &self,
        repo: Option<&str>,
        limit: u32,
    ) -> StorageResult<Vec<TrafficSnapshot>> {
        let rows = if let Some(r) = repo {
            sqlx::query(
                "SELECT id, captured_at, repo, views_json, clones_json, refs_json, paths_json \
                 FROM traffic_snapshots WHERE repo = ? ORDER BY captured_at DESC LIMIT ?",
            )
            .bind(r)
            .bind(limit as i64)
            .fetch_all(&self.pool)
            .await?
        } else {
            sqlx::query(
                "SELECT id, captured_at, repo, views_json, clones_json, refs_json, paths_json \
                 FROM traffic_snapshots ORDER BY captured_at DESC LIMIT ?",
            )
            .bind(limit as i64)
            .fetch_all(&self.pool)
            .await?
        };

        let snaps = rows
            .into_iter()
            .filter_map(|row| deserialize_snapshot_row(&row).ok())
            .collect();
        Ok(snaps)
    }

    async fn export_json(&self, path: &Path) -> StorageResult<()> {
        use tokio::io::AsyncWriteExt;

        let events = self
            .get_events(&EventQuery {
                limit: None,
                ..Default::default()
            })
            .await?;
        let snaps = self.get_snapshots(None, 10_000).await?;

        let mut file = tokio::fs::File::create(path).await?;
        for e in &events {
            let line = serde_json::to_string(e)? + "\n";
            file.write_all(line.as_bytes()).await?;
        }
        for s in &snaps {
            let line = serde_json::to_string(s)? + "\n";
            file.write_all(line.as_bytes()).await?;
        }
        Ok(())
    }
}

pub struct JsonStorage {
    events_path: PathBuf,
    snapshots_path: PathBuf,
}

impl JsonStorage {
    pub fn new(dir: impl AsRef<Path>) -> Self {
        let d = dir.as_ref();
        Self {
            events_path: d.join("visitor_events.jsonl"),
            snapshots_path: d.join("traffic_snapshots.jsonl"),
        }
    }
}

#[async_trait]
impl VisitorStorage for JsonStorage {
    async fn record_event(&self, event: &VisitorEvent) -> StorageResult<()> {
        use tokio::io::AsyncWriteExt;
        let mut f = tokio::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(&self.events_path)
            .await?;
        let line = serde_json::to_string(event)? + "\n";
        f.write_all(line.as_bytes()).await?;
        Ok(())
    }

    async fn get_events(&self, query: &EventQuery) -> StorageResult<Vec<VisitorEvent>> {
        use tokio::io::AsyncBufReadExt;
        let file = match tokio::fs::File::open(&self.events_path).await {
            Ok(f) => f,
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(vec![]),
            Err(e) => return Err(StorageError::Io(e)),
        };
        let reader = tokio::io::BufReader::new(file);
        let mut lines = reader.lines();
        let mut events = Vec::new();
        while let Some(line) = lines.next_line().await? {
            if let Ok(e) = serde_json::from_str::<VisitorEvent>(&line) {
                events.push(e);
            }
        }
        let storage = InMemoryStorage::new();
        for e in events {
            let _ = storage.record_event(&e).await;
        }
        storage
            .get_events(query)
            .await
            .map_err(|_| StorageError::NotInitialised)
    }

    async fn save_snapshot(&self, snapshot: &TrafficSnapshot) -> StorageResult<()> {
        use tokio::io::AsyncWriteExt;
        let mut f = tokio::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(&self.snapshots_path)
            .await?;
        let line = serde_json::to_string(snapshot)? + "\n";
        f.write_all(line.as_bytes()).await?;
        Ok(())
    }

    async fn get_snapshots(
        &self,
        repo: Option<&str>,
        limit: u32,
    ) -> StorageResult<Vec<TrafficSnapshot>> {
        use tokio::io::AsyncBufReadExt;
        let file = match tokio::fs::File::open(&self.snapshots_path).await {
            Ok(f) => f,
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(vec![]),
            Err(e) => return Err(StorageError::Io(e)),
        };
        let reader = tokio::io::BufReader::new(file);
        let mut lines = reader.lines();
        let mut snaps = Vec::new();
        while let Some(line) = lines.next_line().await? {
            if let Ok(s) = serde_json::from_str::<TrafficSnapshot>(&line)
                && repo.map(|r| s.repo == r).unwrap_or(true) {
                    snaps.push(s);
                }
        }
        snaps.sort_by(|a, b| b.captured_at.cmp(&a.captured_at));
        snaps.truncate(limit as usize);
        Ok(snaps)
    }

    async fn export_json(&self, path: &Path) -> StorageResult<()> {
        tokio::fs::copy(&self.events_path, path).await?;
        Ok(())
    }
}

fn deserialize_event_row(row: &sqlx::sqlite::SqliteRow) -> Result<VisitorEvent, StorageError> {
    use sqlx::Row;
    use std::str::FromStr;

    let id_str: String = row.try_get("id")?;
    let ts_str: String = row.try_get("timestamp")?;
    let target_type: String = row.try_get("target_type")?;
    let target_name: String = row.try_get("target_name")?;
    let hashed_identity: Option<String> = row.try_get("hashed_identity")?;
    let user_agent: Option<String> = row.try_get("user_agent")?;
    let source_str: String = row.try_get("source")?;
    let filter_data: String = row.try_get("filter_data")?;

    let id = uuid::Uuid::from_str(&id_str).unwrap_or_else(|_| uuid::Uuid::new_v4());
    let timestamp = chrono::DateTime::parse_from_rfc3339(&ts_str)
        .map(|d| d.with_timezone(&Utc))
        .unwrap_or_else(|_| Utc::now());

    let target = if target_type == "profile" {
        VisitTarget::Profile {
            username: target_name,
        }
    } else {
        let parts: Vec<&str> = target_name.splitn(2, '/').collect();
        VisitTarget::Repository {
            owner: parts.first().unwrap_or(&"").to_string(),
            repo: parts.get(1).unwrap_or(&"").to_string(),
        }
    };

    let source = match source_str.as_str() {
        "github_api" => EventSource::GithubApi,
        "custom_pixel" => EventSource::CustomPixel,
        _ => EventSource::Manual,
    };

    let filter_result = serde_json::from_str(&filter_data)?;

    Ok(VisitorEvent {
        id,
        timestamp,
        target,
        hashed_identity,
        user_agent,
        source,
        filter_result,
    })
}

fn deserialize_snapshot_row(
    row: &sqlx::sqlite::SqliteRow,
) -> Result<TrafficSnapshot, StorageError> {
    use sqlx::Row;
    use std::str::FromStr;

    let id_str: String = row.try_get("id")?;
    let captured_str: String = row.try_get("captured_at")?;
    let repo: String = row.try_get("repo")?;
    let views_json: String = row.try_get("views_json")?;
    let clones_json: String = row.try_get("clones_json")?;
    let refs_json: String = row.try_get("refs_json")?;
    let paths_json: String = row.try_get("paths_json")?;

    let id = uuid::Uuid::from_str(&id_str).unwrap_or_else(|_| uuid::Uuid::new_v4());
    let captured_at = chrono::DateTime::parse_from_rfc3339(&captured_str)
        .map(|d| d.with_timezone(&Utc))
        .unwrap_or_else(|_| Utc::now());

    Ok(TrafficSnapshot {
        id,
        captured_at,
        repo,
        views: serde_json::from_str(&views_json)?,
        clones: serde_json::from_str(&clones_json)?,
        referrers: serde_json::from_str(&refs_json)?,
        top_paths: serde_json::from_str(&paths_json)?,
    })
}

use crate::github_visitors::models::{EventSource, VisitTarget};

#[cfg(test)]
mod tests {
    use super::*;
    use crate::github_visitors::models::{
        EventSource, FilterResult, TrafficClones, TrafficViews, VisitTarget, VisitorEvent,
    };
    use uuid::Uuid;

    fn make_event() -> VisitorEvent {
        VisitorEvent {
            id: Uuid::new_v4(),
            timestamp: Utc::now(),
            target: VisitTarget::Profile {
                username: "Andezion".into(),
            },
            hashed_identity: Some("abc123".into()),
            user_agent: Some("Mozilla/5.0".into()),
            source: EventSource::CustomPixel,
            filter_result: FilterResult::accept(),
        }
    }

    fn make_snapshot(repo: &str) -> TrafficSnapshot {
        TrafficSnapshot {
            id: Uuid::new_v4(),
            captured_at: Utc::now(),
            repo: repo.to_string(),
            views: TrafficViews {
                count: 42,
                uniques: 20,
                days: vec![],
            },
            clones: TrafficClones {
                count: 5,
                uniques: 3,
                days: vec![],
            },
            referrers: vec![],
            top_paths: vec![],
        }
    }

    #[tokio::test]
    async fn in_memory_round_trip_event() {
        let store = InMemoryStorage::new();
        let event = make_event();
        store.record_event(&event).await.unwrap();
        let events = store.get_events(&EventQuery::default()).await.unwrap();
        assert_eq!(events.len(), 1);
        assert_eq!(events[0].id, event.id);
    }

    #[tokio::test]
    async fn in_memory_round_trip_snapshot() {
        let store = InMemoryStorage::new();
        let snap = make_snapshot("Andezion/repo");
        store.save_snapshot(&snap).await.unwrap();
        let snaps = store.get_snapshots(None, 10).await.unwrap();
        assert_eq!(snaps.len(), 1);
        assert_eq!(snaps[0].repo, "Andezion/repo");
    }

    #[tokio::test]
    async fn in_memory_passed_only_filter() {
        use crate::github_visitors::models::{BotDetectionResult, FilterReason};
        let store = InMemoryStorage::new();

        let passed = make_event();
        let mut rejected = make_event();
        rejected.filter_result = FilterResult {
            passed: false,
            reasons: vec![FilterReason::BotUserAgent],
            bot_detection: BotDetectionResult {
                is_bot: true,
                confidence: 0.9,
                reason: Some(FilterReason::BotUserAgent),
                matched_pattern: Some("bot".into()),
            },
        };

        store.record_event(&passed).await.unwrap();
        store.record_event(&rejected).await.unwrap();

        let all = store.get_events(&EventQuery::default()).await.unwrap();
        let only_passed = store
            .get_events(&EventQuery {
                passed_only: true,
                ..Default::default()
            })
            .await
            .unwrap();

        assert_eq!(all.len(), 2);
        assert_eq!(only_passed.len(), 1);
    }

    #[tokio::test]
    async fn sqlite_in_memory_round_trip() {
        let store = SqliteStorage::open_in_memory().await.unwrap();

        let event = make_event();
        store.record_event(&event).await.unwrap();

        let snap = make_snapshot("Andezion/test-repo");
        store.save_snapshot(&snap).await.unwrap();

        let snaps = store
            .get_snapshots(Some("Andezion/test-repo"), 5)
            .await
            .unwrap();
        assert_eq!(snaps.len(), 1);
        assert_eq!(snaps[0].views.count, 42);
    }

    #[tokio::test]
    async fn sqlite_export_to_json() {
        let store = SqliteStorage::open_in_memory().await.unwrap();
        store.record_event(&make_event()).await.unwrap();
        store
            .save_snapshot(&make_snapshot("Andezion/repo"))
            .await
            .unwrap();

        let tmp = std::env::temp_dir().join("test_export.jsonl");
        store.export_json(&tmp).await.unwrap();
        assert!(tmp.exists());
        tokio::fs::remove_file(tmp).await.unwrap();
    }
}
