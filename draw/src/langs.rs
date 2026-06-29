use crate::{helpers::xml_escape, matrix, theme::Theme};
use readme_stuff_aggregator::widgets::LangsWidget;

const W: u32 = 300;
const H: u32 = 205;
const PAD: u32 = 15;
const BAR_X: u32 = 110;
const BAR_W: u32 = 145;
const PCT_X: u32 = W - PAD;

pub fn render_langs(w: &LangsWidget, theme: Theme) -> String {
    let c = theme.colors();
    let rain = matrix::generate(W, H, c.matrix_color, c.matrix_opacity, 0xFACE_FEED, "ln");

    // Row baseline positions for up to 6 languages
    let row_y: [u32; 6] = [75, 99, 123, 147, 171, 195];

    let mut bars = String::new();
    for (i, lang) in w.top.iter().take(6).enumerate() {
        let y = row_y[i];
        let color = lang.color.as_deref().unwrap_or(c.accent);
        let fill_w = ((lang.percentage / 100.0) * BAR_W as f64).round() as u32;
        let bar_y = y - 11;

        bars.push_str(&format!(
            "<circle cx=\"{dot_x}\" cy=\"{dot_y}\" r=\"4\" fill=\"{color}\"/>\
             <text x=\"{name_x}\" y=\"{y}\" \
                font-family=\"monospace\" font-size=\"11\" \
                fill=\"{tv}\">{name}</text>\
             <rect x=\"{BAR_X}\" y=\"{bar_y}\" width=\"{BAR_W}\" height=\"9\" rx=\"4.5\" \
                fill=\"{bar_bg}\"/>\
             <rect x=\"{BAR_X}\" y=\"{bar_y}\" width=\"{fill_w}\" height=\"9\" rx=\"4.5\" \
                fill=\"{color}\"/>\
             <text x=\"{PCT_X}\" y=\"{y}\" text-anchor=\"end\" \
                font-family=\"monospace\" font-size=\"11\" \
                fill=\"{ts}\">{pct:.1}%</text>",
            dot_x = PAD + 4,
            dot_y = y - 4,
            name_x = PAD + 14,
            tv = c.text_primary,
            ts = c.text_secondary,
            bar_bg = c.separator,
            name = xml_escape(&lang.name),
            pct = lang.percentage,
        ));
    }

    format!(
        r#"<svg width="{W}" height="{H}" viewBox="0 0 {W} {H}" xmlns="http://www.w3.org/2000/svg">
<defs>
  <clipPath id="ln-clip">
    <rect width="{W}" height="{H}" rx="6"/>
  </clipPath>
</defs>
<rect width="{W}" height="{H}" rx="6" fill="{bg}"/>
<g clip-path="url(#ln-clip)">{rain}</g>
<rect width="{W}" height="{H}" rx="6" fill="none" stroke="{border}" stroke-width="1"/>
<text x="28" y="35" font-family="monospace" font-size="14" font-weight="600" fill="{title}">Top Languages</text>
<line x1="15" y1="52" x2="285" y2="52" stroke="{sep}" stroke-width="1"/>
{bars}
</svg>"#,
        W = W,
        H = H,
        bg = c.bg,
        border = c.border,
        title = c.title,
        sep = c.separator,
        rain = rain,
        bars = bars,
    )
}
