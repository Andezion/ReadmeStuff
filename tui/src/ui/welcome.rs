use crate::app::App;
use crate::ui::theme::{self, PALETTE};
use ratatui::Frame;
use ratatui::layout::Alignment;
use ratatui::style::Style;
use ratatui::text::{Line, Text};
use ratatui::widgets::Paragraph;

pub fn draw(frame: &mut Frame, app: &App) {
    let area = frame.area();
    let path = app
        .found_config_path
        .as_deref()
        .map(|p| p.display().to_string())
        .unwrap_or_default();

    let text = Text::from(vec![
        Line::from(""),
        Line::from("readme-stuff"),
        Line::from(""),
        Line::from(format!("Found an existing config: {path}")),
        Line::from(""),
        Line::from("[R] Resume it     [N] Start new     [Esc] Quit"),
    ]);

    let para = Paragraph::new(text)
        .style(Style::default().fg(PALETTE.text_primary).bg(PALETTE.bg))
        .alignment(Alignment::Center)
        .block(theme::block("readme-stuff"));
    frame.render_widget(para, area);
}
