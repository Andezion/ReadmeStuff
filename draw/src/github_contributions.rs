use crate::{helpers::fmt_num, matrix, theme::Theme};
use readme_stuff_aggregator::widgets::GithubContributionsWidget;

const W: u32 = 495;
const H: u32 = 175;

pub fn render_github_contributions(w: &GithubContributionsWidget, theme: Theme) -> String {
    let c = theme.colors();
    let rain = matrix::generate(W, H, c.matrix_color, c.matrix_opacity, 0x2233_AABB, "gc");

    let col_x = [25u32, 181, 337];
    let row1: [(&str, String); 3] = [
        ("Total Commits", fmt_num(w.total_commits as u64)),
        ("Pull Requests", fmt_num(w.total_prs as u64)),
        ("Issues", fmt_num(w.total_issues as u64)),
    ];
    let row2: [(&str, String); 2] = [
        ("PR Reviews", fmt_num(w.total_reviews as u64)),
        ("Repos Contributed", fmt_num(w.repos_contributed_to as u64)),
    ];

    let mut stat_svg = String::new();
    for (i, (label, value)) in row1.iter().enumerate() {
        let x = col_x[i];
        stat_svg.push_str(&format!(
            "<text x=\"{x}\" y=\"90\" font-family=\"monospace\" font-size=\"18\" font-weight=\"700\" fill=\"{tv}\">{value}</text>\
             <text x=\"{x}\" y=\"106\" font-family=\"monospace\" font-size=\"11\" fill=\"{tl}\">{label}</text>",
            tv = c.text_primary, tl = c.text_secondary,
        ));
    }
    for (i, (label, value)) in row2.iter().enumerate() {
        let x = col_x[i];
        stat_svg.push_str(&format!(
            "<text x=\"{x}\" y=\"136\" font-family=\"monospace\" font-size=\"18\" font-weight=\"700\" fill=\"{tv}\">{value}</text>\
             <text x=\"{x}\" y=\"150\" font-family=\"monospace\" font-size=\"11\" fill=\"{tl}\">{label}</text>",
            tv = c.text_primary, tl = c.text_secondary,
        ));
    }

    format!(
        r#"<svg width="{W}" height="{H}" viewBox="0 0 {W} {H}" xmlns="http://www.w3.org/2000/svg">
<defs>
  <clipPath id="gc-clip">
    <rect width="{W}" height="{H}" rx="6"/>
  </clipPath>
</defs>
<rect width="{W}" height="{H}" rx="6" fill="{bg}"/>
<g clip-path="url(#gc-clip)">{rain}</g>
<rect width="{W}" height="{H}" rx="6" fill="none" stroke="{border}" stroke-width="1"/>
<text x="25" y="35" font-family="monospace" font-size="14" font-weight="600" fill="{title}">Contributions (This Year)</text>
<line x1="25" y1="52" x2="470" y2="52" stroke="{sep}" stroke-width="1"/>
{stat_svg}
</svg>"#,
        bg = c.bg,
        border = c.border,
        title = c.title,
        sep = c.separator,
        rain = rain,
        stat_svg = stat_svg,
    )
}
