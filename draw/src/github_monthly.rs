use crate::{matrix, theme::Theme};
use readme_stuff_aggregator::widgets::GithubMonthlyWidget;

const W: u32 = 495;
const H: u32 = 210;
const PAD: u32 = 25;
const BAR_AREA_TOP: u32 = 65;
const BAR_AREA_BOTTOM: u32 = 166;
const LABEL_Y: u32 = 181;

pub fn render_github_monthly(w: &GithubMonthlyWidget, theme: Theme) -> String {
    let c = theme.colors();
    let rain = matrix::generate(W, H, c.matrix_color, c.matrix_opacity, 0x9900_CC11, "gmo");

    let n = w.months.len().max(1) as u32;
    let available_w = W - 2 * PAD;
    let total_gap: u32 = 3 * (n - 1);
    let bar_w = (available_w - total_gap) / n;
    let bar_area_h = BAR_AREA_BOTTOM - BAR_AREA_TOP;
    let max_count = w.months.iter().map(|(_, c)| *c).max().unwrap_or(1).max(1);

    let mut bars = String::new();
    for (i, (label, count)) in w.months.iter().enumerate() {
        let x = PAD + i as u32 * (bar_w + 3);
        let bar_h = (*count as f64 / max_count as f64 * bar_area_h as f64).round() as u32;
        let bar_y = BAR_AREA_BOTTOM - bar_h;

        bars.push_str(&format!(
            "<rect x=\"{x}\" y=\"{BAR_AREA_TOP}\" width=\"{bar_w}\" height=\"{bar_area_h}\" rx=\"3\" fill=\"{bg_track}\"/>",
            bg_track = c.separator,
        ));
        if bar_h > 0 {
            bars.push_str(&format!(
                "<rect x=\"{x}\" y=\"{bar_y}\" width=\"{bar_w}\" height=\"{bar_h}\" rx=\"3\" fill=\"{accent}\"/>",
                accent = c.accent,
            ));
        }
        let label_x = x + bar_w / 2;
        bars.push_str(&format!(
            "<text x=\"{label_x}\" y=\"{LABEL_Y}\" text-anchor=\"middle\" \
                font-family=\"monospace\" font-size=\"9\" \
                fill=\"{ts}\">{label}</text>",
            ts = c.text_secondary,
        ));
        if bar_h >= 14 {
            let val_y = bar_y + 11;
            bars.push_str(&format!(
                "<text x=\"{label_x}\" y=\"{val_y}\" text-anchor=\"middle\" \
                    font-family=\"monospace\" font-size=\"9\" \
                    fill=\"{bg}\">{count}</text>",
                bg = c.bg,
            ));
        }
    }

    format!(
        r#"<svg width="{W}" height="{H}" viewBox="0 0 {W} {H}" xmlns="http://www.w3.org/2000/svg">
<defs>
  <clipPath id="gmo-clip">
    <rect width="{W}" height="{H}" rx="6"/>
  </clipPath>
</defs>
<rect width="{W}" height="{H}" rx="6" fill="{bg}"/>
<g clip-path="url(#gmo-clip)">{rain}</g>
<rect width="{W}" height="{H}" rx="6" fill="none" stroke="{border}" stroke-width="1"/>
<text x="25" y="35" font-family="monospace" font-size="14" font-weight="600" fill="{title}">Monthly Contributions</text>
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
