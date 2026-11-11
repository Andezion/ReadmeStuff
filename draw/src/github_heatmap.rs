use crate::{matrix, theme::Theme};
use readme_stuff_aggregator::widgets::GithubHeatmapWidget;

const W: u32 = 495;
const H: u32 = 210;
pub const SIZE: (u32, u32) = (W, H);
const LABEL_X: u32 = 25;
const BAR_X: u32 = 65;
/// Right edge of the content area (matches the separator line end).
const RIGHT_X: u32 = 470;
/// Gap between the end of the bar track and the start of the number.
const COUNT_GAP: u32 = 10;
/// Conservative per-character width for 11px text (covers monospace fonts).
const CHAR_W: f64 = 7.0;
const ROW_H: u32 = 19;
const FIRST_Y: u32 = 70;

const DAYS: [&str; 7] = ["Sun", "Mon", "Tue", "Wed", "Thu", "Fri", "Sat"];

pub fn render_github_heatmap(w: &GithubHeatmapWidget, theme: Theme) -> String {
    let c = theme.colors();
    let font = c.font_family;
    let rain = matrix::generate(W, H, c.matrix_color, c.matrix_opacity, 0x7788_EEFF, "ghm");

    let max_val = *w.weekday_distribution.iter().max().unwrap_or(&1).max(&1);
    let (bar_max_w, count_x) = layout(&w.weekday_distribution);

    let mut bars = String::new();
    for (i, &count) in w.weekday_distribution.iter().enumerate() {
        let y = FIRST_Y + i as u32 * ROW_H - 1;
        let fill_w = (count as f64 / max_val as f64 * bar_max_w as f64).round() as u32;
        let bar_y = y - 11 + 2;

        bars.push_str(&format!(
            "<text x=\"{LABEL_X}\" y=\"{y}\" \
                font-family=\"{font}\" font-size=\"11\" \
                fill=\"{tl}\">{day}</text>\
             <rect x=\"{BAR_X}\" y=\"{bar_y}\" width=\"{bar_max_w}\" height=\"10\" rx=\"5\" \
                fill=\"{bar_bg}\"/>\
             <rect x=\"{BAR_X}\" y=\"{bar_y}\" width=\"{fill_w}\" height=\"10\" rx=\"5\" \
                fill=\"{accent}\"/>\
             <text x=\"{count_x}\" y=\"{y}\" text-anchor=\"end\" \
                font-family=\"{font}\" font-size=\"11\" \
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
<text x="25" y="35" font-family="{font}" font-size="14" font-weight="600" fill="{title}">Activity by Day of Week</text>
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

fn layout<T: ToString>(values: &[T]) -> (u32, u32) {
    let max_chars = values
        .iter()
        .map(|v| v.to_string().chars().count())
        .max()
        .unwrap_or(1)
        .max(1);
    let text_w = (max_chars as f64 * CHAR_W).ceil() as u32;
    let bar_max_w = RIGHT_X.saturating_sub(BAR_X + COUNT_GAP + text_w).max(50);
    (bar_max_w, RIGHT_X)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn layout_leaves_room_for_wide_numbers() {
        for vals in [vec![5u32], vec![123], vec![12_345], vec![1_234_567]] {
            let (bar_w, count_x) = layout(&vals);
            let text_w = (vals[0].to_string().len() as f64 * CHAR_W).ceil() as u32;
            assert!(BAR_X + bar_w + COUNT_GAP + text_w <= count_x, "{vals:?}");
        }
    }

    #[test]
    fn layout_shrinks_bar_as_numbers_grow() {
        let (small, _) = layout(&[99u32]);
        let (big, _) = layout(&[99_999u32]);
        assert!(big < small);
    }
}
