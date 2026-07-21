use crate::{
    helpers::{fmt_num, xml_escape},
    matrix,
    theme::Theme,
};
use readme_stuff_aggregator::widgets::LcBadgesWidget;

const W: u32 = 495;
const ROW_H: u32 = 22;
const FIRST_ROW_Y: u32 = 68;
const MAX_ROWS: usize = 8;
pub const SIZE: (u32, u32) = (W, FIRST_ROW_Y + MAX_ROWS as u32 * ROW_H + 10);

pub fn render_lc_badges(w: &LcBadgesWidget, theme: Theme) -> String {
    let c = theme.colors();
    let h: u32 = FIRST_ROW_Y + MAX_ROWS as u32 * ROW_H + 10;

    let rain = matrix::generate(W, h, c.matrix_color, c.matrix_opacity, 0x1C00_0004, "lcb");

    let total_str = fmt_num(w.total as u64);

    let mut rows = String::new();
    for (i, badge) in w.badges.iter().take(MAX_ROWS).enumerate() {
        let y = FIRST_ROW_Y + i as u32 * ROW_H + ROW_H - 5;
        let stripe_color = if i % 2 == 1 { c.separator } else { "none" };

        rows.push_str(&format!(
            "<rect x=\"20\" y=\"{row_y}\" width=\"455\" height=\"{ROW_H}\" fill=\"{stripe_color}\" rx=\"2\"/>\
             <circle cx=\"32\" cy=\"{dot_y}\" r=\"3\" fill=\"#2fde1b\"/>\
             <text x=\"44\" y=\"{y}\" font-family=\"monospace\" font-size=\"12\" fill=\"{tv}\">{name}</text>\
             <text x=\"440\" y=\"{y}\" text-anchor=\"end\" font-family=\"monospace\" font-size=\"11\" fill=\"{ts}\">{date}</text>",
            row_y = FIRST_ROW_Y + i as u32 * ROW_H,
            dot_y = FIRST_ROW_Y + i as u32 * ROW_H + ROW_H / 2 + 1,
            name = xml_escape(&badge.name),
            date = xml_escape(&badge.date),
            tv = c.text_primary,
            ts = c.text_secondary,
        ));
    }

    format!(
        r#"<svg width="{W}" height="{h}" viewBox="0 0 {W} {h}" xmlns="http://www.w3.org/2000/svg">
<defs>
  <clipPath id="lcb-clip">
    <rect width="{W}" height="{h}" rx="6"/>
  </clipPath>
</defs>
<rect width="{W}" height="{h}" rx="6" fill="{bg}"/>
<g clip-path="url(#lcb-clip)">{rain}</g>
<rect width="{W}" height="{h}" rx="6" fill="none" stroke="{border}" stroke-width="1"/>
<text x="25" y="35" font-family="monospace" font-size="14" font-weight="600" fill="{title}">LeetCode Badges</text>
<text x="440" y="35" text-anchor="end" font-family="monospace" font-size="12" fill="{ts}">{total} total</text>
<line x1="25" y1="48" x2="470" y2="48" stroke="{sep}" stroke-width="1"/>
<text x="44" y="63" font-family="monospace" font-size="10" fill="{ts}">BADGE</text>
<text x="440" y="63" text-anchor="end" font-family="monospace" font-size="10" fill="{ts}">DATE</text>
{rows}
</svg>"#,
        bg = c.bg,
        border = c.border,
        title = c.title,
        sep = c.separator,
        ts = c.text_secondary,
        rain = rain,
        rows = rows,
        total = total_str,
    )
}
