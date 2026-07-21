use crate::app::App;
use crate::ui::theme::{self, PALETTE};
use ratatui::Frame;
use ratatui::layout::Alignment;
use ratatui::style::Style;
use ratatui::text::{Line, Text};
use ratatui::widgets::Paragraph;

const SPINNER: [char; 4] = ['|', '/', '-', '\\'];

pub fn draw(frame: &mut Frame, app: &App) {
    let area = frame.area();
    let spinner = SPINNER[(app.build_tick as usize) % SPINNER.len()];

    let text = Text::from(vec![
        Line::from(""),
        Line::from(format!("Building... {spinner}")),
        Line::from(""),
        Line::from("Fetching, rendering and composing - needs network, this can take a few seconds."),
        Line::from(""),
        Line::from("[Esc] Quit"),
    ]);

    let para = Paragraph::new(text)
        .style(Style::default().fg(PALETTE.text_primary).bg(PALETTE.bg))
        .alignment(Alignment::Center)
        .block(theme::block("building"));
    frame.render_widget(para, area);
}
