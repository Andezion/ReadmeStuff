use crate::{
    helpers::{fmt_num, xml_escape},
    matrix,
    theme::Theme,
};
use readme_stuff_aggregator::widgets::{EngagementWidget, GithubVisitorsWidget};
use readme_stuff_api::github_visitors::models::TrendHighlight;

const W: u32 = 400;
const REPO_ROW_H: u32 = 14;
const REPO_LIST_START_Y: u32 = 238;

const SPARK_CHART_H: u32 = 40;

fn render_sparkline(w: &GithubVisitorsWidget, top_y: u32, c: &crate::theme::Colors) -> (String, u32) {
    if w.weekly_views.len() < 2 {
        return (String::new(), 0);
    }

    let sep_y = top_y + 12;
    let label_y = sep_y + 14;
    let chart_top = label_y + 6;
    let block_h = (chart_top - top_y) + SPARK_CHART_H;

    let max_v = w.weekly_views.iter().map(|(_, v)| *v).max().unwrap_or(1).max(1);
    let n = w.weekly_views.len();
    let chart_left = 25f64;
    let chart_right = (W - 15) as f64;
    let points: String = w
        .weekly_views
        .iter()
        .enumerate()
        .map(|(i, (_, v))| {
            let x = chart_left + (chart_right - chart_left) * i as f64 / (n - 1) as f64;
            let y = chart_top as f64 + SPARK_CHART_H as f64 * (1.0 - *v as f64 / max_v as f64);
            format!("{x:.1},{y:.1}")
        })
        .collect::<Vec<_>>()
        .join(" ");

    let since = w
        .weekly_views
        .first()
        .map(|(d, _)| d.format("%b %Y").to_string())
        .unwrap_or_default();

    let svg = format!(
        "<line x1=\"25\" y1=\"{sep_y}\" x2=\"{lx2}\" y2=\"{sep_y}\" stroke=\"{sep}\" stroke-width=\"1\"/>\
         <text x=\"25\" y=\"{label_y}\" font-family=\"monospace\" font-size=\"10\" fill=\"{tl}\">Weekly views since {since}</text>\
         <polyline points=\"{points}\" fill=\"none\" stroke=\"{accent}\" stroke-width=\"1.5\"/>",
        lx2 = W - 15,
        sep = c.separator,
        tl = c.text_secondary,
        accent = c.accent,
    );
    (svg, block_h)
}

