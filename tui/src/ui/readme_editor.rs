use crate::app::{App, ScreenRect};
use crate::ui::theme::{self, PALETTE};
use ratatui::Frame;
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::{Color, Style};
use ratatui::text::Line;
use ratatui::widgets::Paragraph;
use std::path::Path;

const SIDEBAR_ITEM_HEIGHT: u16 = 5;
const SIDEBAR_ITEM_GAP: u16 = 1;

pub fn draw(frame: &mut Frame, app: &mut App) {
    let area = frame.area();
    let outer = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(3), Constraint::Length(2)])
        .split(area);

    let cols = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(25), Constraint::Percentage(75)])
        .split(outer[0]);

    draw_sidebar(frame, app, cols[0]);
    draw_canvas(frame, app, cols[1]);
    draw_help(frame, app, outer[1]);
}

fn short_label(id: &str) -> &str {
    Path::new(id).file_name().and_then(|s| s.to_str()).unwrap_or(id)
}

fn draw_sidebar(frame: &mut Frame, app: &mut App, area: Rect) {
    let block = theme::block("widgets - RMB drag onto the canvas");
    let inner = block.inner(area);
    frame.render_widget(block, area);

    app.editor.sidebar_items.clear();
    if inner.width < 3 || inner.height < SIDEBAR_ITEM_HEIGHT {
        return;
    }

    let mut y = inner.y;
    for id in app.editor.sidebar.clone() {
        if y + SIDEBAR_ITEM_HEIGHT > inner.y + inner.height {
            break;
        }
        let Some(item) = app.editor.items.get(&id) else {
            continue;
        };
        let (w, h) = item.size;
        let aspect = w as f64 / h as f64;
        let box_w = ((SIDEBAR_ITEM_HEIGHT as f64 * 2.0 * aspect).round() as u16)
            .clamp(3, inner.width);

        let rect = Rect {
            x: inner.x,
            y,
            width: box_w,
            height: SIDEBAR_ITEM_HEIGHT,
        };
        frame.render_widget(theme::focusable_block(short_label(&id), false), rect);
        app.editor.sidebar_items.push((
            id,
            ScreenRect {
                x: rect.x,
                y: rect.y,
                w: rect.width,
                h: rect.height,
            },
        ));

        y += SIDEBAR_ITEM_HEIGHT + SIDEBAR_ITEM_GAP;
    }
}

fn draw_canvas(frame: &mut Frame, app: &mut App, area: Rect) {
    let block = theme::block("readme canvas - RMB: drag/drop   LMB: remove");
    let inner = block.inner(area);
    frame.render_widget(block, area);

    app.editor.canvas_area = ScreenRect {
        x: inner.x,
        y: inner.y,
        w: inner.width,
        h: inner.height,
    };
    if inner.width < 4 || inner.height < 4 {
        return;
    }

    let scale_x = inner.width as f64 / app.editor.canvas_width as f64;
    let scale_y = scale_x / 2.0;
    let scroll_y = app.editor.scroll_y;

    for item in &app.editor.placed {
        draw_box(
            frame, inner, scale_x, scale_y, scroll_y, item.x, item.y, item.w, item.h, &item.id,
            PALETTE.border,
        );
    }

    if let Some(drag) = &app.editor.drag {
        let color = if drag.valid { PALETTE.accent } else { Color::Red };
        draw_box(
            frame,
            inner,
            scale_x,
            scale_y,
            scroll_y,
            drag.current.0,
            drag.current.1,
            drag.w,
            drag.h,
            &drag.id,
            color,
        );
    }
}

#[allow(clippy::too_many_arguments)]
fn draw_box(
    frame: &mut Frame,
    inner: Rect,
    scale_x: f64,
    scale_y: f64,
    scroll_y: u32,
    x: i32,
    y: i32,
    w: u32,
    h: u32,
    id: &str,
    color: Color,
) {
    let rel_y = y - scroll_y as i32;
    if rel_y + h as i32 <= 0 {
        return;
    }
    let top_px = rel_y.max(0) as f64 * scale_y;
    if top_px >= inner.height as f64 {
        return;
    }

    let rx = inner.x + (x.max(0) as f64 * scale_x).round() as u16;
    let ry = inner.y + (top_px.round() as u16).min(inner.height.saturating_sub(1));
    if rx >= inner.x + inner.width || ry >= inner.y + inner.height {
        return;
    }

    let rw = ((w as f64 * scale_x).round() as u16)
        .max(3)
        .min(inner.x + inner.width - rx);
    let rh = ((h as f64 * scale_y).round() as u16)
        .max(2)
        .min(inner.y + inner.height - ry);

    let rect = Rect {
        x: rx,
        y: ry,
        width: rw,
        height: rh,
    };
    frame.render_widget(theme::colored_block(short_label(id), color), rect);
}

fn draw_help(frame: &mut Frame, app: &App, area: Rect) {
    let mut text = "RMB: drag to place/move   LMB: remove   wheel/PageUp/PageDown: scroll   \
                     Ctrl+S: export   Esc: main menu"
        .to_string();
    if let Some(status) = &app.status {
        text.push_str("   -   ");
        text.push_str(status);
    }
    let para = Paragraph::new(Line::from(text))
        .style(Style::default().fg(PALETTE.text_secondary).bg(PALETTE.bg));
    frame.render_widget(para, area);
}
