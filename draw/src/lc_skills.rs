use crate::{helpers::xml_escape, matrix, theme::Theme};
use readme_stuff_aggregator::widgets::LcSkillsWidget;

const W: u32 = 495;
const PAD_X: u32 = 25;
const BAR_X: u32 = 260;
const BAR_MAX_W: u32 = 160;
const COUNT_X: u32 = 452;
const ROW_H: u32 = 18;
const FIRST_Y: u32 = 70;

const CAT_COLORS: [(&str, &str); 3] = [("adv", "#ff6e96"), ("int", "#58a6ff"), ("fun", "#3fb950")];

fn cat_color(cat: &str) -> &'static str {
    CAT_COLORS
        .iter()
        .find(|(c, _)| *c == cat)
        .map(|(_, col)| *col)
        .unwrap_or("#8b949e")
}

pub fn render_lc_skills(w: &LcSkillsWidget, theme: Theme) -> String {
    let c = theme.colors();
    let n = w.skills.len() as u32;
    let h: u32 = FIRST_Y + n * ROW_H + 15;

    let rain = matrix::generate(W, h, c.matrix_color, c.matrix_opacity, 0x1C00_0002, "lcsk");
    let max_amount = w.skills.iter().map(|s| s.amount).max().unwrap_or(1).max(1);

    let mut rows = String::new();
    for (i, skill) in w.skills.iter().enumerate() {
        let y = FIRST_Y + i as u32 * ROW_H + ROW_H - 4;
        let bar_y = y - 9;
        let fill_w = (skill.amount as f64 / max_amount as f64 * BAR_MAX_W as f64).round() as u32;
        let color = cat_color(skill.category);

        rows.push_str(&format!(
            "<text x=\"{PAD_X}\" y=\"{y}\" font-family=\"monospace\" font-size=\"11\" fill=\"{tv}\">{name}</text>\
             <text x=\"{cat_x}\" y=\"{y}\" font-family=\"monospace\" font-size=\"9\" fill=\"{color}\">{cat}</text>\
             <rect x=\"{BAR_X}\" y=\"{bar_y}\" width=\"{BAR_MAX_W}\" height=\"9\" rx=\"4\" fill=\"{bar_bg}\"/>\
             <rect x=\"{BAR_X}\" y=\"{bar_y}\" width=\"{fill_w}\" height=\"9\" rx=\"4\" fill=\"{color}\"/>\
             <text x=\"{COUNT_X}\" y=\"{y}\" text-anchor=\"end\" font-family=\"monospace\" font-size=\"11\" fill=\"{ts}\">{amount}</text>",
            name = xml_escape(&skill.name),
            cat = skill.category.to_uppercase(),
            cat_x = PAD_X + 200,
            bar_bg = c.separator,
            tv = c.text_primary,
            ts = c.text_secondary,
            amount = skill.amount,
        ));
    }

    let legend = CAT_COLORS.iter().enumerate().map(|(i, (cat, color))| {
        let lx = 25 + i as u32 * 110;
        let label = match *cat { "adv" => "Advanced", "int" => "Intermediate", _ => "Fundamental" };
        format!(
            "<circle cx=\"{cx}\" cy=\"58\" r=\"4\" fill=\"{color}\"/>\
             <text x=\"{tx}\" y=\"62\" font-family=\"monospace\" font-size=\"10\" fill=\"{ts}\">{label}</text>",
            cx = lx, tx = lx + 8, ts = c.text_secondary,
        )
    }).collect::<String>();

    format!(
        r#"<svg width="{W}" height="{h}" viewBox="0 0 {W} {h}" xmlns="http://www.w3.org/2000/svg">
<defs>
  <clipPath id="lcsk-clip">
    <rect width="{W}" height="{h}" rx="6"/>
  </clipPath>
</defs>
<rect width="{W}" height="{h}" rx="6" fill="{bg}"/>
<g clip-path="url(#lcsk-clip)">{rain}</g>
<rect width="{W}" height="{h}" rx="6" fill="none" stroke="{border}" stroke-width="1"/>
<text x="25" y="35" font-family="monospace" font-size="14" font-weight="600" fill="{title}">LeetCode - Top Skills</text>
<line x1="25" y1="48" x2="470" y2="48" stroke="{sep}" stroke-width="1"/>
{legend}
{rows}
</svg>"#,
        bg = c.bg,
        border = c.border,
        title = c.title,
        sep = c.separator,
        rain = rain,
        legend = legend,
        rows = rows,
    )
}
