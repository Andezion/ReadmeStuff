use chrono::{DateTime, NaiveDate, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum VisitTarget {
    Profile { username: String },
    Repository { owner: String, repo: String },
}

impl VisitTarget {
    pub fn name(&self) -> String {
        match self {
            Self::Profile { username } => username.clone(),
            Self::Repository { owner, repo } => format!("{owner}/{repo}"),
        }
    }

    pub fn kind(&self) -> &'static str {
        match self {
            Self::Profile { .. } => "profile",
            Self::Repository { .. } => "repository",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EventSource {
    GithubApi,
    CustomPixel,
    Manual,
}

impl std::fmt::Display for EventSource {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::GithubApi => f.write_str("github_api"),
            Self::CustomPixel => f.write_str("custom_pixel"),
            Self::Manual => f.write_str("manual"),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FilterReason {
    BotUserAgent,
    GithubCamoProxy,
    GithubActionsAgent,
    EmptyUserAgent,
    DeduplicatedByWindow,
    RateLimitExceeded,
    SelfVisit,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BotDetectionResult {
    pub is_bot: bool,
    pub confidence: f32,
    pub reason: Option<FilterReason>,
    pub matched_pattern: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FilterResult {
    pub passed: bool,
    pub reasons: Vec<FilterReason>,
    pub bot_detection: BotDetectionResult,
}

impl FilterResult {
    pub fn accept() -> Self {
        Self {
            passed: true,
            reasons: vec![],
            bot_detection: BotDetectionResult {
                is_bot: false,
                confidence: 0.0,
                reason: None,
                matched_pattern: None,
            },
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VisitorEvent {
    pub id: Uuid,
    pub timestamp: DateTime<Utc>,
    pub target: VisitTarget,
    pub hashed_identity: Option<String>,
    pub user_agent: Option<String>,
    pub source: EventSource,
    pub filter_result: FilterResult,
}

impl VisitorEvent {
    pub fn new(
        target: VisitTarget,
        user_agent: Option<String>,
        hashed_identity: Option<String>,
        source: EventSource,
        filter_result: FilterResult,
    ) -> Self {
        Self {
            id: Uuid::new_v4(),
            timestamp: Utc::now(),
            target,
            hashed_identity,
            user_agent,
            source,
            filter_result,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VisitorSession {
    pub session_id: String,
    pub first_seen: DateTime<Utc>,
    pub last_seen: DateTime<Utc>,
    pub event_count: u32,
    pub targets: Vec<VisitTarget>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrafficDay {
    pub date: NaiveDate,
    pub count: u64,
    pub uniques: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrafficViews {
    pub count: u64,
    pub uniques: u64,
    pub days: Vec<TrafficDay>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrafficClones {
    pub count: u64,
    pub uniques: u64,
    pub days: Vec<TrafficDay>,
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
pub struct TrafficSnapshot {
    pub id: Uuid,
    pub captured_at: DateTime<Utc>,
    pub repo: String,
    pub views: TrafficViews,
    pub clones: TrafficClones,
    pub referrers: Vec<TrafficReferrer>,
    pub top_paths: Vec<TrafficPath>,
}

impl TrafficSnapshot {
    pub fn new(
        repo: String,
        views: TrafficViews,
        clones: TrafficClones,
        referrers: Vec<TrafficReferrer>,
        top_paths: Vec<TrafficPath>,
    ) -> Self {
        Self {
            id: Uuid::new_v4(),
            captured_at: Utc::now(),
            repo,
            views,
            clones,
            referrers,
            top_paths,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RepositoryTrafficSummary {
    pub repo: String,
    pub latest_snapshot: Option<TrafficSnapshot>,
    pub total_views_all_time: u64,
    pub total_unique_visitors_all_time: u64,
    pub total_clones_all_time: u64,
    pub total_unique_cloners_all_time: u64,
    pub top_referrers: Vec<TrafficReferrer>,
    pub top_paths: Vec<TrafficPath>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrendPoint {
    pub date: NaiveDate,
    pub total: u64,
    pub unique: u64,
    pub delta: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrafficTrend {
    pub data_points: Vec<TrendPoint>,
    pub growth_rate_pct: f64,
    pub is_growing: bool,
    pub average_daily: f64,
    pub peak_day: Option<NaiveDate>,
    pub peak_value: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct TrafficHeatmap {
    pub grid: [[u64; 24]; 7],
    pub peak_weekday: u8,
    pub peak_hour: u8,
    pub peak_count: u64,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct FilterSummary {
    pub total_raw: u64,
    pub passed: u64,
    pub bots_filtered: u64,
    pub camo_proxy_filtered: u64,
    pub github_actions_filtered: u64,
    pub dedup_filtered: u64,
    pub rate_limit_filtered: u64,
    pub self_visit_filtered: u64,
}

impl FilterSummary {
    pub fn total_filtered(&self) -> u64 {
        self.total_raw.saturating_sub(self.passed)
    }

    pub fn pass_rate(&self) -> f64 {
        if self.total_raw == 0 {
            1.0
        } else {
            self.passed as f64 / self.total_raw as f64
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VisitorAnalytics {
    pub generated_at: DateTime<Utc>,
    pub username: String,
    pub repositories: Vec<RepositoryTrafficSummary>,
    pub trend: TrafficTrend,
    pub heatmap: TrafficHeatmap,
    pub filter_summary: FilterSummary,
    pub returning_visitor_ratio: f64,
    pub top_repos_by_views: Vec<(String, u64)>,
    pub total_clones_all_time: u64,
    pub total_unique_cloners_all_time: u64,
    pub top_referrers: Vec<TrafficReferrer>,
    pub top_paths: Vec<TrafficPath>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RepoEngagement {
    pub repo: String,
    pub stargazer_count: u64,
    pub fork_count: u64,
    pub watcher_count: u64,
    pub recent_stargazers: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EngagementSummary {
    pub generated_at: DateTime<Utc>,
    pub total_stars: u64,
    pub total_forks: u64,
    pub total_watchers: u64,
    pub repositories: Vec<RepoEngagement>,
    pub recent_stargazers: Vec<(String, String)>,
}

#[derive(Debug, Clone, Default)]
pub struct EventQuery {
    pub target: Option<VisitTarget>,
    pub source: Option<EventSource>,
    pub from: Option<DateTime<Utc>>,
    pub to: Option<DateTime<Utc>>,
    pub passed_only: bool,
    pub limit: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UniqueVisitorStats {
    pub total_events: u64,
    pub counted_events: u64,
    pub distinct_identities: u64,
    pub breakdown: HashMap<String, u64>,
}
