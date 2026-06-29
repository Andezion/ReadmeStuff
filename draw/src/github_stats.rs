use crate::{
    helpers::{fmt_num, rank_color, xml_escape},
    matrix,
    theme::Theme,
};
use readme_stuff_aggregator::widgets::GithubStatsWidget;

const W: u32 = 495;
const H: u32 = 195;



pub fn render_github_stats(w: &GithubStatsWidget, theme: Theme) -> String {
    let c = theme.colors();
    let rain = matrix::generate(W, H, c.matrix_color, c.matrix_opacity, 0x1337_DEAD, "gs");
    let rcolor = rank_color(&w.rank);
    let display_name = w.name.as_deref().unwrap_or(&w.login);

    let col_x = [25u32, 181, 337];

    let stats: [(&str, String); 6] = [
        ("Total Commits", fmt_num(w.commits as u64)),
        ("Pull Requests", fmt_num(w.prs as u64)),
        ("Issues", fmt_num(w.issues as u64)),
        ("Total Stars", fmt_num(w.stars)),
        ("Followers", fmt_num(w.followers as u64)),
        ("Top", format!("{:.1}%", w.rank_percentile * 100.0)),
    ];

    let mut stat_svg = String::new();
    for (idx, (label, value)) in stats.iter().enumerate() {
        let x = col_x[idx % 3];
        let val_y = if idx < 3 { 92u32 } else { 152 };
        let lbl_y = if idx < 3 { 108u32 } else { 168 };
        stat_svg.push_str(&format!(
            "<text x=\"{x}\" y=\"{val_y}\" \
                font-family=\"'Segoe UI',Ubuntu,sans-serif\" font-size=\"18\" font-weight=\"700\" \
                fill=\"{tv}\">{value}</text>\
             <text x=\"{x}\" y=\"{lbl_y}\" \
                font-family=\"monospace\" font-size=\"11\" \
                fill=\"{tl}\">{label}</text>",
            tv = c.text_primary,
            tl = c.text_secondary,
        ));
    }

    format!(
        r#"<svg width="{W}" height="{H}" viewBox="0 0 {W} {H}" xmlns="http://www.w3.org/2000/svg">
<defs>
  <clipPath id="gs-clip">
    <rect width="{W}" height="{H}" rx="6"/>
  </clipPath>
</defs>
<rect width="{W}" height="{H}" rx="6" fill="{bg}"/>
<g clip-path="url(#gs-clip)">{rain}</g>
<rect width="{W}" height="3" rx="1.5" fill="{accent}"/>
<rect width="{W}" height="{H}" rx="6" fill="none" stroke="{border}" stroke-width="1"/>
<text x="25" y="30" font-family="monospace" font-size="14" font-weight="600" fill="{title}">{dname}</text>
<text x="25" y="47" font-family="monospace" font-size="11" fill="{text2}">@{login}</text>
<g transform="translate(456,35)">
  <circle r="22" fill="{bg}" stroke="{rcolor}" stroke-width="2.5"/>
  <text text-anchor="middle" dy="6" font-family="'Segoe UI',Ubuntu,sans-serif" font-size="15" font-weight="700" fill="{rcolor}">{rank}</text>
</g>
<line x1="25" y1="62" x2="430" y2="62" stroke="{sep}" stroke-width="1"/>
{stat_svg}
</svg>"#,
        W = W,
        H = H,
        bg = c.bg,
        border = c.border,
        accent = c.accent,
        title = c.title,
        text2 = c.text_secondary,
        sep = c.separator,
        rcolor = rcolor,
        rank = xml_escape(&w.rank),
        dname = xml_escape(display_name),
        login = xml_escape(&w.login),
        rain = rain,
        stat_svg = stat_svg,
    )
}
