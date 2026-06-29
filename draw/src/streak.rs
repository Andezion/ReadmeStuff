use crate::{helpers::fmt_num, matrix, theme::Theme};
use readme_stuff_aggregator::widgets::StreakWidget;

const W: u32 = 495;
const H: u32 = 160;

pub fn render_streak(w: &StreakWidget, theme: Theme) -> String {
    let c = theme.colors();
    let rain = matrix::generate(W, H, c.matrix_color, c.matrix_opacity, 0xBEEF_1337, "sk");

    let cols: [(&str, String); 3] = [
        ("Current Streak", format!("{} days", w.current_streak)),
        ("Longest Streak", format!("{} days", w.longest_streak)),
        ("Total Contributions", fmt_num(w.total_contributions as u64)),
    ];
    let col_x = [25u32, 181, 337];

    let mut stat_svg = String::new();
    for (i, (label, value)) in cols.iter().enumerate() {
        let x = col_x[i];
        stat_svg.push_str(&format!(
            "<text x=\"{x}\" y=\"98\" \
                font-family=\"monospace\" font-size=\"20\" font-weight=\"700\" \
                fill=\"{tv}\">{value}</text>\
             <text x=\"{x}\" y=\"115\" \
                font-family=\"monospace\" font-size=\"11\" \
                fill=\"{tl}\">{label}</text>",
            tv = c.text_primary,
            tl = c.text_secondary,
        ));
    }

    let avg = format!("avg {:.1} contributions / day", w.average_daily);

    format!(
        r#"<svg width="{W}" height="{H}" viewBox="0 0 {W} {H}" xmlns="http://www.w3.org/2000/svg">
<defs>
  <clipPath id="sk-clip">
    <rect width="{W}" height="{H}" rx="6"/>
  </clipPath>
</defs>
<rect width="{W}" height="{H}" rx="6" fill="{bg}"/>
<g clip-path="url(#sk-clip)">{rain}</g>
<rect width="{W}" height="{H}" rx="6" fill="none" stroke="{border}" stroke-width="1"/>
<text x="25" y="35" font-family="monospace" font-size="14" font-weight="600" fill="{title}">GitHub Streak</text>
<line x1="25" y1="52" x2="470" y2="52" stroke="{sep}" stroke-width="1"/>
{stat_svg}
<text x="25" y="143" font-family="monospace" font-size="11" fill="{text2}">{avg}</text>
</svg>"#,
        W = W,
        H = H,
        bg = c.bg,
        border = c.border,
        title = c.title,
        text2 = c.text_secondary,
        sep = c.separator,
        rain = rain,
        stat_svg = stat_svg,
        avg = avg,
    )
}
