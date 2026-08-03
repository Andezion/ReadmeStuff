use crate::{
    matrix,
    text_glyph::{Align, Font, HAlign, VAlign},
    theme::Theme,
};

const FONT_BYTES: &[u8] =
    include_bytes!(concat!(env!("CARGO_MANIFEST_DIR"), "/../text/matrix.ttf"));

const PAD_X: f32 = 25.0;
const PAD_TOP: f32 = 30.0;

pub fn render_text_widget(
    lines: &[&str],
    font_size: f32,
    line_height: f32,
    align: Align,
    theme: Theme,
    width: u32,
    height: u32,
) -> String {
    let c = theme.colors();
    let font = Font::from_bytes(FONT_BYTES);

    let ascent = font.ascent(font_size);
    let descent = font.descent(font_size);
    let block_height = if lines.is_empty() {
        0.0
    } else {
        ascent + descent + (lines.len() as f32 - 1.0) * line_height
    };

    let start_y = match align.v {
        VAlign::Top => PAD_TOP,
        VAlign::Center => ((height as f32 - block_height) / 2.0).max(0.0),
        VAlign::Bottom => (height as f32 - block_height).max(0.0),
    };

    let mut paths = String::new();
    let mut baseline = start_y + ascent;
    for line in lines {
        let line_width = font.measure_line(line, font_size);
        let x0 = match align.h {
            HAlign::Left => PAD_X,
            HAlign::Center => ((width as f32 - line_width) / 2.0).max(0.0),
            HAlign::Right => (width as f32 - PAD_X - line_width).max(0.0),
        };
        let d = font.line_path(line, font_size, x0, baseline);
        if !d.is_empty() {
            paths.push_str(&format!(
                r#"<path d="{d}" fill="{fill}"/>"#,
                fill = c.text_primary
            ));
        }
        baseline += line_height;
    }

    let rain = matrix::generate(
        width,
        height,
        c.matrix_color,
        c.matrix_opacity,
        0x7EA7_7EA7,
        "tw",
    );

    format!(
        r#"<svg width="{width}" height="{height}" viewBox="0 0 {width} {height}" xmlns="http://www.w3.org/2000/svg">
<defs>
  <clipPath id="tw-clip">
    <rect width="{width}" height="{height}" rx="6"/>
  </clipPath>
</defs>
<rect width="{width}" height="{height}" rx="6" fill="{bg}"/>
<g clip-path="url(#tw-clip)">{rain}</g>
<rect width="{width}" height="{height}" rx="6" fill="none" stroke="{border}" stroke-width="1"/>
<g>{paths}</g>
</svg>"#,
        width = width,
        height = height,
        bg = c.bg,
        border = c.border,
        rain = rain,
        paths = paths,
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn renders_declared_width_and_height() {
        let svg = render_text_widget(&["hello"], 16.0, 20.0, Align::DEFAULT, Theme::Dark, 300, 80);
        assert!(svg.contains(r#"width="300""#));
        assert!(svg.contains(r#"height="80""#));
    }

    #[test]
    fn empty_lines_still_produce_a_valid_svg() {
        let svg = render_text_widget(&[], 16.0, 20.0, Align::DEFAULT, Theme::Dark, 300, 80);
        assert!(svg.starts_with("<svg"));
        assert!(svg.trim_end().ends_with("</svg>"));
    }

    #[test]
    fn produces_a_path_per_non_empty_line() {
        let svg = render_text_widget(
            &["one", "two"],
            16.0,
            20.0,
            Align::DEFAULT,
            Theme::Dark,
            300,
            80,
        );
        assert_eq!(svg.matches("<path").count(), 2);
    }
}
