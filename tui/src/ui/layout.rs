use crate::app::App;
use crate::ui::canvas;
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

        let Some(cell) = canvas::project(
            inner,
            scale_x,
            scale_y,
            0,
            x as i32,
            y as i32,
            spec.size.0,
            spec.size.1,
        ) else {
            continue;
        };
        let (border, fill) = if idx == app.layout_cursor {
            (PALETTE.accent, None)
        } else {
            let (b, f) = canvas::tile_colors(id);
            (b, Some(f))
        };
        canvas::draw_tile(frame, cell, spec.label, border, fill);
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
