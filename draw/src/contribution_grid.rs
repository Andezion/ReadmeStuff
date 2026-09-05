use crate::{matrix, rng::Lcg, theme::Theme};
use chrono::{Datelike, Utc};
use readme_stuff_aggregator::widgets::ContributionGridWidget;

const W: u32 = 990;
pub const SIZE: (u32, u32) = (W, H);

const CELL: i32 = 13;
const GAP: i32 = 4;
const PITCH: i32 = CELL + GAP;

const GRID_TOP: i32 = 74;
const H: u32 = 218;

const FLASH_COLOR: &str = "#a9ffcf";
const BASE_DARKEN: f64 = 0.5;

const MONTHS: [&str; 12] = [
    "Jan", "Feb", "Mar", "Apr", "May", "Jun", "Jul", "Aug", "Sep", "Oct", "Nov", "Dec",
];

fn darken(hex: &str, factor: f64) -> String {
    let channel = |offset: usize| -> u8 {
        u8::from_str_radix(&hex[offset..offset + 2], 16)
            .map(|v| (v as f64 * factor).round() as u8)
            .unwrap_or(0)
    };
    format!("#{:02x}{:02x}{:02x}", channel(1), channel(3), channel(5))
}

pub fn render_contribution_grid(w: &ContributionGridWidget, theme: Theme) -> String {
    let c = theme.colors();
    let font = c.font_family;
    let ambient_rain = matrix::generate(W, H, c.matrix_color, c.matrix_opacity, 0x6161_4e21, "cr");

    let cols = w.weeks.len().max(1) as i32;
    let grid_w = cols * PITCH - GAP;
    let left = ((W as i32) - grid_w) / 2;

    let col_x = |i: i32| left + i * PITCH;

    let mut grid = String::new();
    let mut month_labels = String::new();
    let mut last_month: Option<u32> = None;
    let mut last_label_x: Option<i32> = None;

    for (i, week) in w.weeks.iter().enumerate() {
        let x = col_x(i as i32);

        if let Some(first_day) = week.contribution_days.first() {
            let m = first_day.date.month();
            let far_enough = last_label_x.is_none_or(|last_x| x - last_x >= 24);
            if last_month != Some(m) && far_enough {
                last_month = Some(m);
                last_label_x = Some(x);
                month_labels.push_str(&format!(
                    "<text x=\"{x}\" y=\"{y}\" font-family=\"{font}\" font-size=\"10\" fill=\"{tl}\">{label}</text>",
                    y = GRID_TOP - 10,
                    tl = c.text_secondary,
                    label = MONTHS[(m - 1) as usize],
                ));
            }
        }

        for day in &week.contribution_days {
            let row = day.weekday as i32;
            let y = GRID_TOP + row * PITCH;
            grid.push_str(&format!(
                "<rect x=\"{x}\" y=\"{y}\" width=\"{CELL}\" height=\"{CELL}\" rx=\"2\" fill=\"{fill}\"/>",
                fill = darken(&day.color, BASE_DARKEN),
            ));
        }
    }

    let seed_base = Utc::now().timestamp_nanos_opt().unwrap_or(0) as u64;

    let mut pulse_css = String::new();
    let mut pulse_rects = String::new();
    let mut prng = Lcg::new(seed_base ^ 0x4d41_5452);
    let active_cols = (cols / 5).clamp(6, 18) as usize;
    let mut seen = std::collections::HashSet::new();
    let mut picks = Vec::new();
    while picks.len() < active_cols.min(w.weeks.len()) {
        let idx = prng.range(0, cols as u64) as usize;
        if seen.insert(idx) {
            picks.push(idx);
        }
    }

    for &col_idx in &picks {
        let week = &w.weeks[col_idx];
        let days = &week.contribution_days;
        let Some(last_idx) = days.len().checked_sub(1) else {
            continue;
        };
        let start_row = prng.range(0, 2) as usize;
        let end_row = if prng.range(0, 2) == 0 {
            last_idx
        } else {
            last_idx.saturating_sub(1)
        };
        if start_row > end_row || start_row >= days.len() {
            continue;
        }

        let x = col_x(col_idx as i32);
        let duration = prng.rangef(2.6, 5.2);
        let base_delay = -prng.rangef(0.0, duration);
        let row_step = duration * 0.045;

        for (row_idx, day) in days.iter().enumerate().take(end_row + 1).skip(start_row) {
            let y = GRID_TOP + day.weekday as i32 * PITCH;
            let name = format!("cpulse{col_idx}_{row_idx}");
            let delay = base_delay + (row_idx - start_row) as f64 * row_step;
            let base = darken(&day.color, BASE_DARKEN);

            pulse_css.push_str(&format!(
                "@keyframes {name}{{0%{{fill:{base}}}5%{{fill:{flash}}}18%{{fill:{base}}}100%{{fill:{base}}}}}",
                flash = FLASH_COLOR,
            ));
            pulse_rects.push_str(&format!(
                "<rect x=\"{x}\" y=\"{y}\" width=\"{CELL}\" height=\"{CELL}\" rx=\"2\" fill=\"{base}\" style=\"animation:{name} {duration:.2}s ease-in-out {delay:.2}s infinite\"/>",
            ));
        }
    }

    format!(
        r#"<svg width="{W}" height="{H}" viewBox="0 0 {W} {H}" xmlns="http://www.w3.org/2000/svg">
<defs>
  <clipPath id="cgr-clip">
    <rect width="{W}" height="{H}" rx="6"/>
  </clipPath>
</defs>
<rect width="{W}" height="{H}" rx="6" fill="{bg}"/>
<g clip-path="url(#cgr-clip)">{ambient_rain}</g>
<rect width="{W}" height="{H}" rx="6" fill="none" stroke="{border}" stroke-width="1"/>
<text x="25" y="35" font-family="{font}" font-size="14" font-weight="600" fill="{title}">Contribution Rain</text>
<line x1="25" y1="52" x2="965" y2="52" stroke="{sep}" stroke-width="1"/>
{month_labels}
{grid}<style>{pulse_css}</style>{pulse_rects}
</svg>"#,
        bg = c.bg,
        border = c.border,
        title = c.title,
        sep = c.separator,
    )
}