pub fn render_github_visitors(w: &GithubVisitorsWidget, theme: Theme) -> String {
    let c = theme.colors();
    let repo_count = w.top_repos.len() as u32;
    let repo_list_bottom = REPO_LIST_START_Y + repo_count * REPO_ROW_H;
    let (spark_svg, spark_h) = render_sparkline(w, repo_list_bottom, &c);
    let height = repo_list_bottom + if spark_h > 0 { spark_h } else { 0 } + 16;
    let rain = matrix::generate(W, height, c.matrix_color, c.matrix_opacity, 0xABCD_1234, "gvs");

    let stats_svg = format!(
        "<text x=\"25\" y=\"88\" font-family=\"monospace\" font-size=\"22\" font-weight=\"700\" fill=\"{tv}\">{views}</text>\
         <text x=\"25\" y=\"104\" font-family=\"monospace\" font-size=\"11\" fill=\"{tl}\">Total Views</text>\
         <text x=\"200\" y=\"88\" font-family=\"monospace\" font-size=\"22\" font-weight=\"700\" fill=\"{tv}\">{unique}</text>\
         <text x=\"200\" y=\"104\" font-family=\"monospace\" font-size=\"11\" fill=\"{tl}\">Unique Visitors</text>\
         <text x=\"25\" y=\"132\" font-family=\"monospace\" font-size=\"16\" font-weight=\"700\" fill=\"{tv}\">{clones}</text>\
         <text x=\"25\" y=\"146\" font-family=\"monospace\" font-size=\"11\" fill=\"{tl}\">Clones</text>\
         <text x=\"200\" y=\"132\" font-family=\"monospace\" font-size=\"16\" font-weight=\"700\" fill=\"{tv}\">{cloners}</text>\
         <text x=\"200\" y=\"146\" font-family=\"monospace\" font-size=\"11\" fill=\"{tl}\">Unique Cloners</text>",
        views = fmt_num(w.total_views),
        unique = fmt_num(w.total_unique),
        clones = fmt_num(w.total_clones),
        cloners = fmt_num(w.total_unique_cloners),
        tv = c.text_primary,
        tl = c.text_secondary,
    );

    let arrow = if w.growth_rate_pct > 0.0 {
        "^"
    } else if w.growth_rate_pct < 0.0 {
        "v"
    } else {
        "-"
    };
    let peak = w
        .peak_day
        .map(|d| format!("  peak {} on {}", fmt_num(w.peak_value), d.format("%b %d")))
        .unwrap_or_default();
    let trend_text = format!("Trend: {arrow} {:+.1}%{peak}", w.growth_rate_pct);

    let highlight_badge = match &w.highlight {
        Some(TrendHighlight::RecordDay { value, .. }) => {
            Some((format!("New record: {} views", fmt_num(*value)), c.accent))
        }
        Some(TrendHighlight::Spike { value, baseline, .. }) => {
            let ratio = if *baseline > 0.0 {
                *value as f64 / *baseline
            } else {
                0.0
            };
            Some((format!("Spike: {ratio:.1}x normal"), c.title))
        }
        None => None,
    };
    let badge_svg = highlight_badge
        .map(|(label, color)| {
            format!(
                "<text x=\"{rx}\" y=\"35\" font-family=\"monospace\" font-size=\"10\" font-weight=\"600\" fill=\"{color}\" text-anchor=\"end\">{label}</text>",
                rx = W - 15,
                label = xml_escape(&label),
            )
        })
        .unwrap_or_default();

    let mut meta_svg = format!(
        "<text x=\"25\" y=\"172\" font-family=\"monospace\" font-size=\"11\" fill=\"{tl}\">{trend}</text>",
        trend = xml_escape(&trend_text),
        tl = c.text_secondary,
    );
    if let Some((referrer, count)) = &w.top_referrer {
        meta_svg.push_str(&format!(
            "<text x=\"25\" y=\"188\" font-family=\"monospace\" font-size=\"11\" fill=\"{tl}\">Top referrer: {r} ({n})</text>",
            r = xml_escape(referrer),
            n = fmt_num(*count),
            tl = c.text_secondary,
        ));
    }
    if !w.referrer_trend.is_empty() {
        let trail: String = w
            .referrer_trend
            .iter()
            .rev()
            .take(3)
            .rev()
            .map(|(month, referrer, _)| {
                let short_month = month.rsplit('-').next().unwrap_or(month);
                format!("{short_month}:{referrer}")
            })
            .collect::<Vec<_>>()
            .join(" -> ");
        meta_svg.push_str(&format!(
            "<text x=\"25\" y=\"204\" font-family=\"monospace\" font-size=\"10\" fill=\"{tl}\">Referrers: {trail}</text>",
            trail = xml_escape(&trail),
            tl = c.text_secondary,
        ));
    }

    let mut repos_svg = String::new();
    for (i, (repo, views, growth_pct)) in w.top_repos.iter().enumerate() {
        let y = REPO_LIST_START_Y + i as u32 * REPO_ROW_H + 5;
        let short = repo.trim_start_matches('/');
        let growth_color = if *growth_pct > 0.0 {
            c.accent
        } else {
            c.text_secondary
        };
        repos_svg.push_str(&format!(
            "<text x=\"25\" y=\"{y}\" font-family=\"monospace\" font-size=\"10\" fill=\"{tl}\">{repo}</text>\
             <text x=\"{gx}\" y=\"{y}\" font-family=\"monospace\" font-size=\"9\" fill=\"{gc}\" text-anchor=\"end\">{g:+.0}%</text>\
             <text x=\"{rx}\" y=\"{y}\" font-family=\"monospace\" font-size=\"10\" font-weight=\"600\" fill=\"{tv}\" text-anchor=\"end\">{v}</text>",
            repo = xml_escape(short),
            g = growth_pct,
            v = fmt_num(*views),
            gx = W - 62,
            rx = W - 15,
            gc = growth_color,
            tl = c.text_secondary,
            tv = c.text_primary,
        ));
    }

    format!(
        r#"<svg width="{W}" height="{height}" viewBox="0 0 {W} {height}" xmlns="http://www.w3.org/2000/svg">
<defs>
  <clipPath id="gvs-clip">
    <rect width="{W}" height="{height}" rx="6"/>
  </clipPath>
</defs>
<rect width="{W}" height="{height}" rx="6" fill="{bg}"/>
<g clip-path="url(#gvs-clip)">{rain}</g>
<rect width="{W}" height="{height}" rx="6" fill="none" stroke="{border}" stroke-width="1"/>
<text x="25" y="35" font-family="monospace" font-size="14" font-weight="600" fill="{title}">GitHub Traffic</text>
{badge}
<line x1="25" y1="52" x2="{lx2}" y2="52" stroke="{sep}" stroke-width="1"/>
{stats}
<line x1="25" y1="158" x2="{lx2}" y2="158" stroke="{sep}" stroke-width="1"/>
{meta}
<line x1="25" y1="221" x2="{lx2}" y2="221" stroke="{sep}" stroke-width="1"/>
{repos}
{spark}
</svg>"#,
        bg = c.bg,
        border = c.border,
        title = c.title,
        sep = c.separator,
        lx2 = W - 15,
        stats = stats_svg,
        badge = badge_svg,
        meta = meta_svg,
        repos = repos_svg,
        spark = spark_svg,
    )
}

