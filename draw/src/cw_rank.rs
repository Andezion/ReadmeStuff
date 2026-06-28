use crate::{
    helpers::{cw_color, fmt_num, xml_escape},
    matrix,
    theme::Theme,
};
use readme_stuff_aggregator::widgets::CwRankWidget;

const W: u32 = 495;
const H: u32 = 165;

pub fn render_cw_rank(w: &CwRankWidget, theme: Theme) -> String {
    let c = theme.colors();
    let rain = matrix::generate(W, H, c.matrix_color, c.matrix_opacity, 0xCCCC_0101, "cwr");

    let kyu_css_color = cw_color(&w.rank_color);

    let stats: [(&str, String); 3] = [
        ("Honor", fmt_num(w.honor as u64)),
        ("Score", fmt_num(w.score as u64)),
        (
            "Leaderboard",
            w.leaderboard_position
                .map(|p| format!("#{}", fmt_num(p as u64)))
                .unwrap_or_else(|| "—".into()),
        ),
    ];
    let col_x = [185u32, 305, 395];

    let mut stat_svg = String::new();
    for (i, (label, value)) in stats.iter().enumerate() {
        let x = col_x[i];
        stat_svg.push_str(&format!(
            "<text x=\"{x}\" y=\"98\" font-family=\"monospace\" font-size=\"18\" font-weight=\"700\" fill=\"{tv}\">{value}</text>\
             <text x=\"{x}\" y=\"116\" font-family=\"monospace\" font-size=\"11\" fill=\"{tl}\">{label}</text>",
            tv = c.text_primary, tl = c.text_secondary,
        ));
    }

    let clan_svg = if let Some(clan) = &w.clan {
        format!(
            "<text x=\"25\" y=\"145\" font-family=\"monospace\" font-size=\"11\" fill=\"{ts}\">clan: {clan}</text>",
            ts = c.text_secondary,
            clan = xml_escape(clan),
        )
    } else {
        String::new()
    };

    format!(
        r#"<svg width="{W}" height="{H}" viewBox="0 0 {W} {H}" xmlns="http://www.w3.org/2000/svg">
<defs>
  <clipPath id="cwr-clip">
    <rect width="{W}" height="{H}" rx="6"/>
  </clipPath>
</defs>
<rect width="{W}" height="{H}" rx="6" fill="{bg}"/>
<g clip-path="url(#cwr-clip)">{rain}</g>
<rect width="{W}" height="{H}" rx="6" fill="none" stroke="{border}" stroke-width="1"/>
<text x="25" y="35" font-family="monospace" font-size="14" font-weight="600" fill="{title}">Codewars Rank</text>
<line x1="25" y1="52" x2="470" y2="52" stroke="{sep}" stroke-width="1"/>
<line x1="137" y1="58" x2="137" y2="155" stroke="{sep}" stroke-width="1"/>
<line x1="247" y1="58" x2="247" y2="155" stroke="{sep}" stroke-width="1"/>
<line x1="337" y1="58" x2="337" y2="155" stroke="{sep}" stroke-width="1"/>
<text x="25" y="100" font-family="monospace" font-size="32" font-weight="700" fill="{kyu_color}">{rank}</text>
<text x="25" y="120" font-family="monospace " font-size="12" fill="{kyu_color}">Codewars</text>
{stat_svg}
{clan_svg}
</svg>"#,
        bg = c.bg,
        border = c.border,
        title = c.title,
        sep = c.separator,
        rain = rain,
        kyu_color = kyu_css_color,
        rank = xml_escape(&w.rank_name),
        stat_svg = stat_svg,
        clan_svg = clan_svg,
    )
}
