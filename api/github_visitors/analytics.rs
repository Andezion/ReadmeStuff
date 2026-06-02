use crate::github_visitors::models::{
    FilterReason, FilterSummary, RepositoryTrafficSummary, TrafficHeatmap, TrafficSnapshot,
    TrafficTrend, TrendPoint, UniqueVisitorStats, VisitorAnalytics, VisitorEvent,
};
use chrono::{Datelike, NaiveDate, Timelike, Utc};
use std::collections::{BTreeMap, HashMap};

pub fn compute_analytics(
    username: &str,
    events: &[VisitorEvent],
    snapshots: &[TrafficSnapshot],
) -> VisitorAnalytics {
    let filter_summary = summarise_filter(events);
    let heatmap = build_heatmap(events);
    let repositories = aggregate_repo_traffic(snapshots);
    let trend = compute_trend_from_snapshots(snapshots);
    let returning = returning_visitor_ratio(events);

    let mut top_repos: Vec<(String, u64)> = repositories
        .iter()
        .map(|r| (r.repo.clone(), r.total_views_all_time))
        .collect();
    top_repos.sort_by(|a, b| b.1.cmp(&a.1));

    VisitorAnalytics {
        generated_at: Utc::now(),
        username: username.to_string(),
        repositories,
        trend,
        heatmap,
        filter_summary,
        returning_visitor_ratio: returning,
        top_repos_by_views: top_repos.into_iter().take(10).collect(),
    }
}

pub fn aggregate_repo_traffic(snapshots: &[TrafficSnapshot]) -> Vec<RepositoryTrafficSummary> {
    let mut by_repo: HashMap<String, Vec<&TrafficSnapshot>> = HashMap::new();
    for s in snapshots {
        by_repo.entry(s.repo.clone()).or_default().push(s);
    }

    let mut summaries: Vec<RepositoryTrafficSummary> = by_repo
        .into_iter()
        .map(|(repo, snaps)| {
            let total_views: u64 = snaps.iter().map(|s| s.views.count).sum();
            let total_unique: u64 = snaps.iter().map(|s| s.views.uniques).sum();
            let total_clones: u64 = snaps.iter().map(|s| s.clones.count).sum();

            let latest = snaps.iter().max_by_key(|s| s.captured_at).copied().cloned();

            let mut ref_map: HashMap<String, (u64, u64)> = HashMap::new();
            for s in &snaps {
                for r in &s.referrers {
                    let e = ref_map.entry(r.referrer.clone()).or_default();
                    e.0 += r.count;
                    e.1 += r.uniques;
                }
            }
            let mut top_referrers: Vec<_> = ref_map
                .into_iter()
                .map(|(referrer, (count, uniques))| {
                    crate::github_visitors::models::TrafficReferrer {
                        referrer,
                        count,
                        uniques,
                    }
                })
                .collect();
            top_referrers.sort_by(|a, b| b.count.cmp(&a.count));
            top_referrers.truncate(10);

            let mut path_map: HashMap<String, (String, u64, u64)> = HashMap::new();
            for s in &snaps {
                for p in &s.top_paths {
                    let e = path_map
                        .entry(p.path.clone())
                        .or_insert((p.title.clone(), 0, 0));
                    e.1 += p.count;
                    e.2 += p.uniques;
                }
            }
            let mut top_paths: Vec<_> = path_map
                .into_iter()
                .map(|(path, (title, count, uniques))| {
                    crate::github_visitors::models::TrafficPath {
                        path,
                        title,
                        count,
                        uniques,
                    }
                })
                .collect();
            top_paths.sort_by(|a, b| b.count.cmp(&a.count));
            top_paths.truncate(10);

            RepositoryTrafficSummary {
                repo,
                latest_snapshot: latest,
                total_views_all_time: total_views,
                total_unique_visitors_all_time: total_unique,
                total_clones_all_time: total_clones,
                top_referrers,
                top_paths,
            }
        })
        .collect();

    summaries.sort_by(|a, b| b.total_views_all_time.cmp(&a.total_views_all_time));
    summaries
}

