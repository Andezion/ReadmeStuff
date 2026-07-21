use crate::app::{App, Field, available_credentials, is_selectable};
use crate::ui::theme::{self, PALETTE};
use ratatui::Frame;
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{List, ListItem, ListState, Paragraph};
use readme_stuff_catalog::registry::{self, WidgetGroup, WidgetSpec};
use readme_stuff_config::Credential;
use readme_stuff_config::Requirement;

fn group_label(g: WidgetGroup) -> &'static str {
    match g {
        WidgetGroup::GitHub => "GitHub",
        WidgetGroup::Codeforces => "Codeforces",
        WidgetGroup::Codewars => "Codewars",
        WidgetGroup::LeetCode => "LeetCode",
        WidgetGroup::Combined => "Combined",
    }
}

fn credential_label(c: Credential) -> &'static str {
    match c {
        Credential::GitHubToken | Credential::GitHubLogin => "GitHub login+token",
        Credential::CodeforcesHandle => "Codeforces handle",
        Credential::CodewarsUsername => "Codewars username",
        Credential::LeetcodeUsername => "LeetCode username",
    }
}

fn missing_hint(app: &App, spec: &WidgetSpec) -> Option<String> {
    if is_selectable(app, spec) {
        return None;
    }
    let available = available_credentials(app);
    Some(match spec.requires {
        Requirement::All(reqs) => {
            let mut missing: Vec<&str> = reqs
                .iter()
                .filter(|r| !available.contains(r))
                .map(|r| credential_label(*r))
                .collect();
            missing.dedup();
            format!("needs {}", missing.join(", "))
        }
        Requirement::AnyOf(reqs) => {
            let mut opts: Vec<&str> = reqs.iter().map(|r| credential_label(*r)).collect();
            opts.dedup();
            format!("needs any of {}", opts.join(" / "))
        }
    })
}

pub fn draw(frame: &mut Frame, app: &App) {
    let area = frame.area();
    frame.render_widget(ratatui::widgets::Block::default().style(Style::default().bg(PALETTE.bg)), area);

    let cols = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(40), Constraint::Percentage(60)])
        .split(area);

    draw_fields(frame, app, cols[0]);
    draw_widget_list(frame, app, cols[1]);
}

fn draw_fields(frame: &mut Frame, app: &App, area: Rect) {
    let rows = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),
            Constraint::Length(3),
            Constraint::Length(3),
            Constraint::Length(3),
            Constraint::Length(3),
            Constraint::Length(2),
            Constraint::Min(0),
        ])
        .split(area);

    render_field(frame, "GitHub login", &app.github_login, app.focus == Field::GithubLogin, rows[0]);
    render_field(
        frame,
        "GitHub token env var (name only, never the secret)",
        &app.github_token_env,
        app.focus == Field::GithubTokenEnv,
        rows[1],
    );
    render_field(
        frame,
        "Codeforces handle",
        &app.codeforces_handle,
        app.focus == Field::CodeforcesHandle,
        rows[2],
    );
    render_field(
        frame,
        "Codewars username",
        &app.codewars_username,
        app.focus == Field::CodewarsUsername,
        rows[3],
    );
    render_field(
        frame,
        "LeetCode username",
        &app.leetcode_username,
        app.focus == Field::LeetcodeUsername,
        rows[4],
    );

    let help = Paragraph::new(Line::from(
        "Tab/Shift+Tab: move focus   Ctrl+S: save   Esc: quit",
    ))
    .style(Style::default().fg(PALETTE.text_secondary));
    frame.render_widget(help, rows[5]);

    if let Some(status) = &app.status {
        let err = Paragraph::new(Line::from(status.as_str())).style(Style::default().fg(PALETTE.accent));
        frame.render_widget(err, rows[6]);
    }
}

fn render_field(frame: &mut Frame, label: &str, ta: &tui_textarea::TextArea, focused: bool, area: Rect) {
    let block = theme::focusable_block(label, focused);
    let inner = block.inner(area);
    frame.render_widget(block, area);
    frame.render_widget(ta, inner);
}

fn draw_widget_list(frame: &mut Frame, app: &App, area: Rect) {
    let focused = app.focus == Field::WidgetList;
    let block = theme::focusable_block("Widgets to include - [A] Add   [D] Delete   (Space/Enter: toggle)", focused);

    let items: Vec<ListItem> = registry::all_widgets()
        .iter()
        .map(|spec| {
            let checked = app.selected.contains(spec.id);
            let mark = if checked { "x" } else { " " };
            let selectable = is_selectable(app, spec);

            let mut spans = vec![Span::styled(
                format!("[{mark}] "),
                Style::default().fg(if selectable { PALETTE.accent } else { PALETTE.text_secondary }),
            )];
            spans.push(Span::styled(
                format!("{} ", spec.label),
                Style::default().fg(if selectable { PALETTE.text_primary } else { PALETTE.text_secondary }),
            ));
            spans.push(Span::styled(
                format!("({})", group_label(spec.group)),
                Style::default().fg(PALETTE.text_secondary),
            ));
            if let Some(hint) = missing_hint(app, spec) {
                spans.push(Span::styled(format!("  - {hint}"), Style::default().fg(PALETTE.text_secondary)));
            }
            ListItem::new(Line::from(spans))
        })
        .collect();

    let mut state = ListState::default();
    if focused {
        state.select(Some(app.widget_cursor));
    }

    let list = List::new(items)
        .block(block)
        .highlight_style(Style::default().add_modifier(Modifier::REVERSED));

    frame.render_stateful_widget(list, area, &mut state);
}
