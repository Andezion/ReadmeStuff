use crate::{
    helpers::{fmt_num, rank_color, xml_escape},
    matrix,
    theme::Theme,
};
use readme_stuff_aggregator::widgets::GithubStatsWidget;

const W: u32 = 495;
const H: u32 = 195;

// GitHub mark icon (16x16 viewbox)
const GH_ICON: &str = "M8 0C3.58 0 0 3.58 0 8c0 3.54 2.29 6.53 5.47 \
    7.59.4.07.55-.17.55-.38 0-.19-.01-.82-.01-1.49-2.01.37-2.53-.49-2.69-.94-.09-.23-.48-.94-.82-1.13\
    -.28-.15-.68-.52-.01-.53.63-.01 1.08.58 1.23.82.72 1.21 1.87.87 2.33.66.07-.52.28-.87.51-1.07\
    -1.78-.2-3.64-.89-3.64-3.95 0-.87.31-1.59.82-2.15-.08-.2-.36-1.02.08-2.12 0 0 .67-.21 2.2.82\
    .64-.18 1.32-.27 2-.27.68 0 1.36.09 2 .27 1.53-1.04 2.2-.82 2.2-.82.44 1.1.16 1.92.08 2.12\
    .51.56.82 1.27.82 2.15 0 3.07-1.87 3.75-3.65 3.95.29.25.54.73.54 1.48 0 1.07-.01 1.93-.01 2.2\
    0 .21.15.46.55.38A8.013 8.013 0 0016 8c0-4.42-3.58-8-8-8z";

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
                font-family=\"'Segoe UI',Ubuntu,sans-serif\" font-size=\"11\" \
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
<path transform="translate(25,16)" fill="{title}" d="{GH_ICON}"/>
<text x="47" y="30" font-family="'Segoe UI',Ubuntu,sans-serif" font-size="14" font-weight="600" fill="{title}">{dname}</text>
<text x="47" y="47" font-family="'Segoe UI',Ubuntu,sans-serif" font-size="11" fill="{text2}">@{login}</text>
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
        GH_ICON = GH_ICON,
        rain = rain,
        stat_svg = stat_svg,
    )
}
