use crate::{
    helpers::{cf_rank_color, fmt_num, xml_escape},
    matrix,
    theme::Theme,
};
use readme_stuff_aggregator::widgets::CfRatingWidget;

const W: u32 = 495;
const H: u32 = 165;

pub fn render_cf_rating(w: &CfRatingWidget, theme: Theme) -> String {
    let c = theme.colors();
    let rain = matrix::generate(W, H, c.matrix_color, c.matrix_opacity, 0xCF_1234, "cfr");

    let rank_color = cf_rank_color(&w.rank);
    let max_rank_color = cf_rank_color(&w.max_rank);

    let rating_str = fmt_num(w.rating.unsigned_abs() as u64);
    let max_rating_str = fmt_num(w.max_rating.unsigned_abs() as u64);

    let stats: [(&str, String, &str); 3] = [
        ("Current Rating", rating_str.clone(), rank_color),
        ("Max Rating", max_rating_str, max_rank_color),
        ("Contests", w.contest_count.to_string(), c.text_primary),
    ];
    let col_x = [25u32, 185, 345];

    let mut stat_svg = String::new();
    for (i, (label, value, color)) in stats.iter().enumerate() {
        let x = col_x[i];
        stat_svg.push_str(&format!(
            "<text x=\"{x}\" y=\"98\" \
                font-family=\"monospace\" font-size=\"24\" font-weight=\"700\" \
                fill=\"{color}\">{value}</text>\
             <text x=\"{x}\" y=\"117\" \
                font-family=\"monospace\" font-size=\"11\" \
                fill=\"{tl}\">{label}</text>",
            tl = c.text_secondary,
        ));
    }

    stat_svg.push_str(&format!(
        "<text x=\"25\" y=\"136\" \
            font-family=\"monospace\" font-size=\"12\" font-weight=\"600\" \
            fill=\"{rank_color}\">{rank}</text>\
         <text x=\"185\" y=\"136\" \
            font-family=\"monospace\" font-size=\"12\" font-weight=\"600\" \
            fill=\"{max_rank_color}\">{max_rank}</text>",
        rank = xml_escape(&w.rank),
        max_rank = xml_escape(&w.max_rank),
    ));

    format!(
        r#"<svg width="{W}" height="{H}" viewBox="0 0 {W} {H}" xmlns="http://www.w3.org/2000/svg">
<defs>
  <clipPath id="cfr-clip">
    <rect width="{W}" height="{H}" rx="6"/>
  </clipPath>
</defs>
<rect width="{W}" height="{H}" rx="6" fill="{bg}"/>
<g clip-path="url(#cfr-clip)">{rain}</g>
<rect width="{W}" height="{H}" rx="6" fill="none" stroke="{border}" stroke-width="1"/>
<text x="38" y="35" font-family="monospace" font-size="14" font-weight="600" fill="{title}">My Codeforces Rating</text>
<line x1="25" y1="52" x2="470" y2="52" stroke="{sep}" stroke-width="1"/>
<line x1="177" y1="58" x2="177" y2="155" stroke="{sep}" stroke-width="1"/>
<line x1="337" y1="58" x2="337" y2="155" stroke="{sep}" stroke-width="1"/>
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
