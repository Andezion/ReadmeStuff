use crate::app::{self, App, ScreenRect};
use crate::ui::canvas::{self, CellRect};
use crate::ui::theme::{self, PALETTE};
use ratatui::Frame;
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::{Color, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::Paragraph;
use std::path::Path;

const SIDEBAR_ITEM_HEIGHT: u16 = 5;
const SIDEBAR_ITEM_GAP: u16 = 1;

const DROP_OK: Color = Color::Rgb(0x3f, 0xb9, 0x50);
const DROP_BLOCKED: Color = Color::Rgb(0xf8, 0x51, 0x49);
const BLOCKED_FILL: Color = Color::Rgb(0x3b, 0x10, 0x10);

pub fn draw(frame: &mut Frame, app: &mut App) {
    let area = frame.area();
    let outer = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(3), Constraint::Length(3)])
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
    Path::new(id)
        .file_name()
        .and_then(|s| s.to_str())
        .unwrap_or(id)
}

fn to_screen_rect(r: Rect) -> ScreenRect {
    ScreenRect {
        x: r.x,
        y: r.y,
        w: r.width,
        h: r.height,
    }
}

fn draw_sidebar(frame: &mut Frame, app: &mut App, area: Rect) {
    let stride = SIDEBAR_ITEM_HEIGHT + SIDEBAR_ITEM_GAP;
    let inner_h = area.height.saturating_sub(2);
    // the last item needs no trailing gap
    let capacity = ((inner_h + SIDEBAR_ITEM_GAP) / stride).max(1) as usize;
    let len = app.editor.sidebar.len();
    let max_scroll = len.saturating_sub(capacity);
    app.editor.sidebar_scroll = app.editor.sidebar_scroll.min(max_scroll);
    let start = app.editor.sidebar_scroll;
    let end = (start + capacity).min(len);

    let title = if len == 0 {
        "widgets - all placed".to_string()
    } else {
        let up = if start > 0 { "▲" } else { " " };
        let down = if end < len { "▼" } else { " " };
        format!("widgets {}-{} of {len} {up}{down}", start + 1, end)
    };
    let block = theme::block(&title);
    let inner = block.inner(area);
    frame.render_widget(block, area);

    app.editor.sidebar_items.clear();
    app.editor.sidebar_area = to_screen_rect(inner);
    if inner.width < 3 || inner.height < SIDEBAR_ITEM_HEIGHT {
        return;
    }

    let dragging = app.editor.drag.as_ref().map(|d| d.id.clone());
    let mut y = inner.y;
    for id in app.editor.sidebar[start..end].iter().cloned() {
        if y + SIDEBAR_ITEM_HEIGHT > inner.y + inner.height {
            break;
        }
        let Some(item) = app.editor.items.get(&id) else {
            continue;
        };
        let (w, h) = item.size;
        let aspect = w as f64 / h.max(1) as f64;
        let box_w =
            ((SIDEBAR_ITEM_HEIGHT as f64 * 2.0 * aspect).round() as u16).clamp(3, inner.width);

        let rect = Rect {
            x: inner.x,
            y,
            width: box_w,
            height: SIDEBAR_ITEM_HEIGHT,
        };
        let (border, fill) = canvas::tile_colors(&id);
        let border = if dragging.as_deref() == Some(id.as_str()) {
            PALETTE.border
        } else {
            border
        };
        let cell = CellRect {
            rect,
            clipped_top: false,
            clipped_bottom: false,
        };
        canvas::draw_tile(frame, cell, short_label(&id), border, Some(fill));

        let size = format!("{w}x{h}");
        if box_w >= size.len() as u16 + 2 {
            let label = Rect {
                x: rect.x + 1,
                y: rect.y + rect.height - 2,
                width: rect.width - 2,
                height: 1,
            };
            frame.render_widget(
                Paragraph::new(size).style(Style::default().fg(PALETTE.text_secondary)),
                label,
            );
        }

        app.editor.sidebar_items.push((id, to_screen_rect(rect)));
        y += stride;
    }
}

fn draw_canvas(frame: &mut Frame, app: &mut App, area: Rect) {
    let scroll_y = app.editor.scroll_y;
    let inner_preview = theme::block("").inner(area);
    let scale_x = inner_preview.width as f64 / app.editor.canvas_width as f64;
    let scale_y = scale_x / 2.0;

    let title = if scale_y > 0.0 {
        let view_bottom = scroll_y + (inner_preview.height as f64 / scale_y) as u32;
        format!(
            "canvas {}px - showing y {scroll_y}-{view_bottom} (content ends at {})",
            app.editor.canvas_width,
            app::max_scroll_y(app)
        )
    } else {
        "canvas".to_string()
    };
    let block = theme::block(&title);
    let inner = block.inner(area);
    frame.render_widget(block, area);

    app.editor.canvas_area = to_screen_rect(inner);
    if inner.width < 4 || inner.height < 4 {
        return;
    }

    let project = |x: i32, y: i32, w: u32, h: u32| {
        canvas::project(inner, scale_x, scale_y, scroll_y, x, y, w, h)
    };

    let drag = app.editor.drag.as_ref().filter(|d| d.over_canvas);
    let colliding = |id: &str| drag.is_some_and(|d| d.collisions.iter().any(|c| c == id));

    for item in &app.editor.placed {
        let Some(cell) = project(item.x, item.y, item.w, item.h) else {
            continue;
        };
        let (border, fill) = if colliding(&item.id) {
            (DROP_BLOCKED, BLOCKED_FILL)
        } else {
            canvas::tile_colors(&item.id)
        };
        let title = format!("{} {}x{}", short_label(&item.id), item.w, item.h);
        canvas::draw_tile(frame, cell, &title, border, Some(fill));
    }

    if let Some(drag) = drag {
        let Some(cell) = project(drag.current.0, drag.current.1, drag.w, drag.h) else {
            return;
        };
        let (color, verdict) = if drag.valid {
            (DROP_OK, "release to place".to_string())
        } else if drag.too_wide {
            (DROP_BLOCKED, "wider than canvas".to_string())
        } else {
            (
                DROP_BLOCKED,
                format!("overlaps {}", short_labels(&drag.collisions)),
            )
        };
        let title = format!("{} - {verdict}", short_label(&drag.id));
        canvas::draw_tile(frame, cell, &title, color, None);
    }
}

fn short_labels(ids: &[String]) -> String {
    ids.iter()
        .map(|id| short_label(id))
        .collect::<Vec<_>>()
        .join(", ")
}

fn draw_help(frame: &mut Frame, app: &App, area: Rect) {
    let legend = Line::from(vec![
        Span::styled("■ ", Style::default().fg(DROP_OK)),
        Span::styled("can drop  ", Style::default().fg(PALETTE.text_secondary)),
        Span::styled("■ ", Style::default().fg(DROP_BLOCKED)),
        Span::styled(
            "overlaps another widget - releasing puts it back",
            Style::default().fg(PALETTE.text_secondary),
        ),
    ]);
    let keys = Line::from(Span::styled(
        "drag: place/move   click: remove   wheel: scroll panel under pointer   \
         PgUp/PgDn/Home/End: scroll canvas   Ctrl+S: export   Esc: main menu",
        Style::default().fg(PALETTE.text_secondary),
    ));
    let status = Line::from(Span::styled(
        app.status.clone().unwrap_or_default(),
        Style::default().fg(PALETTE.text_primary),
    ));
    let para = Paragraph::new(vec![legend, keys, status]).style(Style::default().bg(PALETTE.bg));
    frame.render_widget(para, area);
}
