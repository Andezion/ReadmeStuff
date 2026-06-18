use crate::{
    helpers::{cw_color, fmt_num, xml_escape},
    matrix,
    theme::Theme,
};
use readme_stuff_aggregator::widgets::CwLanguagesWidget;

const W: u32 = 495;
const ROW_H: u32 = 22;
const FIRST_ROW_Y: u32 = 72;
const MAX_ROWS: usize = 8;

pub fn render_cw_languages(w: &CwLanguagesWidget, theme: Theme) -> String {
    let c = theme.colors();
    let n = w.languages.len().min(MAX_ROWS);
    let h: u32 = FIRST_ROW_Y + n as u32 * ROW_H + 10;

    let rain = matrix::generate(W, h, c.matrix_color, c.matrix_opacity, 0xCCCC_0303, "cwl");

    let max_score = w
        .languages
        .iter()
        .map(|l| l.score)
        .max()
        .unwrap_or(1)
        .max(1);
    const BAR_X: u32 = 220;
    const BAR_MAX_W: u32 = 175;
    const SCORE_X: u32 = 400;
    const RANK_X: u32 = 130;

    let mut rows = String::new();
    for (i, entry) in w.languages.iter().take(MAX_ROWS).enumerate() {
        let y = FIRST_ROW_Y + i as u32 * ROW_H + ROW_H - 5;
        let bar_y = y - 11;
        let fill_w = (entry.score as f64 / max_score as f64 * BAR_MAX_W as f64).round() as u32;
        let kyu_css = cw_color(&entry.rank_color);

        rows.push_str(&format!(
            "<text x=\"25\" y=\"{y}\" font-family=\"'Segoe UI',Ubuntu,sans-serif\" font-size=\"12\" fill=\"{tv}\">{lang}</text>\
             <text x=\"{RANK_X}\" y=\"{y}\" font-family=\"'Segoe UI',Ubuntu,sans-serif\" font-size=\"11\" font-weight=\"600\" fill=\"{kyu_css}\">{rank}</text>\
             <rect x=\"{BAR_X}\" y=\"{bar_y}\" width=\"{BAR_MAX_W}\" height=\"9\" rx=\"4\" fill=\"{bar_bg}\"/>\
             <rect x=\"{BAR_X}\" y=\"{bar_y}\" width=\"{fill_w}\" height=\"9\" rx=\"4\" fill=\"{kyu_css}\"/>\
             <text x=\"{SCORE_X}\" y=\"{y}\" font-family=\"'Segoe UI',Ubuntu,sans-serif\" font-size=\"11\" fill=\"{ts}\">{score}</text>",
            lang = xml_escape(&entry.lang),
            rank = xml_escape(&entry.rank_name),
            score = fmt_num(entry.score as u64),
            bar_bg = c.separator,
            tv = c.text_primary,
            ts = c.text_secondary,
        ));
    }

    format!(
        r#"<svg width="{W}" height="{h}" viewBox="0 0 {W} {h}" xmlns="http://www.w3.org/2000/svg">
<defs>
  <clipPath id="cwl-clip">
    <rect width="{W}" height="{h}" rx="6"/>
  </clipPath>
</defs>
<rect width="{W}" height="{h}" rx="6" fill="{bg}"/>
<g clip-path="url(#cwl-clip)">{rain}</g>
<rect width="{W}" height="3" rx="1.5" fill="{accent}"/>
<rect width="{W}" height="{h}" rx="6" fill="none" stroke="{border}" stroke-width="1"/>
<circle cx="25" cy="30" r="6" fill="{accent}"/>
<text x="38" y="35" font-family="'Segoe UI',Ubuntu,sans-serif" font-size="14" font-weight="600" fill="{title}">Codewars by Language</text>
<line x1="25" y1="52" x2="470" y2="52" stroke="{sep}" stroke-width="1"/>
<text x="25" y="67" font-family="'Segoe UI',Ubuntu,sans-serif" font-size="10" fill="{ts}">LANGUAGE</text>
<text x="130" y="67" font-family="'Segoe UI',Ubuntu,sans-serif" font-size="10" fill="{ts}">RANK</text>
<text x="220" y="67" font-family="'Segoe UI',Ubuntu,sans-serif" font-size="10" fill="{ts}">SCORE</text>
{rows}
</svg>"#,
        bg = c.bg,
        border = c.border,
        accent = c.accent,
        title = c.title,
        sep = c.separator,
        ts = c.text_secondary,
        rain = rain,
        rows = rows,
    )
}
