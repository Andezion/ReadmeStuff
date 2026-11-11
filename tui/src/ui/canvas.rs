use ratatui::Frame;
use ratatui::layout::Rect;
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::Span;
use ratatui::widgets::{Block, BorderType, Borders};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CellRect {
    pub rect: Rect,
    pub clipped_top: bool,
    pub clipped_bottom: bool,
}

#[allow(clippy::too_many_arguments)]
pub fn project(
    inner: Rect,
    scale_x: f64,
    scale_y: f64,
    scroll_y: u32,
    x: i32,
    y: i32,
    w: u32,
    h: u32,
) -> Option<CellRect> {
    let col = |px: i64| (px as f64 * scale_x).round() as i64;
    let row = |py: i64| (py as f64 * scale_y).round() as i64;

    let left = col(x as i64);
    let right = col(x as i64 + w as i64).max(left + 1);
    let rel_top = y as i64 - scroll_y as i64;
    let top = row(rel_top);
    let bottom = row(rel_top + h as i64).max(top + 1);

    let (iw, ih) = (inner.width as i64, inner.height as i64);
    let (cl, cr) = (left.clamp(0, iw), right.clamp(0, iw));
    let (ct, cb) = (top.clamp(0, ih), bottom.clamp(0, ih));
    if cl >= cr || ct >= cb {
        return None;
    }

    Some(CellRect {
        rect: Rect {
            x: inner.x + cl as u16,
            y: inner.y + ct as u16,
            width: (cr - cl) as u16,
            height: (cb - ct) as u16,
        },
        clipped_top: top < 0,
        clipped_bottom: bottom > ih,
    })
}

pub fn draw_tile(
    frame: &mut Frame,
    cell: CellRect,
    title: &str,
    border: Color,
    fill: Option<Color>,
) {
    let rect = cell.rect;
    if rect.width < 2 || rect.height < 2 {
        let block = Block::new().style(Style::default().bg(fill.unwrap_or(border)));
        frame.render_widget(block, rect);
        return;
    }

    let mut borders = Borders::ALL;
    if cell.clipped_top {
        borders -= Borders::TOP;
    }
    if cell.clipped_bottom {
        borders -= Borders::BOTTOM;
    }

    let mut block = Block::new()
        .borders(borders)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(border))
        .title(Span::styled(
            title,
            Style::default().fg(border).add_modifier(Modifier::BOLD),
        ));
    if let Some(fill) = fill {
        block = block.style(Style::default().bg(fill));
    }
    frame.render_widget(block, rect);
}

const TILE_COLORS: [(Color, Color); 6] = [
    (Color::Rgb(0x58, 0xa6, 0xff), Color::Rgb(0x0f, 0x1d, 0x30)),
    (Color::Rgb(0xbc, 0x8c, 0xff), Color::Rgb(0x1f, 0x16, 0x30)),
    (Color::Rgb(0xf0, 0x88, 0x3e), Color::Rgb(0x2d, 0x1a, 0x0c)),
    (Color::Rgb(0x39, 0xc5, 0xcf), Color::Rgb(0x0b, 0x25, 0x27)),
    (Color::Rgb(0xd2, 0x99, 0x22), Color::Rgb(0x2a, 0x20, 0x08)),
    (Color::Rgb(0xf7, 0x78, 0xba), Color::Rgb(0x2e, 0x14, 0x22)),
];

pub fn tile_colors(id: &str) -> (Color, Color) {
    let hash = id.bytes().fold(0x811c_9dc5_u32, |h, b| {
        (h ^ b as u32).wrapping_mul(0x0100_0193)
    });
    TILE_COLORS[hash as usize % TILE_COLORS.len()]
}

#[cfg(test)]
mod tests {
    use super::*;

    const INNER: Rect = Rect {
        x: 10,
        y: 5,
        width: 99,
        height: 40,
    };
    const SX: f64 = 0.1;
    const SY: f64 = 0.05;

    #[test]
    fn widgets_touching_in_pixels_do_not_share_a_cell() {
        let a = project(INNER, SX, SY, 0, 0, 0, 333, 100).unwrap().rect;
        let b = project(INNER, SX, SY, 0, 333, 0, 333, 100).unwrap().rect;
        assert_eq!(a.x + a.width, b.x);

        let c = project(INNER, SX, SY, 0, 0, 0, 100, 133).unwrap().rect;
        let d = project(INNER, SX, SY, 0, 0, 133, 100, 133).unwrap().rect;
        assert_eq!(c.y + c.height, d.y);
    }

    #[test]
    fn a_widget_scrolled_partly_out_keeps_only_its_visible_height() {
        let cell = project(INNER, SX, SY, 100, 0, 0, 100, 200).unwrap();
        assert_eq!(cell.rect.y, INNER.y);
        assert_eq!(cell.rect.height, 5);
        assert!(cell.clipped_top);
        assert!(!cell.clipped_bottom);
    }

    #[test]
    fn a_widget_below_the_viewport_is_clipped_at_the_bottom() {
        let cell = project(INNER, SX, SY, 0, 0, 700, 100, 300).unwrap();
        assert_eq!(cell.rect.y + cell.rect.height, INNER.y + INNER.height);
        assert!(cell.clipped_bottom);
    }

    #[test]
    fn fully_hidden_widgets_are_not_projected() {
        assert!(project(INNER, SX, SY, 500, 0, 0, 100, 200).is_none());
        assert!(project(INNER, SX, SY, 0, 0, 900, 100, 200).is_none());
    }

    #[test]
    fn tile_colors_are_stable_per_id() {
        assert_eq!(tile_colors("stats.svg"), tile_colors("stats.svg"));
    }
}