pub fn compute_trend_from_snapshots(snapshots: &[TrafficSnapshot]) -> TrafficTrend {
    let mut daily: BTreeMap<NaiveDate, (u64, u64)> = BTreeMap::new();
    for snap in snapshots {
        for d in &snap.views.days {
            let e = daily.entry(d.date).or_default();
            e.0 = e.0.max(d.count);
            e.1 = e.1.max(d.uniques);
        }
    }

    if daily.is_empty() {
        return TrafficTrend {
            data_points: vec![],
            growth_rate_pct: 0.0,
            is_growing: false,
            average_daily: 0.0,
            peak_day: None,
            peak_value: 0,
        };
    }

    let dates: Vec<NaiveDate> = daily.keys().copied().collect();
    let mut data_points: Vec<TrendPoint> = Vec::with_capacity(dates.len());
    let mut prev_total = 0u64;

    for date in &dates {
        let (total, unique) = daily[date];
        let delta = total as i64 - prev_total as i64;
        data_points.push(TrendPoint {
            date: *date,
            total,
            unique,
            delta,
        });
        prev_total = total;
    }
    if let Some(p) = data_points.first_mut() {
        p.delta = 0;
    }

    let peak = data_points.iter().max_by_key(|p| p.total).unwrap();
    let peak_day = Some(peak.date);
    let peak_value = peak.total;

    let n = data_points.len();
    let half = n / 2;
    let first_half_avg =
        data_points[..half].iter().map(|p| p.total).sum::<u64>() as f64 / half.max(1) as f64;
    let second_half_avg =
        data_points[half..].iter().map(|p| p.total).sum::<u64>() as f64 / (n - half).max(1) as f64;

    let growth_rate_pct = if first_half_avg > 0.0 {
        (second_half_avg - first_half_avg) / first_half_avg * 100.0
    } else {
        0.0
    };

    let average_daily = data_points.iter().map(|p| p.total).sum::<u64>() as f64 / n as f64;

    TrafficTrend {
        data_points,
        growth_rate_pct,
        is_growing: growth_rate_pct > 0.0,
        average_daily,
        peak_day,
        peak_value,
    }
}

pub fn build_heatmap(events: &[VisitorEvent]) -> TrafficHeatmap {
    let mut grid = [[0u64; 24]; 7];

    for e in events.iter().filter(|e| e.filter_result.passed) {
        let ts = e.timestamp;
        let weekday = ts.weekday().num_days_from_sunday() as usize;
        let hour = ts.hour() as usize;
        grid[weekday][hour] += 1;
    }

    let (mut peak_wd, mut peak_hr, mut peak_count) = (0u8, 0u8, 0u64);
    for (wd, hours) in grid.iter().enumerate() {
        for (hr, &cnt) in hours.iter().enumerate() {
            if cnt > peak_count {
                peak_count = cnt;
                peak_wd = wd as u8;
                peak_hr = hr as u8;
            }
        }
    }

    TrafficHeatmap {
        grid,
        peak_weekday: peak_wd,
        peak_hour: peak_hr,
        peak_count,
    }
}

pub fn summarise_filter(events: &[VisitorEvent]) -> FilterSummary {
    let mut s = FilterSummary {
        total_raw: events.len() as u64,
        ..Default::default()
    };

    for e in events {
        if e.filter_result.passed {
            s.passed += 1;
        } else {
            for reason in &e.filter_result.reasons {
                match reason {
                    FilterReason::BotUserAgent => s.bots_filtered += 1,
                    FilterReason::GithubCamoProxy => s.camo_proxy_filtered += 1,
                    FilterReason::GithubActionsAgent => s.github_actions_filtered += 1,
                    FilterReason::DeduplicatedByWindow => s.dedup_filtered += 1,
                    FilterReason::RateLimitExceeded => s.rate_limit_filtered += 1,
                    FilterReason::SelfVisit => s.self_visit_filtered += 1,
                    FilterReason::EmptyUserAgent => s.bots_filtered += 1,
                }
            }
        }
    }
    s
}

