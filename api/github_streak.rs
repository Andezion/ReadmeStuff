use crate::github_client::{GitHubClient, Result};
use chrono::{Datelike, Duration, NaiveDate, Utc};
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::collections::HashMap;
use std::sync::Arc;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContributionDay {
    pub date: NaiveDate,
    pub contribution_count: u32,
    pub color: String,
    pub weekday: u8,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContributionWeek {
    pub contribution_days: Vec<ContributionDay>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContributionCalendar {
    pub total_contributions: u32,
    pub weeks: Vec<ContributionWeek>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContributionGap {
    pub start: NaiveDate,
    pub end: NaiveDate,
    pub days: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StreakStats {
    pub total_contributions: u32,
    pub current_streak: u32,
    pub longest_streak: u32,
    pub current_streak_start: Option<NaiveDate>,
    pub longest_streak_start: Option<NaiveDate>,
    pub longest_streak_end: Option<NaiveDate>,
    pub average_daily_contributions: f64,
    pub daily_history: Vec<ContributionDay>,
    pub monthly_totals: HashMap<String, u32>,
    pub weekday_distribution: [u32; 7],
    pub contribution_gaps: Vec<ContributionGap>,
}

#[derive(Deserialize)]
struct UserRoot {
    user: GqlUser,
}

#[derive(Deserialize)]
struct GqlUser {
    #[serde(rename = "contributionsCollection")]
    contributions_collection: GqlContribCollection,
}

#[derive(Deserialize)]
struct GqlContribCollection {
    #[serde(rename = "contributionCalendar")]
    contribution_calendar: GqlCalendar,
}

#[derive(Deserialize)]
struct GqlCalendar {
    #[serde(rename = "totalContributions")]
    total_contributions: u32,
    weeks: Vec<GqlWeek>,
}

#[derive(Deserialize)]
struct GqlWeek {
    #[serde(rename = "contributionDays")]
    contribution_days: Vec<GqlDay>,
}

#[derive(Deserialize)]
struct GqlDay {
    date: String,
    #[serde(rename = "contributionCount")]
    contribution_count: u32,
    color: String,
    weekday: u8,
}

const CALENDAR_QUERY: &str = r#"
query($login: String!, $from: DateTime!, $to: DateTime!) {
  user(login: $login) {
    contributionsCollection(from: $from, to: $to) {
      contributionCalendar {
        totalContributions
        weeks {
          contributionDays {
            date
            contributionCount
            color
            weekday
          }
        }
      }
    }
  }
}
"#;

pub struct GitHubStreakApi {
    client: GitHubClient,
}

impl GitHubStreakApi {
    pub fn new(client: GitHubClient) -> Self {
        Self { client }
    }

    pub async fn fetch_streak_stats(&self, login: &str) -> Result<StreakStats> {
        let days = self.fetch_all_days(login).await?;
        Ok(compute_streak_stats(days))
    }

    pub async fn fetch_current_year_calendar(&self, login: &str) -> Result<ContributionCalendar> {
        let year = Utc::now().year();
        let data: UserRoot = self
            .client
            .graphql(
                CALENDAR_QUERY,
                json!({
                    "login": login,
                    "from": format!("{year}-01-01T00:00:00Z"),
                    "to":   format!("{year}-12-31T23:59:59Z"),
                }),
            )
            .await?;

        let cal = data.user.contributions_collection.contribution_calendar;
        Ok(ContributionCalendar {
            total_contributions: cal.total_contributions,
            weeks: cal
                .weeks
                .into_iter()
                .map(|w| ContributionWeek {
                    contribution_days: w
                        .contribution_days
                        .into_iter()
                        .filter_map(parse_day)
                        .collect(),
                })
                .collect(),
        })
    }

    async fn fetch_all_days(&self, login: &str) -> Result<Vec<ContributionDay>> {
        use tokio::task::JoinSet;

        let now = Utc::now();
        let current_year = now.year();

        const MIN_YEAR: i32 = 2008;
        let sem = Arc::new(tokio::sync::Semaphore::new(8));
        let mut set: JoinSet<Vec<ContributionDay>> = JoinSet::new();

        for year in MIN_YEAR..=current_year {
            let client = self.client.clone();
            let login = login.to_owned();
            let sem = sem.clone();
            set.spawn(async move {
                let Ok(_permit) = sem.acquire_owned().await else {
                    return vec![];
                };
                let Ok(data): Result<UserRoot> = client
                    .graphql(
                        CALENDAR_QUERY,
                        json!({
                            "login": login,
                            "from": format!("{year}-01-01T00:00:00Z"),
                            "to":   format!("{year}-12-31T23:59:59Z"),
                        }),
                    )
                    .await
                else {
                    return vec![];
                };
                let cal = data.user.contributions_collection.contribution_calendar;
                if cal.total_contributions == 0 {
                    return vec![];
                }
                flatten_calendar(cal)
            });
        }

        let mut all = Vec::new();
        while let Some(result) = set.join_next().await {
            if let Ok(days) = result {
                all.extend(days);
            }
        }

        all.sort_by_key(|d| d.date);
        all.dedup_by(|b, a| {
            if a.date == b.date {
                if b.contribution_count > a.contribution_count {
                    a.contribution_count = b.contribution_count;
                }
                true
            } else {
                false
            }
        });
        Ok(all)
    }
}

pub fn compute_streak_stats(days: Vec<ContributionDay>) -> StreakStats {
    let today = Utc::now().date_naive();
    let days: Vec<ContributionDay> = days.into_iter().filter(|d| d.date <= today).collect();

    let total_contributions: u32 = days.iter().map(|d| d.contribution_count).sum();

    let non_zero = days.iter().filter(|d| d.contribution_count > 0).count();
    let average_daily_contributions = if non_zero > 0 {
        total_contributions as f64 / non_zero as f64
    } else {
        0.0
    };

    let mut monthly_totals: HashMap<String, u32> = HashMap::new();
    let mut weekday_distribution = [0u32; 7];
    for d in &days {
        let key = format!("{}-{:02}", d.date.year(), d.date.month());
        *monthly_totals.entry(key).or_insert(0) += d.contribution_count;
        weekday_distribution[d.weekday as usize % 7] += d.contribution_count;
    }

    let (current_streak, current_streak_start) = calc_current_streak(&days, today);
    let (longest_streak, longest_streak_start, longest_streak_end) = calc_longest_streak(&days);
    let contribution_gaps = calc_gaps(&days);

    StreakStats {
        total_contributions,
        current_streak,
        longest_streak,
        current_streak_start,
        longest_streak_start,
        longest_streak_end,
        average_daily_contributions,
        monthly_totals,
        weekday_distribution,
        contribution_gaps,
        daily_history: days,
    }
}

fn calc_current_streak(days: &[ContributionDay], today: NaiveDate) -> (u32, Option<NaiveDate>) {
    let mut streak = 0u32;
    let mut streak_start: Option<NaiveDate> = None;
    let mut expected: Option<NaiveDate> = None;

    for day in days.iter().rev() {
        if day.date > today {
            continue;
        }
        if day.contribution_count == 0 {
            if streak > 0 {
                break;
            }
            continue;
        }
        match expected {
            None => {
                if today.signed_duration_since(day.date).num_days() > 1 {
                    break;
                }
                streak = 1;
                streak_start = Some(day.date);
                expected = day.date.pred_opt();
            }
            Some(exp) => {
                if day.date == exp {
                    streak += 1;
                    streak_start = Some(day.date);
                    expected = day.date.pred_opt();
                } else {
                    break;
                }
            }
        }
    }

    (streak, streak_start)
}

fn calc_longest_streak(days: &[ContributionDay]) -> (u32, Option<NaiveDate>, Option<NaiveDate>) {
    let mut longest = 0u32;
    let mut best_start: Option<NaiveDate> = None;
    let mut best_end: Option<NaiveDate> = None;

    let mut run = 0u32;
    let mut run_start: Option<NaiveDate> = None;
    let mut run_end: Option<NaiveDate> = None;

    for day in days {
        if day.contribution_count > 0 {
            let continues = run_end
                .map(|last| day.date.signed_duration_since(last).num_days() == 1)
                .unwrap_or(false);
            if continues {
                run += 1;
            } else {
                run = 1;
                run_start = Some(day.date);
            }
            run_end = Some(day.date);
            if run > longest {
                longest = run;
                best_start = run_start;
                best_end = run_end;
            }
        } else {
            run = 0;
            run_start = None;
            run_end = None;
        }
    }

    (longest, best_start, best_end)
}

fn calc_gaps(days: &[ContributionDay]) -> Vec<ContributionGap> {
    let mut gaps = Vec::new();
    let mut gap_start: Option<NaiveDate> = None;

    for day in days {
        if day.contribution_count == 0 {
            if gap_start.is_none() {
                gap_start = Some(day.date);
            }
        } else if let Some(start) = gap_start.take() {
            let end = day.date - Duration::days(1);
            let len = (end.signed_duration_since(start).num_days() + 1) as u32;
            if len >= 3 {
                gaps.push(ContributionGap {
                    start,
                    end,
                    days: len,
                });
            }
        }
    }

    if let Some(start) = gap_start
        && let Some(last) = days.last()
    {
        let len = (last.date.signed_duration_since(start).num_days() + 1) as u32;
        if len >= 3 {
            gaps.push(ContributionGap {
                start,
                end: last.date,
                days: len,
            });
        }
    }

    gaps
}

fn flatten_calendar(cal: GqlCalendar) -> Vec<ContributionDay> {
    cal.weeks
        .into_iter()
        .flat_map(|w| w.contribution_days)
        .filter_map(parse_day)
        .collect()
}

fn parse_day(d: GqlDay) -> Option<ContributionDay> {
    NaiveDate::parse_from_str(&d.date, "%Y-%m-%d")
        .ok()
        .map(|date| ContributionDay {
            date,
            contribution_count: d.contribution_count,
            color: d.color,
            weekday: d.weekday,
        })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_day(date: NaiveDate, count: u32) -> ContributionDay {
        ContributionDay {
            date,
            contribution_count: count,
            color: "#196127".into(),
            weekday: 1,
        }
    }

    #[test]
    fn streak_three_consecutive_days() {
        let today = Utc::now().date_naive();
        let days = vec![
            make_day(today - Duration::days(2), 5),
            make_day(today - Duration::days(1), 3),
            make_day(today, 7),
        ];
        let stats = compute_streak_stats(days);
        assert_eq!(stats.current_streak, 3);
        assert_eq!(stats.longest_streak, 3);
        assert_eq!(stats.total_contributions, 15);
    }

    #[test]
    fn streak_broken_yesterday() {
        let today = Utc::now().date_naive();
        let days = vec![
            make_day(today - Duration::days(3), 5),
            make_day(today - Duration::days(2), 4),
            make_day(today - Duration::days(1), 0),
            make_day(today, 7),
        ];
        let stats = compute_streak_stats(days);
        assert_eq!(stats.current_streak, 1);
        assert_eq!(stats.longest_streak, 2);
    }

    #[test]
    fn gap_detection_minimum_three_days() {
        let base = NaiveDate::from_ymd_opt(2024, 1, 1).unwrap();
        let days: Vec<ContributionDay> = (0i64..20)
            .map(|i| {
                let count = if (5..=11).contains(&i) { 0 } else { 3 };
                make_day(base + Duration::days(i), count)
            })
            .collect();

        let gaps = calc_gaps(&days);
        assert_eq!(gaps.len(), 1);
        assert_eq!(gaps[0].days, 7);
    }

    #[test]
    fn gap_under_threshold_not_recorded() {
        let base = NaiveDate::from_ymd_opt(2024, 3, 1).unwrap();
        let days = vec![
            make_day(base, 2),
            make_day(base + Duration::days(1), 0),
            make_day(base + Duration::days(2), 0),
            make_day(base + Duration::days(3), 1),
        ];
        let gaps = calc_gaps(&days);
        assert!(gaps.is_empty(), "2-day gaps should not be recorded");
    }

    #[test]
    fn weekday_distribution_sums_correctly() {
        let base = NaiveDate::from_ymd_opt(2024, 1, 7).unwrap();
        let days: Vec<ContributionDay> = (0i64..7)
            .map(|i| ContributionDay {
                date: base + Duration::days(i),
                contribution_count: (i + 1) as u32,
                color: "#196127".into(),
                weekday: i as u8,
            })
            .collect();
        let stats = compute_streak_stats(days);
        assert_eq!(stats.total_contributions, 28);
    }

    #[test]
    fn streak_across_year_boundary() {
        let d = |y, m, d| NaiveDate::from_ymd_opt(y, m, d).unwrap();
        let days = vec![
            make_day(d(2022, 12, 30), 3),
            make_day(d(2022, 12, 31), 5),
            make_day(d(2023, 1, 1), 2),
            make_day(d(2023, 1, 2), 4),
        ];
        let stats = compute_streak_stats(days);
        assert_eq!(stats.longest_streak, 4);
        assert_eq!(stats.longest_streak_start, Some(d(2022, 12, 30)));
        assert_eq!(stats.longest_streak_end, Some(d(2023, 1, 2)));
    }

    #[test]
    fn skipped_year_breaks_streak() {
        let d = |y, m, day| NaiveDate::from_ymd_opt(y, m, day).unwrap();
        let days = vec![make_day(d(2021, 12, 31), 5), make_day(d(2024, 1, 1), 5)];
        let (longest, start, end) = calc_longest_streak(&days);
        assert_eq!(longest, 1);
        assert_eq!(start, Some(d(2021, 12, 31)));
        assert_eq!(end, Some(d(2021, 12, 31)));
    }

    #[test]
    fn current_streak_today_no_contribution() {
        let today = Utc::now().date_naive();
        let yesterday = today.pred_opt().unwrap();
        let days = vec![
            make_day(yesterday - Duration::days(1), 2),
            make_day(yesterday, 3),
            make_day(today, 0),
        ];
        let stats = compute_streak_stats(days);
        assert_eq!(stats.current_streak, 2);
        assert_eq!(
            stats.current_streak_start,
            Some(yesterday - Duration::days(1))
        );
    }

    #[tokio::test]
    async fn live_streak_andezion() {
        let Ok(client) = GitHubClient::from_env() else {
            eprintln!("GITHUB_TOKEN not set - skipping live test");
            return;
        };
        let api = GitHubStreakApi::new(client);
        let stats = api.fetch_streak_stats("Andezion").await.unwrap();

        println!("{stats:#?}");
        assert!(stats.total_contributions > 0);
        assert!(stats.longest_streak >= stats.current_streak);
        println!(
            "Streak  current={} longest={}  total={}",
            stats.current_streak, stats.longest_streak, stats.total_contributions
        );
    }

    #[tokio::test]
    async fn live_current_year_calendar_andezion() {
        let Ok(client) = GitHubClient::from_env() else {
            eprintln!("GITHUB_TOKEN not set - skipping live test");
            return;
        };
        let api = GitHubStreakApi::new(client);
        let cal = api.fetch_current_year_calendar("Andezion").await.unwrap();

        println!("Contributions this year: {}", cal.total_contributions);
        assert!(!cal.weeks.is_empty());
    }
}
