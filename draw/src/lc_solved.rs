use crate::{helpers::fmt_num, matrix, theme::Theme};
use readme_stuff_aggregator::widgets::LcSolvedWidget;

const W: u32 = 495;
const H: u32 = 175;

const LC_EASY: &str = "#00b8a3";
const LC_MEDIUM: &str = "#ffc01e";
const LC_HARD: &str = "#ff375f";
const LC_ORANGE: &str = "#ffa116";

pub fn render_lc_solved(w: &LcSolvedWidget, theme: Theme) -> String {
    let c = theme.colors();
    let rain = matrix::generate(W, H, c.matrix_color, c.matrix_opacity, 0x1C00_0001, "lcs");

    let total_str = fmt_num(w.total as u64);

    let total_shown = (w.easy + w.medium + w.hard).max(1);
    let bar_total_w: u32 = 420;
    let easy_w = (w.easy as f64 / total_shown as f64 * bar_total_w as f64) as u32;
    let med_w = (w.medium as f64 / total_shown as f64 * bar_total_w as f64) as u32;
    let hard_w = bar_total_w.saturating_sub(easy_w + med_w);

    let stats = [
        ("Easy", w.easy, LC_EASY),
        ("Medium", w.medium, LC_MEDIUM),
        ("Hard", w.hard, LC_HARD),
    ];
    let col_x = [25u32, 185, 345];

    let mut stat_svg = String::new();
    for (i, (label, count, color)) in stats.iter().enumerate() {
        let x = col_x[i];
        stat_svg.push_str(&format!(
            "<text x=\"{x}\" y=\"138\" font-family=\"'Segoe UI',Ubuntu,sans-serif\" font-size=\"20\" font-weight=\"700\" fill=\"{color}\">{val}</text>\
             <text x=\"{x}\" y=\"155\" font-family=\"'Segoe UI',Ubuntu,sans-serif\" font-size=\"11\" fill=\"{tl}\">{label}</text>",
            val = fmt_num(*count as u64),
            tl = c.text_secondary,
        ));
    }

    format!(
        r#"<svg width="{W}" height="{H}" viewBox="0 0 {W} {H}" xmlns="http://www.w3.org/2000/svg">
<defs>
  <clipPath id="lcs-clip">
    <rect width="{W}" height="{H}" rx="6"/>
  </clipPath>
</defs>
<rect width="{W}" height="{H}" rx="6" fill="{bg}"/>
<g clip-path="url(#lcs-clip)">{rain}</g>
<rect width="{W}" height="3" rx="1.5" fill="{lc_accent}"/>
<rect width="{W}" height="{H}" rx="6" fill="none" stroke="{border}" stroke-width="1"/>
<circle cx="25" cy="30" r="6" fill="{lc_accent}"/>
<text x="38" y="35" font-family="'Segoe UI',Ubuntu,sans-serif" font-size="14" font-weight="600" fill="{title}">LeetCode &#8212; Problems Solved</text>
<line x1="25" y1="52" x2="470" y2="52" stroke="{sep}" stroke-width="1"/>
<text x="25" y="88" font-family="'Segoe UI',Ubuntu,sans-serif" font-size="36" font-weight="700" fill="{tv}">{total}</text>
<text x="108" y="88" font-family="'Segoe UI',Ubuntu,sans-serif" font-size="14" fill="{ts}">solved</text>
<rect x="25" y="98" width="{easy_w}" height="6" rx="3" fill="{lc_easy}"/>
<rect x="{med_x}" y="98" width="{med_w}" height="6" fill="{lc_medium}"/>
<rect x="{hard_x}" y="98" width="{hard_w}" height="6" rx="3" fill="{lc_hard}"/>
<line x1="25" y1="113" x2="470" y2="113" stroke="{sep}" stroke-width="1"/>
{stat_svg}
</svg>"#,
        bg = c.bg,
        border = c.border,
        title = c.title,
        sep = c.separator,
        rain = rain,
        stat_svg = stat_svg,
        tv = c.text_primary,
        ts = c.text_secondary,
        lc_accent = LC_ORANGE,
        total = total_str,
        easy_w = easy_w,
        med_x = 25 + easy_w,
        med_w = med_w,
        hard_x = 25 + easy_w + med_w,
        hard_w = hard_w,
        lc_easy = LC_EASY,
        lc_medium = LC_MEDIUM,
        lc_hard = LC_HARD,
    )
}
