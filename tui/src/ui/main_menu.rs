use crate::app::App;
use crate::ui::theme::{self, PALETTE};
use ratatui::Frame;
use ratatui::layout::Alignment;
use ratatui::style::Style;
use ratatui::text::{Line, Text};
use ratatui::widgets::Paragraph;

pub fn draw(frame: &mut Frame, app: &mut App) {
    let area = frame.area();

    let mut lines = vec![
        Line::from(""),
        Line::from("readme-stuff - README page builder"),
        Line::from(""),
        Line::from("[G] Generate graphical widgets"),
        Line::from("[T] Create text widget"),
        Line::from("[W] Select widgets & build README"),
        Line::from(""),
        Line::from("[Esc] Quit"),
    ];
    if let Some(status) = &app.status {
        lines.push(Line::from(""));
        lines.push(Line::styled(
            status.as_str(),
            Style::default().fg(PALETTE.accent),
        ));
    }

    let para = Paragraph::new(Text::from(lines))
        .style(Style::default().fg(PALETTE.text_primary).bg(PALETTE.bg))
        .alignment(Alignment::Center)
        .block(theme::block("main menu"));
    frame.render_widget(para, area);
}
