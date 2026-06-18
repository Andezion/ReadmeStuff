use crate::{helpers::{fmt_num, xml_escape}, matrix, theme::Theme};
use readme_stuff_aggregator::widgets::LcLanguagesWidget;

const W: u32 = 300;
const PAD: u32 = 15;
const BAR_X: u32 = 110;
const BAR_W: u32 = 130;
const PCT_X: u32 = W - PAD;
const ROW_H: u32 = 22;
const FIRST_ROW_Y: u32 = 60;
const MAX_ROWS: usize = 8;

const LC_LANG_COLORS: &[(&str, &str)] = &[
    ("C++", "#f34b7d"),
    ("C", "#555555"),
    ("C#", "#178600"),
    ("Rust", "#dea584"),
    ("Python", "#3572A5"),
    ("Java", "#b07219"),
    ("JavaScript", "#f1e05a"),
    ("TypeScript", "#3178c6"),
    ("Go", "#00ADD8"),
    ("Kotlin", "#A97BFF"),
];

fn lc_lang_color(name: &str) -> &'static str {
    LC_LANG_COLORS
        .iter()
        .find(|(n, _)| n.eq_ignore_ascii_case(name))
        .map(|(_, c)| *c)
        .unwrap_or("#8b949e")
}

pub fn render_lc_languages(w: &LcLanguagesWidget, theme: Theme) -> String {
    let c = theme.colors();
    let n = w.languages.len().min(MAX_ROWS);
    let h: u32 = FIRST_ROW_Y + n as u32 * ROW_H + 10;

    let rain = matrix::generate(W, h, c.matrix_color, c.matrix_opacity, 0x1C00_0003, "lcl");
    let max_solved = w.languages.iter().map(|l| l.solved).max().unwrap_or(1).max(1);

    let mut rows = String::new();
    for (i, entry) in w.languages.iter().take(MAX_ROWS).enumerate() {
        let y = FIRST_ROW_Y + i as u32 * ROW_H + ROW_H - 5;
        let bar_y = y - 11;
        let fill_w = (entry.solved as f64 / max_solved as f64 * BAR_W as f64).round() as u32;
        let color = lc_lang_color(&entry.name);

        rows.push_str(&format!(
            "<circle cx=\"{dot_x}\" cy=\"{dot_y}\" r=\"4\" fill=\"{color}\"/>\
             <text x=\"{name_x}\" y=\"{y}\" font-family=\"'Segoe UI',Ubuntu,sans-serif\" font-size=\"11\" fill=\"{tv}\">{name}</text>\
             <rect x=\"{BAR_X}\" y=\"{bar_y}\" width=\"{BAR_W}\" height=\"9\" rx=\"4\" fill=\"{bar_bg}\"/>\
             <rect x=\"{BAR_X}\" y=\"{bar_y}\" width=\"{fill_w}\" height=\"9\" rx=\"4\" fill=\"{color}\"/>\
             <text x=\"{PCT_X}\" y=\"{y}\" text-anchor=\"end\" font-family=\"'Segoe UI',Ubuntu,sans-serif\" font-size=\"11\" fill=\"{ts}\">{solved}</text>",
            dot_x = PAD + 4,
            dot_y = y - 4,
            name_x = PAD + 14,
            name = xml_escape(&entry.name),
            solved = fmt_num(entry.solved as u64),
            bar_bg = c.separator,
            tv = c.text_primary,
            ts = c.text_secondary,
        ));
    }

    format!(
        r#"<svg width="{W}" height="{h}" viewBox="0 0 {W} {h}" xmlns="http://www.w3.org/2000/svg">
<defs>
  <clipPath id="lcl-clip">
    <rect width="{W}" height="{h}" rx="6"/>
  </clipPath>
</defs>
<rect width="{W}" height="{h}" rx="6" fill="{bg}"/>
<g clip-path="url(#lcl-clip)">{rain}</g>
<rect width="{W}" height="3" rx="1.5" fill="{lc_accent}"/>
<rect width="{W}" height="{h}" rx="6" fill="none" stroke="{border}" stroke-width="1"/>
<circle cx="15" cy="30" r="6" fill="{lc_accent}"/>
<text x="28" y="35" font-family="'Segoe UI',Ubuntu,sans-serif" font-size="14" font-weight="600" fill="{title}">LeetCode Languages</text>
<line x1="15" y1="48" x2="285" y2="48" stroke="{sep}" stroke-width="1"/>
{rows}
</svg>"#,
        bg = c.bg, border = c.border, title = c.title,
        sep = c.separator, rain = rain, rows = rows,
        lc_accent = "#ffa116",
    )
}