pub fn returning_visitor_ratio(events: &[VisitorEvent]) -> f64 {
    let passed: Vec<&VisitorEvent> = events.iter().filter(|e| e.filter_result.passed).collect();
    let with_identity: Vec<&&VisitorEvent> = passed
        .iter()
        .filter(|e| e.hashed_identity.is_some())
        .collect();

    if with_identity.is_empty() {
        return 0.0;
    }

    let mut counts: HashMap<&str, u32> = HashMap::new();
    for e in &with_identity {
        *counts
            .entry(e.hashed_identity.as_deref().unwrap())
            .or_insert(0) += 1;
    }

    let returning = counts.values().filter(|&&n| n > 1).count();
    returning as f64 / counts.len() as f64
}

pub fn unique_visitor_stats(events: &[VisitorEvent]) -> UniqueVisitorStats {
    let passed: Vec<&VisitorEvent> = events.iter().filter(|e| e.filter_result.passed).collect();

    let distinct_identities = passed
        .iter()
        .filter_map(|e| e.hashed_identity.as_deref())
        .collect::<std::collections::HashSet<_>>()
        .len() as u64;

    let mut breakdown: HashMap<String, u64> = HashMap::new();
    for e in &passed {
        *breakdown.entry(e.source.to_string()).or_insert(0) += 1;
    }

    UniqueVisitorStats {
        total_events: events.len() as u64,
        counted_events: passed.len() as u64,
        distinct_identities,
        breakdown,
    }
}

pub fn daily_active_visitors(events: &[VisitorEvent]) -> BTreeMap<NaiveDate, u64> {
    let mut day_map: BTreeMap<NaiveDate, std::collections::HashSet<String>> = BTreeMap::new();

    for e in events.iter().filter(|e| e.filter_result.passed) {
        let date = e.timestamp.date_naive();
        let key = e
            .hashed_identity
            .clone()
            .unwrap_or_else(|| format!("anon-{}", e.id));
        day_map.entry(date).or_default().insert(key);
    }

    day_map
        .into_iter()
        .map(|(d, set)| (d, set.len() as u64))
        .collect()
}

