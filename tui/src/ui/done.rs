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
        .saved_path
        .as_deref()
        .map(|p| p.display().to_string())
        .unwrap_or_default();

    let text = Text::from(vec![
        Line::from(""),
        Line::from(format!("Saved {path}")),
        Line::from(""),
        Line::from("Run `cargo run -p readme-stuff-cli` to build your README assets."),
        Line::from(""),
        Line::from("[B] Back to edit     [Q] Quit"),
    ]);

    let para = Paragraph::new(text)
        .style(Style::default().fg(PALETTE.text_primary).bg(PALETTE.bg))
        .alignment(Alignment::Center)
        .block(theme::block("saved"));
    frame.render_widget(para, area);
}
