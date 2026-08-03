use crate::app::{App, TextWidgetField};
use crate::ui::theme::{self, PALETTE};
use ratatui::Frame;
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::Style;
use ratatui::text::Line;
use ratatui::widgets::Paragraph;
use readme_stuff_draw::{HAlign, VAlign};

pub fn draw(frame: &mut Frame, app: &mut App) {
    let area = frame.area();
    let rows = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),
            Constraint::Min(5),
            Constraint::Length(3),
            Constraint::Length(3),
            Constraint::Length(3),
            Constraint::Length(3),
            Constraint::Length(2),
            Constraint::Length(2),
        ])
        .split(area);

    render_field(
        frame,
        "Output filename (saved as <name>_text.svg)",
        &app.text_widget.filename,
        app.text_widget.focus == TextWidgetField::Filename,
        rows[0],
    );
    render_field(
        frame,
        "Text content (multiline - Enter for a new line)",
        &app.text_widget.content,
        app.text_widget.focus == TextWidgetField::Content,
        rows[1],
    );
    render_field(
        frame,
        "Font size",
        &app.text_widget.font_size,
        app.text_widget.focus == TextWidgetField::FontSize,
        rows[2],
    );
    render_field(
        frame,
        "Line height",
        &app.text_widget.line_height,
        app.text_widget.focus == TextWidgetField::LineHeight,
        rows[3],
    );
    render_field(
        frame,
        "Widget width",
        &app.text_widget.width,
        app.text_widget.focus == TextWidgetField::Width,
        rows[4],
    );
    render_field(
        frame,
        "Widget height",
        &app.text_widget.height,
        app.text_widget.focus == TextWidgetField::Height,
        rows[5],
    );

    let align_line = Paragraph::new(Line::from(format!(
        "Align: {} / {}   (Ctrl+Left/Right, Ctrl+Up/Down to change)",
        halign_label(app.text_widget.halign),
        valign_label(app.text_widget.valign),
    )))
    .style(Style::default().fg(PALETTE.text_secondary).bg(PALETTE.bg));
    frame.render_widget(align_line, rows[6]);

    draw_help(frame, app, rows[7]);
}

fn halign_label(h: HAlign) -> &'static str {
    match h {
        HAlign::Left => "left",
        HAlign::Center => "center",
        HAlign::Right => "right",
    }
}

fn valign_label(v: VAlign) -> &'static str {
    match v {
        VAlign::Top => "top",
        VAlign::Center => "center",
        VAlign::Bottom => "bottom",
    }
}

fn render_field(
    frame: &mut Frame,
    label: &str,
    ta: &tui_textarea::TextArea,
    focused: bool,
    area: Rect,
) {
    let block = theme::focusable_block(label, focused);
    let inner = block.inner(area);
    frame.render_widget(block, area);
    frame.render_widget(ta, inner);
}

fn draw_help(frame: &mut Frame, app: &App, area: Rect) {
    let mut text = "Tab/Shift+Tab: move focus   Ctrl+S: save widget   Esc: main menu".to_string();
    if let Some(status) = &app.status {
        text.push_str("   -   ");
        text.push_str(status);
    }
    let para = Paragraph::new(Line::from(text))
        .style(Style::default().fg(PALETTE.text_secondary).bg(PALETTE.bg));
    frame.render_widget(para, area);
}
