use crate::{
    matrix,
    text_glyph::{Align, Font, HAlign, TextLine, VAlign},
    theme::Theme,
};

const FONT_BYTES: &[u8] =
    include_bytes!(concat!(env!("CARGO_MANIFEST_DIR"), "/../text/matrix.ttf"));

pub const DEFAULT_WIDTH: u32 = 495;
pub const DEFAULT_HEIGHT: u32 = 80;
const PAD_X: f32 = 25.0;
const PAD_TOP: f32 = 30.0;
const PAD_BOTTOM: f32 = 20.0;

pub fn render_text_card(
    lines: &[TextLine],
    align: Align,
    theme: Theme,
    width: u32,
    min_height: u32,
) -> String {
    let c = theme.colors();
    let font = Font::from_bytes(FONT_BYTES);

    struct LineMetric {
        ascent: f32,
        descent: f32,
        width: f32,
    }
    let metrics: Vec<LineMetric> = lines
        .iter()
        .map(|line| LineMetric {
            ascent: font.ascent(line.size),
            descent: font.descent(line.size),
            width: font.measure_line(&line.text, line.size),
        })
        .collect();
    let text_height: f32 = metrics.iter().map(|m| m.ascent + m.descent).sum();

    let h = ((text_height + PAD_TOP + PAD_BOTTOM).ceil() as u32).max(min_height);

    let mut y = match align.v {
        VAlign::Top => PAD_TOP,
        VAlign::Center => ((h as f32 - text_height) / 2.0).max(PAD_TOP),
        VAlign::Bottom => (h as f32 - PAD_BOTTOM - text_height).max(PAD_TOP),
    };

    let mut paths = String::new();
    for (line, m) in lines.iter().zip(metrics.iter()) {
        y += m.ascent;
        let x0 = match align.h {
            HAlign::Left => PAD_X,
            HAlign::Center => ((width as f32 - m.width) / 2.0).max(PAD_X),
            HAlign::Right => (width as f32 - PAD_X - m.width).max(PAD_X),
        };
        let d = font.line_path(&line.text, line.size, x0, y);
        if !d.is_empty() {
            paths.push_str(&format!(
                "<path d=\"{d}\" fill=\"{fill}\"/>",
                fill = c.text_primary
            ));
        }
        y += m.descent;
    }

    let rain = matrix::generate(width, h, c.matrix_color, c.matrix_opacity, 0x7EA7_7EA7, "tx");

    format!(
        r#"<svg width="{width}" height="{h}" viewBox="0 0 {width} {h}" xmlns="http://www.w3.org/2000/svg">
<defs>
  <clipPath id="tx-clip">
    <rect width="{width}" height="{h}" rx="6"/>
  </clipPath>
</defs>
<rect width="{width}" height="{h}" rx="6" fill="{bg}"/>
<g clip-path="url(#tx-clip)">{rain}</g>
<rect width="{width}" height="{h}" rx="6" fill="none" stroke="{border}" stroke-width="1"/>
<g>{paths}</g>
</svg>"#,
        width = width,
        h = h,
        bg = c.bg,
        border = c.border,
        rain = rain,
        paths = paths,
    )
}
