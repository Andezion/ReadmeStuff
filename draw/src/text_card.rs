use crate::{
    matrix,
    text_glyph::{Align, Font, TextLine},
    theme::Theme,
};

const FONT_BYTES: &[u8] =
    include_bytes!(concat!(env!("CARGO_MANIFEST_DIR"), "/../text/matrix.ttf"));

const W: u32 = 495;
const PAD_X: f32 = 25.0;
const PAD_TOP: f32 = 30.0;
const PAD_BOTTOM: f32 = 20.0;
const MIN_H: u32 = 80;

pub fn render_text_card(lines: &[TextLine], align: Align, theme: Theme) -> String {
    let c = theme.colors();
    let font = Font::from_bytes(FONT_BYTES);

    let mut paths = String::new();
    let mut y = PAD_TOP;
    for line in lines {
        y += font.ascent(line.size);
        let width = font.measure_line(&line.text, line.size);
        let x0 = match align {
            Align::Left => PAD_X,
            Align::Center => ((W as f32 - width) / 2.0).max(PAD_X),
            Align::Right => (W as f32 - PAD_X - width).max(PAD_X),
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
    let h = ((y + PAD_BOTTOM).ceil() as u32).max(MIN_H);

    let rain = matrix::generate(W, h, c.matrix_color, c.matrix_opacity, 0x7EA7_7EA7, "tx");

    format!(
        r#"<svg width="{W}" height="{h}" viewBox="0 0 {W} {h}" xmlns="http://www.w3.org/2000/svg">
<defs>
  <clipPath id="tx-clip">
    <rect width="{W}" height="{h}" rx="6"/>
  </clipPath>
</defs>
<rect width="{W}" height="{h}" rx="6" fill="{bg}"/>
<g clip-path="url(#tx-clip)">{rain}</g>
<rect width="{W}" height="{h}" rx="6" fill="none" stroke="{border}" stroke-width="1"/>
<g>{paths}</g>
</svg>"#,
        W = W,
        h = h,
        bg = c.bg,
        border = c.border,
        rain = rain,
        paths = paths,
    )
}
