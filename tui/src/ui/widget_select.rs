use crate::app::{App, ScreenRect};
use crate::ui::theme::{self, PALETTE};
use ratatui::Frame;
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::Paragraph;

pub fn draw(frame: &mut Frame, app: &mut App) {
    let area = frame.area();
    let rows = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(3), Constraint::Length(2)])
        .split(area);

    draw_list(frame, app, rows[0]);
    draw_help(frame, app, rows[1]);
}

fn ensure_cursor_visible(app: &mut App, visible: usize) {
    if visible == 0 {
        return;
    }
    if app.discovered_cursor < app.discovered_scroll {
        app.discovered_scroll = app.discovered_cursor;
    } else if app.discovered_cursor >= app.discovered_scroll + visible {
        app.discovered_scroll = app.discovered_cursor + 1 - visible;
    }
}

fn draw_list(frame: &mut Frame, app: &mut App, area: Rect) {
    let block = theme::block("select widgets - click or Space/Enter: toggle   L: continue");
    let inner = block.inner(area);
    frame.render_widget(block, area);

    app.discovered_rows.clear();
    if inner.width == 0 || inner.height == 0 || app.discovered.is_empty() {
        return;
    }

    let visible = inner.height as usize;
    ensure_cursor_visible(app, visible);

    let start = app.discovered_scroll;
    let end = (start + visible).min(app.discovered.len());

    for (row_offset, idx) in (start..end).enumerate() {
        let w = &app.discovered[idx];
        let checked = app.discovered_selected.contains(&w.id);
        let mark = if checked { "x" } else { " " };
        let focused = idx == app.discovered_cursor;

        let spans = vec![
            Span::styled(
                format!("[{mark}] "),
                Style::default().fg(if checked {
                    PALETTE.accent
                } else {
                    PALETTE.text_secondary
                }),
            ),
            Span::styled(w.id.clone(), Style::default().fg(PALETTE.text_primary)),
            Span::styled(
                format!("  ({}x{})", w.size.0, w.size.1),
                Style::default().fg(PALETTE.text_secondary),
            ),
        ];

        let row_rect = Rect {
            x: inner.x,
            y: inner.y + row_offset as u16,
            width: inner.width,
            height: 1,
        };
        let style = if focused {
            Style::default().add_modifier(Modifier::REVERSED)
        } else {
            Style::default()
        };
        frame.render_widget(Paragraph::new(Line::from(spans)).style(style), row_rect);

        app.discovered_rows.push((
            idx,
            ScreenRect {
                x: row_rect.x,
                y: row_rect.y,
                w: row_rect.width,
                h: row_rect.height,
            },
        ));
    }
}

fn draw_help(frame: &mut Frame, app: &App, area: Rect) {
    let mut text = if app.discovered.is_empty() {
        format!(
            "no SVGs found under {} or {}   -   Esc: main menu",
            readme_stuff_catalog::WIDGETS_DIR,
            readme_stuff_catalog::TEXT_WIDGETS_DIR
        )
    } else {
        "click or Space/Enter: toggle   wheel/Up/Down/j/k: move   L: continue   Esc: main menu"
            .to_string()
    };
    if let Some(status) = &app.status {
        text.push_str("   -   ");
        text.push_str(status);
    }
    let para = Paragraph::new(Line::from(text))
        .style(Style::default().fg(PALETTE.text_secondary).bg(PALETTE.bg));
    frame.render_widget(para, area);
}
