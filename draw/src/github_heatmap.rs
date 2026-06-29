use crate::{matrix, theme::Theme};
use readme_stuff_aggregator::widgets::GithubHeatmapWidget;

const W: u32 = 515;
const H: u32 = 214;
const LABEL_X: u32 = 25;
const BAR_X: u32 = 65;
const BAR_MAX_W: u32 = 370;
const COUNT_X: u32 = 465;
const ROW_H: u32 = 19;
const FIRST_Y: u32 = 70;

const DAYS: [&str; 7] = ["Sun", "Mon", "Tue", "Wed", "Thu", "Fri", "Sat"];

pub fn render_github_heatmap(w: &GithubHeatmapWidget, theme: Theme) -> String {
    let c = theme.colors();
    let rain = matrix::generate(W, H, c.matrix_color, c.matrix_opacity, 0x7788_EEFF, "ghm");

    let max_val = *w.weekday_distribution.iter().max().unwrap_or(&1).max(&1);

    let mut bars = String::new();
    for (i, &count) in w.weekday_distribution.iter().enumerate() {
        let y = FIRST_Y + i as u32 * ROW_H - 1;
        let fill_w = (count as f64 / max_val as f64 * BAR_MAX_W as f64).round() as u32;
        let bar_y = y - 11 + 2;

        bars.push_str(&format!(
            "<text x=\"{LABEL_X}\" y=\"{y}\" \
                font-family=\"monospace\" font-size=\"11\" \
                fill=\"{tl}\">{day}</text>\
             <rect x=\"{BAR_X}\" y=\"{bar_y}\" width=\"{BAR_MAX_W}\" height=\"10\" rx=\"5\" \
                fill=\"{bar_bg}\"/>\
             <rect x=\"{BAR_X}\" y=\"{bar_y}\" width=\"{fill_w}\" height=\"10\" rx=\"5\" \
                fill=\"{accent}\"/>\
             <text x=\"{COUNT_X}\" y=\"{y}\" text-anchor=\"end\" \
                font-family=\"monospace\" font-size=\"11\" \
                fill=\"{tv}\">{count}</text>",
            day = DAYS[i],
            bar_bg = c.separator,
            accent = c.accent,
            tv = c.text_primary,
            tl = c.text_secondary,
        ));
    }

    format!(
        r#"<svg width="{W}" height="{H}" viewBox="0 0 {W} {H}" xmlns="http://www.w3.org/2000/svg">
<defs>
  <clipPath id="ghm-clip">
    <rect width="{W}" height="{H}" rx="6"/>
  </clipPath>
</defs>
<rect width="{W}" height="{H}" rx="6" fill="{bg}"/>
<g clip-path="url(#ghm-clip)">{rain}</g>
<rect width="{W}" height="{H}" rx="6" fill="none" stroke="{border}" stroke-width="1"/>
<text x="25" y="35" font-family="monospace" font-size="14" font-weight="600" fill="{title}">Activity by Day of Week</text>
<line x1="25" y1="52" x2="470" y2="52" stroke="{sep}" stroke-width="1"/>
{bars}
</svg>"#,
        bg = c.bg,
        border = c.border,
        title = c.title,
        sep = c.separator,
        rain = rain,
        bars = bars,
    )
}
