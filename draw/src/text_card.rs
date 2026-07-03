use crate::{
    matrix,
    text_glyph::{Align, Font, TextLine},
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

    let mut paths = String::new();
    let mut y = PAD_TOP;
    for line in lines {
        y += font.ascent(line.size);
        let line_width = font.measure_line(&line.text, line.size);
        let x0 = match align {
            Align::Left => PAD_X,
            Align::Center => ((width as f32 - line_width) / 2.0).max(PAD_X),
            Align::Right => (width as f32 - PAD_X - line_width).max(PAD_X),
        };
        let d = font.line_path(&line.text, line.size, x0, y);
        if !d.is_empty() {
            paths.push_str(&format!(
                "<path d=\"{d}\" fill=\"{fill}\"/>",
                fill = c.text_primary
            ));
        }
        y += font.descent(line.size);
    }
    let h = ((y + PAD_BOTTOM).ceil() as u32).max(min_height);

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