const ENGAGEMENT_ROW_H: u32 = 14;
const ENGAGEMENT_LIST_START_Y: u32 = 130;

pub fn render_github_engagement(w: &EngagementWidget, theme: Theme) -> String {
    let c = theme.colors();
    let row_count = w.recent_stargazers.len() as u32;
    let height = ENGAGEMENT_LIST_START_Y + row_count * ENGAGEMENT_ROW_H + 16;
    let rain = matrix::generate(W, height, c.matrix_color, c.matrix_opacity, 0x5EED_1234, "gge");

    let stats_svg = format!(
        "<text x=\"25\" y=\"80\" font-family=\"monospace\" font-size=\"22\" font-weight=\"700\" fill=\"{tv}\">{stars}</text>\
         <text x=\"25\" y=\"96\" font-family=\"monospace\" font-size=\"11\" fill=\"{tl}\">Stars</text>\
         <text x=\"160\" y=\"80\" font-family=\"monospace\" font-size=\"22\" font-weight=\"700\" fill=\"{tv}\">{forks}</text>\
         <text x=\"160\" y=\"96\" font-family=\"monospace\" font-size=\"11\" fill=\"{tl}\">Forks</text>\
         <text x=\"290\" y=\"80\" font-family=\"monospace\" font-size=\"22\" font-weight=\"700\" fill=\"{tv}\">{watchers}</text>\
         <text x=\"290\" y=\"96\" font-family=\"monospace\" font-size=\"11\" fill=\"{tl}\">Watchers</text>",
        stars = fmt_num(w.total_stars),
        forks = fmt_num(w.total_forks),
        watchers = fmt_num(w.total_watchers),
        tv = c.text_primary,
        tl = c.text_secondary,
    );

    let mut rows_svg = String::new();
    for (i, (login, repo)) in w.recent_stargazers.iter().enumerate() {
        let y = ENGAGEMENT_LIST_START_Y + i as u32 * ENGAGEMENT_ROW_H + 5;
        rows_svg.push_str(&format!(
            "<text x=\"25\" y=\"{y}\" font-family=\"monospace\" font-size=\"10\" fill=\"{tv}\">@{login}</text>\
             <text x=\"{rx}\" y=\"{y}\" font-family=\"monospace\" font-size=\"10\" fill=\"{tl}\" text-anchor=\"end\">{repo}</text>",
            login = xml_escape(login),
            repo = xml_escape(repo.trim_start_matches('/')),
            rx = W - 15,
            tv = c.text_primary,
            tl = c.text_secondary,
        ));
    }

    let list_label = if w.recent_stargazers.is_empty() {
        String::new()
    } else {
        format!(
            "<text x=\"25\" y=\"117\" font-family=\"monospace\" font-size=\"11\" fill=\"{tl}\">Recently starred by</text>",
            tl = c.text_secondary,
        )
    };

    format!(
        r#"<svg width="{W}" height="{height}" viewBox="0 0 {W} {height}" xmlns="http://www.w3.org/2000/svg">
<defs>
  <clipPath id="gge-clip">
    <rect width="{W}" height="{height}" rx="6"/>
  </clipPath>
</defs>
<rect width="{W}" height="{height}" rx="6" fill="{bg}"/>
<g clip-path="url(#gge-clip)">{rain}</g>
<rect width="{W}" height="{height}" rx="6" fill="none" stroke="{border}" stroke-width="1"/>
<text x="25" y="35" font-family="monospace" font-size="14" font-weight="600" fill="{title}">GitHub Engagement</text>
<line x1="25" y1="52" x2="{lx2}" y2="52" stroke="{sep}" stroke-width="1"/>
{stats}
{list_label}
{rows}
</svg>"#,
        bg = c.bg,
        border = c.border,
        title = c.title,
        sep = c.separator,
        lx2 = W - 15,
        stats = stats_svg,
        list_label = list_label,
        rows = rows_svg,
    )
}