pub fn repo_popularity_ranking(snapshots: &[TrafficSnapshot]) -> Vec<(String, u64, u64)> {
    let mut totals: HashMap<String, (u64, u64)> = HashMap::new();
    for s in snapshots {
        let e = totals.entry(s.repo.clone()).or_default();
        e.0 += s.views.count;
        e.1 += s.views.uniques;
    }

    let mut ranked: Vec<(String, u64, u64)> =
        totals.into_iter().map(|(r, (v, u))| (r, v, u)).collect();
    ranked.sort_by(|a, b| b.1.cmp(&a.1));
    ranked
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::github_visitors::models::{
        BotDetectionResult, EventSource, FilterResult, TrafficClones, TrafficViews, VisitTarget,
        VisitorEvent,
    };
    use chrono::{TimeZone, Utc};
    use uuid::Uuid;

    fn make_snap(
        repo: &str,
        days: &[(NaiveDate, u64, u64)],
        total: u64,
        unique: u64,
    ) -> TrafficSnapshot {
        TrafficSnapshot {
            id: Uuid::new_v4(),
            captured_at: Utc::now(),
            repo: repo.to_string(),
            views: TrafficViews {
                count: total,
                uniques: unique,
                days: days
                    .iter()
                    .map(|(d, c, u)| TrafficDay {
                        date: *d,
                        count: *c,
                        uniques: *u,
                    })
                    .collect(),
            },
            clones: TrafficClones {
                count: 0,
                uniques: 0,
                days: vec![],
            },
            referrers: vec![],
            top_paths: vec![],
        }
    }

    fn make_event(passed: bool, hashed_id: Option<&str>, ts_secs: i64) -> VisitorEvent {
        VisitorEvent {
            id: Uuid::new_v4(),
            timestamp: Utc.timestamp_opt(ts_secs, 0).unwrap(),
            target: VisitTarget::Profile {
                username: "Andezion".into(),
            },
            hashed_identity: hashed_id.map(|s| s.to_string()),
            user_agent: Some("Mozilla/5.0".into()),
            source: EventSource::CustomPixel,
            filter_result: if passed {
                FilterResult::accept()
            } else {
                FilterResult {
                    passed: false,
                    reasons: vec![FilterReason::BotUserAgent],
                    bot_detection: BotDetectionResult {
                        is_bot: true,
                        confidence: 0.95,
                        reason: Some(FilterReason::BotUserAgent),
                        matched_pattern: Some("bot".into()),
                    },
                }
            },
        }
    }

    #[test]
    fn repo_aggregation_sums_across_snapshots() {
        let d1 = NaiveDate::from_ymd_opt(2024, 5, 1).unwrap();
        let d2 = NaiveDate::from_ymd_opt(2024, 5, 2).unwrap();
        let snaps = vec![
            make_snap("Andezion/repo-a", &[(d1, 10, 7)], 10, 7),
            make_snap("Andezion/repo-a", &[(d2, 15, 9)], 15, 9),
            make_snap("Andezion/repo-b", &[(d1, 5, 3)], 5, 3),
        ];
        let summaries = aggregate_repo_traffic(&snaps);
        let a = summaries
            .iter()
            .find(|s| s.repo == "Andezion/repo-a")
            .unwrap();
        assert_eq!(a.total_views_all_time, 25);
        assert!(summaries[0].total_views_all_time >= summaries[1].total_views_all_time);
    }

    #[test]
    fn heatmap_populated_correctly() {
        let events = vec![
            make_event(true, Some("id-1"), 0),
            make_event(true, Some("id-2"), 3_600),
        ];
        let hm = build_heatmap(&events);
        let total: u64 = hm.grid.iter().flat_map(|r| r.iter()).sum();
        assert_eq!(total, 2);
    }

    #[test]
    fn filter_summary_counts_correctly() {
        let events = vec![
            make_event(true, Some("id-1"), 0),
            make_event(false, None, 10),
            make_event(false, None, 20),
        ];
        let s = summarise_filter(&events);
        assert_eq!(s.total_raw, 3);
        assert_eq!(s.passed, 1);
        assert_eq!(s.bots_filtered, 2);
    }

    #[test]
    fn returning_visitor_ratio_all_new() {
        let events = vec![
            make_event(true, Some("id-1"), 0),
            make_event(true, Some("id-2"), 10),
            make_event(true, Some("id-3"), 20),
        ];
        assert_eq!(returning_visitor_ratio(&events), 0.0);
    }

    #[test]
    fn returning_visitor_ratio_half_returning() {
        let events = vec![
            make_event(true, Some("id-1"), 0),
            make_event(true, Some("id-1"), 100),
            make_event(true, Some("id-2"), 200),
        ];
        assert_eq!(returning_visitor_ratio(&events), 0.5);
    }

    #[test]
    fn trend_growth_rate_positive() {
        let base = NaiveDate::from_ymd_opt(2024, 5, 1).unwrap();
        let days: Vec<(NaiveDate, u64, u64)> = (0..14i64)
            .map(|i| {
                let d = base + chrono::Duration::days(i);
                let count = (i + 1) as u64 * 10;
                (d, count, count / 2)
            })
            .collect();
        let snap = make_snap("repo", &days, days.iter().map(|(_, c, _)| c).sum(), 0);
        let trend = compute_trend_from_snapshots(&[snap]);
        assert!(trend.is_growing);
        assert!(trend.growth_rate_pct > 0.0);
    }

    #[test]
    fn daily_active_visitors_deduplicates() {
        let d1 = NaiveDate::from_ymd_opt(2024, 5, 1).unwrap();
        let day_start = d1.and_hms_opt(0, 0, 0).unwrap().and_utc().timestamp();
        let events = vec![
            make_event(true, Some("id-1"), day_start),
            make_event(true, Some("id-1"), day_start + 3_600),
            make_event(true, Some("id-2"), day_start + 7_200),
        ];
        let dav = daily_active_visitors(&events);
        assert_eq!(dav[&d1], 2);
    }
}
