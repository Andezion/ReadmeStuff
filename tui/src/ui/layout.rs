use crate::app::App;
use crate::ui::theme::{self, PALETTE};
use ratatui::Frame;
use ratatui::layout::{Constraint, Direction, Layout as RatatuiLayout, Rect};
use ratatui::style::Style;
use ratatui::text::Line;
use ratatui::widgets::Paragraph;
use readme_stuff_catalog::registry;

const CANVAS_WIDTH: f64 = 990.0;

pub fn draw(frame: &mut Frame, app: &mut App) {
    let area = frame.area();
    let chunks = RatatuiLayout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(3), Constraint::Length(2)])
        .split(area);

    draw_canvas(frame, app, chunks[0]);
    draw_help(frame, app, chunks[1]);
}

fn draw_canvas(frame: &mut Frame, app: &App, area: Rect) {
    let block = theme::block("layout - arrange widgets on the canvas");
    let inner = block.inner(area);
    frame.render_widget(block, area);

    if inner.width < 4 || inner.height < 4 {
        return;
    }

    let scale_x = inner.width as f64 / CANVAS_WIDTH;
    let scale_y = scale_x / 2.0;

    let order: Vec<&'static str> = registry::all_widgets()
        .iter()
        .filter(|spec| app.selected.contains(spec.id))
        .map(|spec| spec.id)
        .collect();

    for (idx, id) in order.iter().copied().enumerate() {
        let Some(spec) = registry::find(id) else {
            continue;
        };
        let Some(&(x, y)) = app.layout_positions.get(id) else {
            continue;
        };

        let rx = inner.x + (x as f64 * scale_x).round() as u16;
        let ry = inner.y + (y as f64 * scale_y).round() as u16;
        if rx >= inner.x + inner.width || ry >= inner.y + inner.height {
            continue;
        }

        let rw = ((spec.size.0 as f64 * scale_x).round() as u16)
            .max(3)
            .min(inner.x + inner.width - rx);
        let rh = ((spec.size.1 as f64 * scale_y).round() as u16)
            .max(3)
            .min(inner.y + inner.height - ry);

        let rect = Rect {
            x: rx,
            y: ry,
            width: rw,
            height: rh,
        };
        let focused = idx == app.layout_cursor;
        frame.render_widget(theme::focusable_block(spec.label, focused), rect);
    }
}

fn draw_help(frame: &mut Frame, app: &App, area: Rect) {
    let mut text = "Tab: next widget   arrows: move   Ctrl+S: save & build   Esc: back".to_string();
    if let Some(status) = &app.status {
        text.push_str("   -   ");
        text.push_str(status);
    }
    let para = Paragraph::new(Line::from(text))
        .style(Style::default().fg(PALETTE.text_secondary).bg(PALETTE.bg));
    frame.render_widget(para, area);
}
