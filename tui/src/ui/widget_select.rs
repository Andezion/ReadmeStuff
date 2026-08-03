use crate::app::App;
use crate::ui::theme::{self, PALETTE};
use ratatui::Frame;
use ratatui::layout::{Constraint, Direction, Layout};
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{List, ListItem, ListState, Paragraph};

pub fn draw(frame: &mut Frame, app: &mut App) {
    let area = frame.area();
    let rows = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(3), Constraint::Length(2)])
        .split(area);

    let block = theme::block("select widgets - Space/Enter: toggle   L: continue");

    let items: Vec<ListItem> = app
        .discovered
        .iter()
        .map(|w| {
            let checked = app.discovered_selected.contains(&w.id);
            let mark = if checked { "x" } else { " " };
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
            ListItem::new(Line::from(spans))
        })
        .collect();

    let mut state = ListState::default();
    if !app.discovered.is_empty() {
        state.select(Some(app.discovered_cursor));
    }

    let list = List::new(items)
        .block(block)
        .highlight_style(Style::default().add_modifier(Modifier::REVERSED));
    frame.render_stateful_widget(list, rows[0], &mut state);

    draw_help(frame, app, rows[1]);
}

fn draw_help(frame: &mut Frame, app: &App, area: ratatui::layout::Rect) {
    let mut text = if app.discovered.is_empty() {
        format!(
            "no SVGs found under {} or {}   -   Esc: main menu",
            readme_stuff_catalog::WIDGETS_DIR,
            readme_stuff_catalog::TEXT_WIDGETS_DIR
        )
    } else {
        "Up/Down/j/k: move   Space/Enter: toggle   L: continue   Esc: main menu".to_string()
    };
    if let Some(status) = &app.status {
        text.push_str("   -   ");
        text.push_str(status);
    }
    let para = Paragraph::new(Line::from(text))
        .style(Style::default().fg(PALETTE.text_secondary).bg(PALETTE.bg));
    frame.render_widget(para, area);
}
