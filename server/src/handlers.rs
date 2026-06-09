use axum::{
    extract::{Query, State},
    http::header,
    response::{IntoResponse, Response},
};
use readme_stuff_aggregator::{models::UserProfile, profile::build_profile, widgets};
use readme_stuff_cache::{CacheKey, DashboardCache};
use serde::Deserialize;
use std::sync::Arc;

use crate::AppState;


#[derive(Deserialize)]
pub struct ProfileQuery {
    pub login: Option<String>,
    pub cf:    Option<String>,
    pub cw:    Option<String>,
    pub lc:    Option<String>,
}

#[derive(Deserialize)]
pub struct LangsQuery {
    pub login: Option<String>,
    pub cf:    Option<String>,
    pub cw:    Option<String>,
    pub lc:    Option<String>,
    pub top:   Option<usize>,
}


fn svg_response(svg: String) -> Response {
    ([(header::CONTENT_TYPE, "image/svg+xml; charset=utf-8")], svg).into_response()
}

fn placeholder(label: &str) -> String {
    format!(
        r##"<svg xmlns="http://www.w3.org/2000/svg" width="400" height="60">
  <rect width="400" height="60" rx="6" fill="#1e1e2e"/>
  <text x="200" y="35" font-family="monospace" font-size="14"
        fill="#6e7681" text-anchor="middle">{label}: no data</text>
</svg>"##
    )
}


async fn get_profile(
    cache: &DashboardCache,
    login: &str,
    cf: &str,
    cw: &str,
    lc: &str,
) -> Arc<UserProfile> {
    let key = CacheKey::new(login, cf, cw, lc);
    if let Some(cached) = cache.get(&key).await {
        return cached;
    }
    tracing::info!(login, cf, cw, lc, "cache miss — fetching profile");
    let profile = build_profile(login, cf, cw, lc).await;
    cache.set(key, profile).await
}


fn render_github(profile: &UserProfile) -> String {
    let Some(w) = widgets::github_stats_widget(profile) else {
        return placeholder("github stats");
    };
    format!(
        r##"<svg xmlns="http://www.w3.org/2000/svg" width="400" height="120">
  <rect width="400" height="120" rx="6" fill="#1e1e2e"/>
  <text x="20" y="30"  font-family="monospace" font-size="14" fill="#cdd6f4">{login}</text>
  <text x="20" y="55"  font-family="monospace" font-size="12" fill="#89b4fa">stars {stars}  commits {commits}  PRs {prs}  issues {issues}</text>
  <text x="20" y="80"  font-family="monospace" font-size="12" fill="#a6e3a1">rank {rank}  followers {followers}</text>
</svg>"##,
        login     = w.login,
        stars     = w.stars,
        commits   = w.commits,
        prs       = w.prs,
        issues    = w.issues,
        rank      = w.rank,
        followers = w.followers,
    )
}

fn render_streak(profile: &UserProfile) -> String {
    let Some(w) = widgets::streak_widget(profile) else {
        return placeholder("streak");
    };
    format!(
        r##"<svg xmlns="http://www.w3.org/2000/svg" width="400" height="90">
  <rect width="400" height="90" rx="6" fill="#1e1e2e"/>
  <text x="20" y="35" font-family="monospace" font-size="14" fill="#cdd6f4">current streak: {current}</text>
  <text x="20" y="60" font-family="monospace" font-size="12" fill="#89b4fa">longest {longest}  total contributions {total}</text>
</svg>"##,
        current = w.current_streak,
        longest = w.longest_streak,
        total   = w.total_contributions,
    )
}

fn render_langs(profile: &UserProfile, top_n: usize) -> String {
    let Some(w) = widgets::langs_widget(profile, top_n) else {
        return placeholder("langs");
    };
    let rows: String = w
        .top
        .iter()
        .enumerate()
        .map(|(i, lang)| {
            let y = 35 + i * 22;
            let color = lang.color.as_deref().unwrap_or("#89b4fa");
            format!(
                r##"  <text x="20" y="{y}" font-family="monospace" font-size="12" fill="{color}">{name}  {pct:.1}%</text>"##,
                name = lang.name,
                pct  = lang.percentage,
            )
        })
        .collect::<Vec<_>>()
        .join("\n");
    let height = 50 + w.top.len() * 22;
    format!(
        r##"<svg xmlns="http://www.w3.org/2000/svg" width="400" height="{height}">
  <rect width="400" height="{height}" rx="6" fill="#1e1e2e"/>
{rows}
</svg>"##
    )
}

fn render_competitive(profile: &UserProfile) -> String {
    let Some(w) = widgets::competitive_widget(profile) else {
        return placeholder("competitive");
    };
    let cf_rating = w.cf_rating.map_or_else(|| "-".to_owned(), |r| r.to_string());
    let cf_rank   = w.cf_rank.as_deref().unwrap_or("-");
    let cw_rank   = w.cw_rank.as_deref().unwrap_or("-");
    let cw_honor  = w.cw_honor.map_or_else(|| "-".to_owned(), |h| h.to_string());
    let lc_solved = w.lc_solved.map_or_else(|| "-".to_owned(), |s| s.to_string());
    format!(
        r##"<svg xmlns="http://www.w3.org/2000/svg" width="400" height="110">
  <rect width="400" height="110" rx="6" fill="#1e1e2e"/>
  <text x="20" y="35" font-family="monospace" font-size="12" fill="#f38ba8">Codeforces {cf_rating} ({cf_rank})</text>
  <text x="20" y="60" font-family="monospace" font-size="12" fill="#a6e3a1">Codewars {cw_rank}  honor {cw_honor}</text>
  <text x="20" y="85" font-family="monospace" font-size="12" fill="#fab387">LeetCode solved {lc_solved}</text>
</svg>"##
    )
}


pub async fn github(
    State(state): State<AppState>,
    Query(q): Query<ProfileQuery>,
) -> Response {
    let profile = get_profile(
        &state.cache,
        q.login.as_deref().unwrap_or(""),
        q.cf.as_deref().unwrap_or(""),
        q.cw.as_deref().unwrap_or(""),
        q.lc.as_deref().unwrap_or(""),
    ).await;
    svg_response(render_github(&profile))
}

pub async fn streak(
    State(state): State<AppState>,
    Query(q): Query<ProfileQuery>,
) -> Response {
    let profile = get_profile(
        &state.cache,
        q.login.as_deref().unwrap_or(""),
        q.cf.as_deref().unwrap_or(""),
        q.cw.as_deref().unwrap_or(""),
        q.lc.as_deref().unwrap_or(""),
    ).await;
    svg_response(render_streak(&profile))
}

pub async fn langs(
    State(state): State<AppState>,
    Query(q): Query<LangsQuery>,
) -> Response {
    let profile = get_profile(
        &state.cache,
        q.login.as_deref().unwrap_or(""),
        q.cf.as_deref().unwrap_or(""),
        q.cw.as_deref().unwrap_or(""),
        q.lc.as_deref().unwrap_or(""),
    ).await;
    svg_response(render_langs(&profile, q.top.unwrap_or(6)))
}

pub async fn competitive(
    State(state): State<AppState>,
    Query(q): Query<ProfileQuery>,
) -> Response {
    let profile = get_profile(
        &state.cache,
        q.login.as_deref().unwrap_or(""),
        q.cf.as_deref().unwrap_or(""),
        q.cw.as_deref().unwrap_or(""),
        q.lc.as_deref().unwrap_or(""),
    ).await;
    svg_response(render_competitive(&profile))
}
