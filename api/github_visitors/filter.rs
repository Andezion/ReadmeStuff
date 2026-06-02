use crate::github_visitors::models::{BotDetectionResult, FilterReason, FilterResult};
use chrono::{DateTime, Utc};
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct FilterConfig {
    pub dedup_window_secs: u64,
    pub max_requests_per_window: u32,
    pub rate_window_secs: u64,
    pub filter_bots: bool,
    pub filter_camo_proxy: bool,
    pub filter_github_actions: bool,
    pub filter_empty_user_agents: bool,
    pub owner_ip_hash: Option<String>,
}

impl Default for FilterConfig {
    fn default() -> Self {
        Self {
            dedup_window_secs: 1_800,
            max_requests_per_window: 20,
            rate_window_secs: 60,
            filter_bots: true,
            filter_camo_proxy: true,
            filter_github_actions: true,
            filter_empty_user_agents: false,
            owner_ip_hash: None,
        }
    }
}

impl FilterConfig {
    pub fn hash_ip(ip: &str) -> String {
        use sha2::{Digest, Sha256};
        let mut h = Sha256::new();
        h.update(ip.as_bytes());
        hex::encode(h.finalize())
    }
}

const BOT_PATTERNS: &[&str] = &[
    "googlebot",
    "bingbot",
    "yandexbot",
    "baiduspider",
    "duckduckbot",
    "slurp",
    "ia_archiver",
    "archive.org_bot",
    "msnbot",
    "teoma",
    "sogou",
    "facebookexternalhit",
    "twitterbot",
    "linkedinbot",
    "discordbot",
    "telegrambot",
    "whatsapp",
    "applebot",
    "petalbot",
    "semrushbot",
    "ahrefsbot",
    "mj12bot",
    "dotbot",
    "rogerbot",
    "exabot",
    "bot",
    "crawler",
    "spider",
    "scraper",
    "fetch",
    "python-requests",
    "python-urllib",
    "python/",
    "go-http-client",
    "java/",
    "jakarta commons-httpclient",
    "wget/",
    "curl/",
    "libwww-perl",
    "scrapy/",
    "aiohttp/",
    "axios/",
    "node-fetch/",
    "node.js",
    "okhttp/",
    "http.rb",
    "ruby",
    "httpclient",
    "php/",
    "perl/",
    "mechanize",
    "github-camo",
    "camo-rs",
    "camo asset proxy",
    "github actions",
    "github-actions",
];

const CAMO_PATTERNS: &[&str] = &["camo-rs", "camo asset proxy", "github-camo", "camo/"];

const GITHUB_ACTIONS_PATTERNS: &[&str] = &["github-actions", "github actions", "githubactions"];

#[derive(Debug, Default)]
pub struct FilterState {
    dedup: HashMap<String, DateTime<Utc>>,
    rate_windows: HashMap<String, Vec<DateTime<Utc>>>,
}

impl FilterState {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn gc(&mut self, max_age_secs: u64) {
        let cutoff = Utc::now() - chrono::Duration::seconds(max_age_secs as i64);
        self.dedup.retain(|_, ts| *ts > cutoff);
        for times in self.rate_windows.values_mut() {
            times.retain(|t| *t > cutoff);
        }
    }
}

pub struct VisitorFilter {
    pub config: FilterConfig,
    state: FilterState,
}

impl VisitorFilter {
    pub fn new(config: FilterConfig) -> Self {
        Self {
            config,
            state: FilterState::new(),
        }
    }

