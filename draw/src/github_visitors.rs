use crate::{
    helpers::{fmt_num, xml_escape},
    matrix,
    theme::Theme,
};
use readme_stuff_aggregator::widgets::GithubVisitorsWidget;

const W: u32 = 400;
const H: u32 = 360;

pub fn render_github_visitors(w: &GithubVisitorsWidget, theme: Theme) -> String {
    let c = theme.colors();
    let rain = matrix::generate(W, H, c.matrix_color, c.matrix_opacity, 0xABCD_1234, "gvs");

    let stats_svg = format!(
        "<text x=\"25\" y=\"88\" font-family=\"monospace\" font-size=\"22\" font-weight=\"700\" fill=\"{tv}\">{views}</text>\
         <text x=\"25\" y=\"104\" font-family=\"monospace\" font-size=\"11\" fill=\"{tl}\">Total Views (14d)</text>\
         <text x=\"200\" y=\"88\" font-family=\"monospace\" font-size=\"22\" font-weight=\"700\" fill=\"{tv}\">{unique}</text>\
         <text x=\"200\" y=\"104\" font-family=\"monospace\" font-size=\"11\" fill=\"{tl}\">Unique Visitors</text>",
        views = fmt_num(w.total_views),
        unique = fmt_num(w.total_unique),
        tv = c.text_primary,
        tl = c.text_secondary,
    );

    let mut repos_svg = String::new();
    for (i, (repo, views)) in w.top_repos.iter().take(5).enumerate() {
        let y = 122 + i as u32 * 14;
        let short = repo.trim_start_matches('/');
        repos_svg.push_str(&format!(
            "<text x=\"25\" y=\"{y}\" font-family=\"monospace\" font-size=\"10\" fill=\"{tl}\">{repo}</text>\
             <text x=\"{rx}\" y=\"{y}\" font-family=\"monospace\" font-size=\"10\" font-weight=\"600\" fill=\"{tv}\" text-anchor=\"end\">{v}</text>",
            repo = xml_escape(short),
            v = fmt_num(*views),
            rx = W - 15,
            tl = c.text_secondary,
            tv = c.text_primary,
        ));
    }

    format!(
        r#"<svg width="{W}" height="{H}" viewBox="0 0 {W} {H}" xmlns="http://www.w3.org/2000/svg">
<defs>
  <clipPath id="gvs-clip">
    <rect width="{W}" height="{H}" rx="6"/>
  </clipPath>
</defs>
<rect width="{W}" height="{H}" rx="6" fill="{bg}"/>
<g clip-path="url(#gvs-clip)">{rain}</g>
<rect width="{W}" height="{H}" rx="6" fill="none" stroke="{border}" stroke-width="1"/>
<text x="25" y="35" font-family="monospace" font-size="14" font-weight="600" fill="{title}">GitHub Traffic</text>
<line x1="25" y1="52" x2="{lx2}" y2="52" stroke="{sep}" stroke-width="1"/>
{stats}
<line x1="25" y1="113" x2="{lx2}" y2="113" stroke="{sep}" stroke-width="1"/>
{repos}
</svg>"#,
        bg = c.bg,
        border = c.border,
        title = c.title,
        sep = c.separator,
        lx2 = W - 15,
        stats = stats_svg,
        repos = repos_svg,
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
         <text x=\"290\" y\":\"96\" font-family=\"monospace\" font-size=\"11\" fill=\"{tl}\">Longest Streak</text>\
         <text x=\"25\" y\":\"128\" font-family=\"monospace\" font-size\":\"10\" fill=\"{tl}\">Best streak: {range}</text>",
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
