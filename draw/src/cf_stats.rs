use crate::{helpers::fmt_num, matrix, theme::Theme};
use readme_stuff_aggregator::widgets::CfStatsWidget;

const W: u32 = 495;
const H: u32 = 120;

pub fn render_cf_stats(w: &CfStatsWidget, theme: Theme) -> String {
    let c = theme.colors();
    let rain = matrix::generate(W, H, c.matrix_color, c.matrix_opacity, 0xCF_5678, "cfs");

    let contribution_str = if w.contribution >= 0 {
        format!("+{}", w.contribution)
    } else {
        w.contribution.to_string()
    };

    let stats: [(&str, String); 4] = [
        ("Problems Solved", fmt_num(w.problems_solved as u64)),
        ("Contests", w.contest_count.to_string()),
        ("Friends", fmt_num(w.friend_of_count as u64)),
        ("Contribution", contribution_str),
    ];
    let col_x = [25u32, 145, 265, 385];

    let mut stat_svg = String::new();
    for (i, (label, value)) in stats.iter().enumerate() {
        let x = col_x[i];
        stat_svg.push_str(&format!(
            "<text x=\"{x}\" y=\"88\" font-family=\"monospace\" font-size=\"18\" font-weight=\"700\" fill=\"{tv}\">{value}</text>\
             <text x=\"{x}\" y=\"104\" font-family=\"monospace\" font-size=\"11\" fill=\"{tl}\">{label}</text>",
            tv = c.text_primary, tl = c.text_secondary,
        ));
    }

    format!(
        r#"<svg width="{W}" height="{H}" viewBox="0 0 {W} {H}" xmlns="http://www.w3.org/2000/svg">
<defs>
  <clipPath id="cfs-clip">
    <rect width="{W}" height="{H}" rx="6"/>
  </clipPath>
</defs>
<rect width="{W}" height="{H}" rx="6" fill="{bg}"/>
<g clip-path="url(#cfs-clip)">{rain}</g>
<rect width="{W}" height="{H}" rx="6" fill="none" stroke="{border}" stroke-width="1"/>
<text x="38" y="35" font-family="monospace" font-size="14" font-weight="600" fill="{title}">Codeforces Stats</text>
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
