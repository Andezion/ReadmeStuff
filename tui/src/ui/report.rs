use crate::app::App;
use crate::ui::theme::{self, PALETTE};
use ratatui::Frame;
use ratatui::layout::Rect;
use ratatui::style::{Color, Style};
use ratatui::text::{Line, Span, Text};
use ratatui::widgets::{Paragraph, Wrap};
use readme_stuff_catalog::WidgetOutcome;

pub fn draw(frame: &mut Frame, app: &App) {
    let area = frame.area();
    let block = theme::block("build result");
    let inner = block.inner(area);
    frame.render_widget(block, area);

    match &app.build_output {
        None => {}
        Some(Err(e)) => draw_error(frame, inner, e),
        Some(Ok(output)) => draw_output(frame, inner, output),
    }
}

fn draw_error(frame: &mut Frame, area: Rect, message: &str) {
    let text = Text::from(vec![
        Line::from(""),
        Line::styled(
            format!("Build failed: {message}"),
            Style::default().fg(Color::Red),
        ),
        Line::from(""),
        Line::from("[B] Back     [Q] Quit"),
    ]);
    let para = Paragraph::new(text)
        .style(Style::default().fg(PALETTE.text_primary).bg(PALETTE.bg))
        .wrap(Wrap { trim: false });
    frame.render_widget(para, area);
}

fn draw_output(frame: &mut Frame, area: Rect, output: &readme_stuff_catalog::BuildOutput) {
    let mut lines = vec![
        Line::from(format!("Output dir: {}", output.out_dir.display())),
        Line::from(""),
    ];

    for outcome in &output.widgets {
        lines.push(match outcome {
            WidgetOutcome::Written { id, path } => Line::styled(
                format!("  [OK]    {id} -> {}", path.display()),
                Style::default().fg(PALETTE.accent),
            ),
            WidgetOutcome::Skipped { id, reason } => Line::styled(
                format!("  [SKIP]  {id} ({reason})"),
                Style::default().fg(PALETTE.text_secondary),
            ),
            WidgetOutcome::Error { id, reason } => Line::styled(
                format!("  [ERROR] {id} ({reason})"),
                Style::default().fg(Color::Red),
            ),
        });
    }

    lines.push(Line::from(""));
    lines.push(match &output.mosaic_path {
        Some(p) => Line::styled(
            format!("Mosaic: {}", p.display()),
            Style::default().fg(PALETTE.title),
        ),
        None => Line::styled(
            "Mosaic: not built (no rows produced output)",
            Style::default().fg(PALETTE.text_secondary),
        ),
    });
    lines.push(Line::from(""));
    lines.push(Line::from(vec![Span::raw("[B] Back to edit     [Q] Quit")]));

    let para = Paragraph::new(Text::from(lines))
        .style(Style::default().fg(PALETTE.text_primary).bg(PALETTE.bg));
    frame.render_widget(para, area);
}
