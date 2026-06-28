use crate::{
    helpers::{cf_rank_color, fmt_num, xml_escape},
    matrix,
    theme::Theme,
};
use readme_stuff_aggregator::widgets::CompetitiveWidget;

const W: u32 = 495;
const H: u32 = 195;

// Column x positions (left edges of 3 sections)
const CF_X: u32 = 25;
const CW_X: u32 = 185;
const LC_X: u32 = 345;
const DIV1: u32 = 167;
const DIV2: u32 = 327;

pub fn render_competitive(w: &CompetitiveWidget, theme: Theme) -> String {
    let c = theme.colors();
    let rain = matrix::generate(W, H, c.matrix_color, c.matrix_opacity, 0xCAFE_BABE, "cp");

    let cf_svg = render_cf(w, &c);
    let cw_svg = render_cw(w, &c);
    let lc_svg = render_lc(w, &c);

    format!(
        r#"<svg width="{W}" height="{H}" viewBox="0 0 {W} {H}" xmlns="http://www.w3.org/2000/svg">
<defs>
  <clipPath id="cp-clip">
    <rect width="{W}" height="{H}" rx="6"/>
  </clipPath>
</defs>
<rect width="{W}" height="{H}" rx="6" fill="{bg}"/>
<g clip-path="url(#cp-clip)">{rain}</g>
<rect width="{W}" height="{H}" rx="6" fill="none" stroke="{border}" stroke-width="1"/>
<text x="35" y="35" font-family="monospace" font-size="14" font-weight="600" fill="{title}">Competitive Programming</text>
<line x1="25" y1="52" x2="470" y2="52" stroke="{sep}" stroke-width="1"/>
<line x1="{DIV1}" y1="58" x2="{DIV1}" y2="185" stroke="{sep}" stroke-width="1"/>
<line x1="{DIV2}" y1="58" x2="{DIV2}" y2="185" stroke="{sep}" stroke-width="1"/>
{cf_svg}{cw_svg}{lc_svg}
</svg>"#,
        W = W,
        H = H,
        bg = c.bg,
        border = c.border,
        title = c.title,
        sep = c.separator,
        DIV1 = DIV1,
        DIV2 = DIV2,
        rain = rain,
        cf_svg = cf_svg,
        cw_svg = cw_svg,
        lc_svg = lc_svg,
    )
}

fn platform_label(x: u32, label: &str, color: &str) -> String {
    format!(
        "<text x=\"{x}\" y=\"72\" \
            font-family=\"monospace\" font-size=\"10\" font-weight=\"600\" \
            fill=\"{color}\" letter-spacing=\"1\">{label}</text>",
        label = label.to_uppercase(),
    )
}

fn stat_line(x: u32, y: u32, text: &str, size: u32, weight: &str, color: &str) -> String {
    format!(
        "<text x=\"{x}\" y=\"{y}\" \
            font-family=\"monospace\" font-size=\"{size}\" font-weight=\"{weight}\" \
            fill=\"{color}\">{text}</text>",
        text = xml_escape(text),
    )
}

fn render_cf(w: &CompetitiveWidget, c: &crate::theme::Colors) -> String {
    let Some(rating) = w.cf_rating else {
        return platform_label(CF_X, "codeforces", c.text_secondary)
            + &stat_line(CF_X, 105, "—", 18, "400", c.text_secondary);
    };

    let rank = w.cf_rank.as_deref().unwrap_or("unrated");
    let rcolor = cf_rank_color(rank);

    platform_label(CF_X, "codeforces", rcolor)
        + &stat_line(
            CF_X,
            105,
            &fmt_num(rating.unsigned_abs() as u64),
            20,
            "700",
            c.text_primary,
        )
        + &stat_line(CF_X, 123, rank, 11, "400", rcolor)
}

fn render_cw(w: &CompetitiveWidget, c: &crate::theme::Colors) -> String {
    let Some(ref rank) = w.cw_rank else {
        return platform_label(CW_X, "codewars", c.text_secondary)
            + &stat_line(CW_X, 105, "—", 18, "400", c.text_secondary);
    };

    // Codewars kyu colors (1-8, lower = better)
    let kyu_color = if rank.contains('1') || rank.contains('2') {
        "#ff6e96" // red/pink for 1-2 kyu
    } else if rank.contains('3') || rank.contains('4') {
        "#9b59b6" // purple for 3-4 kyu
    } else {
        "#8b949e" // gray for 5-8 kyu
    };

    let honor_str = w
        .cw_honor
        .map(|h| format!("honor {}", fmt_num(h as u64)))
        .unwrap_or_default();

    platform_label(CW_X, "codewars", kyu_color)
        + &stat_line(CW_X, 105, rank, 20, "700", c.text_primary)
        + &stat_line(CW_X, 123, &honor_str, 11, "400", c.text_secondary)
}

fn render_lc(w: &CompetitiveWidget, c: &crate::theme::Colors) -> String {
    let Some(solved) = w.lc_solved else {
        return platform_label(LC_X, "leetcode", c.text_secondary)
            + &stat_line(LC_X, 105, "—", 18, "400", c.text_secondary);
    };

    let solved_str = format!("{solved} solved");
    let breakdown = match (w.lc_easy, w.lc_medium, w.lc_hard) {
        (Some(e), Some(m), Some(h)) => format!("E:{e} M:{m} H:{h}"),
        _ => String::new(),
    };

    platform_label(LC_X, "leetcode", "#ffa116")
        + &stat_line(LC_X, 105, &solved_str, 18, "700", c.text_primary)
        + &stat_line(LC_X, 123, &breakdown, 11, "400", c.text_secondary)
}
