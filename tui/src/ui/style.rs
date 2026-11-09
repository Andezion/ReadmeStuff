use crate::app::{App, StyleField};
use crate::ui::theme::{self, PALETTE};
use ratatui::Frame;
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span, Text};
use ratatui::widgets::Paragraph;
use readme_stuff_config::{ColorChoice, ThemeChoice, mosaic_file_name};

pub fn draw(frame: &mut Frame, app: &mut App) {
    let area = frame.area();
    let rows = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Min(5),
            Constraint::Length(3),
            Constraint::Length(1),
        ])
        .split(area);
    let cols = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(40), Constraint::Percentage(60)])
        .split(rows[0]);

    draw_themes(frame, app, cols[0]);
    draw_colors(frame, app, cols[1]);
    draw_preview(frame, app, rows[1]);

    let help = Paragraph::new(Line::from(
        "Up/Down/j/k: choose   Tab/Left/Right: switch list   Enter/Esc: back (saved with the next build)",
    ))
    .style(Style::default().fg(PALETTE.text_secondary).bg(PALETTE.bg));
    frame.render_widget(help, rows[2]);
}

fn row_style(selected: bool, focused_list: bool) -> Style {
    let mut style = Style::default().fg(PALETTE.text_primary);
    if selected {
        style = style.add_modifier(Modifier::BOLD);
        if focused_list {
            style = style.add_modifier(Modifier::REVERSED);
        }
    }
    style
}

fn marker(selected: bool) -> Span<'static> {
    Span::styled(
        if selected { "(*) " } else { "( ) " },
        Style::default().fg(if selected {
            PALETTE.accent
        } else {
            PALETTE.text_secondary
        }),
    )
}

fn draw_themes(frame: &mut Frame, app: &App, area: Rect) {
    let focused = app.style_focus == StyleField::Theme;
    let lines: Vec<Line> = ThemeChoice::ALL
        .iter()
        .map(|&t| {
            let selected = t == app.theme;
            Line::from(vec![
                marker(selected),
                Span::styled(t.slug(), row_style(selected, focused)),
                Span::styled(
                    format!("  {}", theme_description(t)),
                    Style::default().fg(PALETTE.text_secondary),
                ),
            ])
        })
        .collect();
    let para = Paragraph::new(Text::from(lines))
        .style(Style::default().bg(PALETTE.bg))
        .block(theme::focusable_block("theme", focused));
    frame.render_widget(para, area);
}

fn theme_description(t: ThemeChoice) -> &'static str {
    match t {
        ThemeChoice::Matrix => "falling glyph rain",
    }
}

fn draw_colors(frame: &mut Frame, app: &App, area: Rect) {
    let focused = app.style_focus == StyleField::Color;
    let lines: Vec<Line> = ColorChoice::ALL
        .iter()
        .map(|&c| {
            let selected = c == app.color;
            let mut spans = vec![marker(selected)];
            spans.extend(swatch(c));
            spans.push(Span::raw(" "));
            spans.push(Span::styled(c.slug(), row_style(selected, focused)));
            Line::from(spans)
        })
        .collect();
    let para = Paragraph::new(Text::from(lines))
        .style(Style::default().bg(PALETTE.bg))
        .block(theme::focusable_block("backdrop color", focused));
    frame.render_widget(para, area);
}

fn swatch(c: ColorChoice) -> Vec<Span<'static>> {
    match hex(c) {
        Some(color) => vec![Span::styled("████", Style::default().fg(color))],
        None => RAINBOW
            .iter()
            .map(|&col| Span::styled("█", Style::default().fg(col)))
            .collect(),
    }
}

const RAINBOW: [Color; 4] = [
    Color::Rgb(0xff, 0x1a, 0x1a),
    Color::Rgb(0xff, 0xd6, 0x00),
    Color::Rgb(0x1a, 0xff, 0x8c),
    Color::Rgb(0x8c, 0x1a, 0xff),
];

fn hex(c: ColorChoice) -> Option<Color> {
    let theme = readme_stuff_catalog::draw_theme(ThemeChoice::Matrix, c, Default::default());
    let raw = theme.tint.hex()?;
    let v = u32::from_str_radix(raw.trim_start_matches('#'), 16).ok()?;
    Some(Color::Rgb((v >> 16) as u8, (v >> 8) as u8, v as u8))
}

fn draw_preview(frame: &mut Frame, app: &App, area: Rect) {
    let mut spans = vec![Span::raw("mosaic will be saved as ")];
    spans.push(Span::styled(
        mosaic_file_name(app.theme, app.color),
        Style::default()
            .fg(PALETTE.title)
            .add_modifier(Modifier::BOLD),
    ));
    spans.push(Span::raw("   "));
    spans.extend(swatch(app.color));
    let para = Paragraph::new(Line::from(spans))
        .style(Style::default().fg(PALETTE.text_primary).bg(PALETTE.bg))
        .block(theme::block("preview"));
    frame.render_widget(para, area);
}