pub fn render_github_commit_streak(
    w: &readme_stuff_aggregator::widgets::CommitStreakWidget,
    theme: Theme,
) -> String {
    use crate::helpers::fmt_num;
    let c = theme.colors();
    let rain = matrix::generate(
        400,
        150,
        c.matrix_color,
        c.matrix_opacity,
        0xDEAD_BEEF,
        "gcs",
    );

    let fmt_date = |d: Option<chrono::NaiveDate>| {
        d.map(|dt| dt.format("%b %d %Y").to_string())
            .unwrap_or_else(|| "-".to_string())
    };

    let streak_range = if w.longest_streak_start.is_some() || w.longest_streak_end.is_some() {
        format!(
            "{} -> {}",
            fmt_date(w.longest_streak_start),
            fmt_date(w.longest_streak_end)
        )
    } else {
        "-".to_string()
    };

    let stats_svg = format!(
        "<text x=\"25\" y=\"80\" font-family=\"monospace\" font-size=\"22\" font-weight=\"700\" fill=\"{tv}\">{tc}</text>\
         <text x=\"25\" y=\"96\" font-family=\"monospace\" font-size=\"11\" fill=\"{tl}\">Total Commits</text>\
         <text x=\"160\" y=\"80\" font-family=\"monospace\" font-size=\"22\" font-weight=\"700\" fill=\"{tv}\">{cs}</text>\
         <text x=\"160\" y=\"96\" font-family=\"monospace\" font-size=\"11\" fill=\"{tl}\">Current Streak</text>\
         <text x=\"290\" y=\"80\" font-family=\"monospace\" font-size=\"22\" font-weight=\"700\" fill=\"{tv}\">{ls}</text>\
         <text x=\"290\" y=\"96\" font-family=\"monospace\" font-size=\"11\" fill=\"{tl}\">Longest Streak</text>\
         <text x=\"25\" y=\"128\" font-family=\"monospace\" font-size=\"10\" fill=\"{tl}\">Best streak: {range}</text>",
        tc = fmt_num(w.total_commits),
        cs = format!("{}d", w.current_streak),
        ls = format!("{}d", w.longest_streak),
        range = xml_escape(&streak_range),
        tv = c.text_primary,
        tl = c.text_secondary,
    );

    format!(
        r#"<svg width="400" height="150" viewBox="0 0 400 150" xmlns="http://www.w3.org/2000/svg">
<defs>
  <clipPath id="gcs-clip">
    <rect width="400" height="150" rx="6"/>
  </clipPath>
</defs>
<rect width="400" height="150" rx="6" fill="{bg}"/>
<g clip-path="url(#gcs-clip)">{rain}</g>
<rect width="400" height="150" rx="6" fill="none" stroke="{border}" stroke-width="1"/>
<text x="25" y="35" font-family="monospace" font-size="14" font-weight="600" fill="{title}">Commit Streak</text>
<line x1="25" y1="48" x2="385" y2="48" stroke="{sep}" stroke-width="1"/>
{stats}
</svg>"#,
        bg = c.bg,
        border = c.border,
        title = c.title,
        sep = c.separator,
        rain = rain,
        stats = stats_svg,
    )
}