    pub fn evaluate(
        &mut self,
        user_agent: Option<&str>,
        hashed_identity: Option<&str>,
        target_key: &str,
        now: DateTime<Utc>,
    ) -> FilterResult {
        let mut reasons = Vec::new();

        let bot = detect_bot(user_agent);
        if self.config.filter_bots && bot.is_bot {
            reasons.push(bot.reason.clone().unwrap_or(FilterReason::BotUserAgent));
        }

        if self.config.filter_camo_proxy && is_camo_proxy(user_agent) {
            reasons.push(FilterReason::GithubCamoProxy);
        }

        if self.config.filter_github_actions && is_github_actions(user_agent) {
            reasons.push(FilterReason::GithubActionsAgent);
        }

        if self.config.filter_empty_user_agents
            && user_agent.map(|s| s.trim().is_empty()).unwrap_or(true)
        {
            reasons.push(FilterReason::EmptyUserAgent);
        }

        if let (Some(owner_hash), Some(identity)) = (&self.config.owner_ip_hash, hashed_identity) {
            if owner_hash == identity {
                reasons.push(FilterReason::SelfVisit);
            }
        }

        if !reasons.is_empty() {
            return FilterResult {
                passed: false,
                reasons,
                bot_detection: bot,
            };
        }

        if self.is_rate_limited(target_key, now) {
            reasons.push(FilterReason::RateLimitExceeded);
            return FilterResult {
                passed: false,
                reasons,
                bot_detection: bot,
            };
        }

        let dedup_key = match hashed_identity {
            Some(id) => format!("{id}:{target_key}"),
            None => {
                let bucket = now.timestamp() / self.config.dedup_window_secs as i64;
                format!("anon_bucket_{bucket}:{target_key}")
            }
        };

        if let Some(&last_seen) = self.state.dedup.get(&dedup_key) {
            let age = now.signed_duration_since(last_seen).num_seconds() as u64;
            if age < self.config.dedup_window_secs {
                reasons.push(FilterReason::DeduplicatedByWindow);
                return FilterResult {
                    passed: false,
                    reasons,
                    bot_detection: bot,
                };
            }
        }

        self.state.dedup.insert(dedup_key, now);
        FilterResult {
            passed: true,
            reasons: vec![],
            bot_detection: bot,
        }
    }

    fn is_rate_limited(&mut self, target_key: &str, now: DateTime<Utc>) -> bool {
        let window_start = now - chrono::Duration::seconds(self.rate_window_secs() as i64);
        let times = self
            .state
            .rate_windows
            .entry(target_key.to_string())
            .or_default();

        times.retain(|t| *t > window_start);
        times.push(now);

        times.len() as u32 > self.config.max_requests_per_window
    }

    fn rate_window_secs(&self) -> u64 {
        self.config.rate_window_secs
    }

    pub fn gc(&mut self) {
        let keep = self
            .config
            .dedup_window_secs
            .max(self.config.rate_window_secs)
            * 2;
        self.state.gc(keep);
    }
}

pub fn detect_bot(user_agent: Option<&str>) -> BotDetectionResult {
    let ua = match user_agent {
        Some(s) if !s.trim().is_empty() => s.to_lowercase(),
        _ => {
            return BotDetectionResult {
                is_bot: false,
                confidence: 0.0,
                reason: None,
                matched_pattern: None,
            };
        }
    };

    for &pattern in BOT_PATTERNS {
        if ua.contains(pattern) {
            return BotDetectionResult {
                is_bot: true,
                confidence: 0.95,
                reason: Some(FilterReason::BotUserAgent),
                matched_pattern: Some(pattern.to_string()),
            };
        }
    }

    BotDetectionResult {
        is_bot: false,
        confidence: 0.05,
        reason: None,
        matched_pattern: None,
    }
}

pub fn is_camo_proxy(user_agent: Option<&str>) -> bool {
    let ua = match user_agent.map(|s| s.to_lowercase()) {
        Some(s) if !s.is_empty() => s,
        _ => return false,
    };
    CAMO_PATTERNS.iter().any(|p| ua.contains(p))
}

pub fn is_github_actions(user_agent: Option<&str>) -> bool {
    let ua = match user_agent.map(|s| s.to_lowercase()) {
        Some(s) if !s.is_empty() => s,
        _ => return false,
    };
    GITHUB_ACTIONS_PATTERNS.iter().any(|p| ua.contains(p))
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::TimeZone;

    fn ts(secs: i64) -> DateTime<Utc> {
        Utc.timestamp_opt(secs, 0).unwrap()
    }

    #[test]
    fn bot_ua_detected() {
        let r = detect_bot(Some(
            "Mozilla/5.0 (compatible; Googlebot/2.1; +http://www.google.com/bot.html)",
        ));
        assert!(r.is_bot);
        assert_eq!(r.reason, Some(FilterReason::BotUserAgent));
    }

    #[test]
    fn curl_detected_as_bot() {
        let r = detect_bot(Some("curl/7.88.0"));
        assert!(r.is_bot);
    }

    #[test]
    fn real_browser_not_bot() {
        let r = detect_bot(Some(
            "Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0 Safari/537.36",
        ));
        assert!(!r.is_bot);
    }

    #[test]
    fn camo_proxy_detected() {
        assert!(is_camo_proxy(Some(
            "camo-rs (https://example.com; +https://github.com/atmos/camo) abc123"
        )));
        assert!(!is_camo_proxy(Some("Mozilla/5.0 Chrome/120.0")));
    }

    #[test]
    fn dedup_within_window_filtered() {
        let mut f = VisitorFilter::new(FilterConfig {
            dedup_window_secs: 1_800,
            ..Default::default()
        });

        let r1 = f.evaluate(Some("Mozilla/5.0"), Some("id-abc"), "Andezion", ts(1_000));
        assert!(r1.passed);
        let r2 = f.evaluate(Some("Mozilla/5.0"), Some("id-abc"), "Andezion", ts(1_300));
        assert!(!r2.passed);
        assert!(r2.reasons.contains(&FilterReason::DeduplicatedByWindow));
        let r3 = f.evaluate(
            Some("Mozilla/5.0"),
            Some("id-abc"),
            "Andezion",
            ts(1_000 + 1_860),
        );
        assert!(r3.passed);
    }

    #[test]
    fn self_visit_flagged() {
        let owner_hash = FilterConfig::hash_ip("192.168.1.1");
        let mut f = VisitorFilter::new(FilterConfig {
            owner_ip_hash: Some(owner_hash.clone()),
            ..Default::default()
        });
        let r = f.evaluate(
            Some("Mozilla/5.0"),
            Some(&owner_hash),
            "Andezion",
            Utc::now(),
        );
        assert!(!r.passed);
        assert!(r.reasons.contains(&FilterReason::SelfVisit));
    }

    #[test]
    fn rate_limit_triggers() {
        let mut f = VisitorFilter::new(FilterConfig {
            max_requests_per_window: 3,
            rate_window_secs: 60,
            filter_bots: false,
            filter_camo_proxy: false,
            filter_github_actions: false,
            filter_empty_user_agents: false,
            dedup_window_secs: 1,
            owner_ip_hash: None,
        });

        let base = 1_000i64;
        for i in 0..3 {
            let id = format!("id-{i}");
            let r = f.evaluate(Some("Mozilla"), Some(&id), "Andezion", ts(base + i));
            assert!(r.passed, "request {i} should pass");
        }
        let r = f.evaluate(Some("Mozilla"), Some("id-99"), "Andezion", ts(base + 3));
        assert!(!r.passed);
        assert!(r.reasons.contains(&FilterReason::RateLimitExceeded));
    }

    #[test]
    fn github_actions_filtered() {
        let mut f = VisitorFilter::new(FilterConfig::default());
        let r = f.evaluate(Some("github-actions/2.x"), None, "Andezion", Utc::now());
        assert!(!r.passed);
        assert!(r.reasons.contains(&FilterReason::GithubActionsAgent));
    }

    #[test]
    fn camo_request_filtered_by_default() {
        let mut f = VisitorFilter::new(FilterConfig::default());
        let r = f.evaluate(
            Some("camo-rs (https://raw.githubusercontent.com; +https://github.com/atmos/camo) 1a2b3c"),
            None,
            "Andezion",
            Utc::now(),
        );
        assert!(!r.passed);
        assert!(r.reasons.contains(&FilterReason::GithubCamoProxy));
    }

    #[test]
    fn anon_bucket_dedup_without_identity() {
        let mut f = VisitorFilter::new(FilterConfig {
            dedup_window_secs: 300,
            filter_bots: false,
            filter_camo_proxy: false,
            filter_github_actions: false,
            filter_empty_user_agents: false,
            ..Default::default()
        });

        let t1 = ts(0);
        let r1 = f.evaluate(Some("Mozilla"), None, "Andezion", t1);
        let r2 = f.evaluate(Some("Mozilla"), None, "Andezion", ts(60));
        let r3 = f.evaluate(Some("Mozilla"), None, "Andezion", ts(600));

        assert!(r1.passed);
        assert!(!r2.passed);
        assert!(r3.passed);
    }
}
