use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use readme_stuff_catalog::registry::{self, WidgetSpec};
use readme_stuff_config::{Config, Credential, Layout, PlacedWidget, ProfileConfig, Row, ThemeChoice, io};
use std::collections::HashSet;
use std::path::PathBuf;
use tui_textarea::TextArea;

const CANVAS_WIDTH: u32 = 990;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Screen {
    Welcome,
    Questionnaire,
    Done,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Field {
    GithubLogin,
    GithubTokenEnv,
    CodeforcesHandle,
    CodewarsUsername,
    LeetcodeUsername,
    WidgetList,
}

impl Field {
    const ORDER: [Field; 6] = [
        Field::GithubLogin,
        Field::GithubTokenEnv,
        Field::CodeforcesHandle,
        Field::CodewarsUsername,
        Field::LeetcodeUsername,
        Field::WidgetList,
    ];

    fn index(self) -> usize {
        Field::ORDER.iter().position(|f| *f == self).unwrap()
    }

    pub fn next(self) -> Field {
        Field::ORDER[(self.index() + 1) % Field::ORDER.len()]
    }

    pub fn prev(self) -> Field {
        let n = Field::ORDER.len();
        Field::ORDER[(self.index() + n - 1) % n]
    }
}

pub struct App {
    pub screen: Screen,
    pub should_quit: bool,

    pub found_config_path: Option<PathBuf>,
    pending_resume: Option<Config>,

    pub github_login: TextArea<'static>,
    pub github_token_env: TextArea<'static>,
    pub codeforces_handle: TextArea<'static>,
    pub codewars_username: TextArea<'static>,
    pub leetcode_username: TextArea<'static>,
    pub focus: Field,

    pub selected: HashSet<&'static str>,
    pub widget_cursor: usize,

    pub saved_path: Option<PathBuf>,
    pub status: Option<String>,
}

fn single_line(text: &str) -> TextArea<'static> {
    TextArea::from(vec![text.to_string()])
}

impl App {
    
    pub fn new(existing: Option<(PathBuf, Config)>) -> App {
        let mut app = App {
            screen: Screen::Questionnaire,
            should_quit: false,
            found_config_path: None,
            pending_resume: None,
            github_login: TextArea::default(),
            github_token_env: single_line("GITHUB_TOKEN"),
            codeforces_handle: TextArea::default(),
            codewars_username: TextArea::default(),
            leetcode_username: TextArea::default(),
            focus: Field::GithubLogin,
            selected: HashSet::new(),
            widget_cursor: 0,
            saved_path: None,
            status: None,
        };
        if let Some((path, cfg)) = existing {
            app.found_config_path = Some(path);
            app.pending_resume = Some(cfg);
            app.screen = Screen::Welcome;
        }
        app
    }
}

fn field_text(ta: &TextArea) -> String {
    ta.lines().join("")
}

fn field_opt(ta: &TextArea) -> Option<String> {
    let s = field_text(ta).trim().to_string();
    if s.is_empty() { None } else { Some(s) }
}

pub fn available_credentials(app: &App) -> HashSet<Credential> {
    to_config(app).profile.available_credentials()
}

pub fn is_selectable(app: &App, spec: &WidgetSpec) -> bool {
    spec.requires.is_satisfied(&available_credentials(app))
}

pub fn toggle_selected(app: &mut App, id: &'static str) {
    if app.selected.contains(id) {
        app.selected.remove(id);
        return;
    }
    let Some(spec) = registry::find(id) else { return };
    if is_selectable(app, spec) {
        app.selected.insert(id);
    }
}

pub fn pack_layout(selected: &HashSet<&'static str>) -> Layout {
    let mut rows: Vec<Row> = Vec::new();
    let mut current: Vec<PlacedWidget> = Vec::new();
    let mut x = 0u32;

    for spec in registry::all_widgets() {
        if !selected.contains(spec.id) {
            continue;
        }
        let w = spec.size.0;
        if !current.is_empty() && x + w > CANVAS_WIDTH {
            rows.push(Row { widgets: std::mem::take(&mut current) });
            x = 0;
        }
        current.push(PlacedWidget { id: spec.id.to_string(), x, y: 0 });
        x += w;
    }
    if !current.is_empty() {
        rows.push(Row { widgets: current });
    }

    Layout { canvas_width: CANVAS_WIDTH, rows }
}

pub fn to_config(app: &App) -> Config {
    Config {
        profile: ProfileConfig {
            github_login: field_opt(&app.github_login),
            github_token_env: field_opt(&app.github_token_env),
            codeforces_handle: field_opt(&app.codeforces_handle),
            codewars_username: field_opt(&app.codewars_username),
            leetcode_username: field_opt(&app.leetcode_username),
        },
        theme: ThemeChoice::Matrix,
        layout: pack_layout(&app.selected),
    }
}

pub fn load_into(app: &mut App, cfg: &Config) {
    app.github_login = single_line(cfg.profile.github_login.as_deref().unwrap_or(""));
    app.github_token_env = single_line(cfg.profile.github_token_env.as_deref().unwrap_or("GITHUB_TOKEN"));
    app.codeforces_handle = single_line(cfg.profile.codeforces_handle.as_deref().unwrap_or(""));
    app.codewars_username = single_line(cfg.profile.codewars_username.as_deref().unwrap_or(""));
    app.leetcode_username = single_line(cfg.profile.leetcode_username.as_deref().unwrap_or(""));
    app.selected = cfg
        .layout
        .rows
        .iter()
        .flat_map(|r| r.widgets.iter())
        .filter_map(|pw| registry::find(&pw.id).map(|s| s.id))
        .collect();
}

fn save_current_dir(app: &mut App) {
    let cfg = to_config(app);
    let path = std::env::current_dir().unwrap_or_default().join(io::CONFIG_FILE_NAME);
    match io::save(&path, &cfg) {
        Ok(()) => {
            app.saved_path = Some(path);
            app.status = None;
            app.screen = Screen::Done;
        }
        Err(e) => app.status = Some(format!("save failed: {e}")),
    }
}

pub fn handle_key(app: &mut App, key: KeyEvent) {
    match app.screen {
        Screen::Welcome => handle_welcome_key(app, key),
        Screen::Questionnaire => handle_questionnaire_key(app, key),
        Screen::Done => handle_done_key(app, key),
    }
}

fn handle_welcome_key(app: &mut App, key: KeyEvent) {
    match key.code {
        KeyCode::Char('r') | KeyCode::Char('R') => {
            if let Some(cfg) = app.pending_resume.take() {
                load_into(app, &cfg);
            }
            app.screen = Screen::Questionnaire;
        }
        KeyCode::Char('n') | KeyCode::Char('N') => {
            app.pending_resume = None;
            app.screen = Screen::Questionnaire;
        }
        KeyCode::Esc | KeyCode::Char('q') => app.should_quit = true,
        _ => {}
    }
}

fn handle_questionnaire_key(app: &mut App, key: KeyEvent) {
    if key.code == KeyCode::Char('s') && key.modifiers.contains(KeyModifiers::CONTROL) {
        save_current_dir(app);
        return;
    }
    if key.code == KeyCode::Esc {
        app.should_quit = true;
        return;
    }
    if key.code == KeyCode::Tab {
        app.focus = app.focus.next();
        return;
    }
    if key.code == KeyCode::BackTab {
        app.focus = app.focus.prev();
        return;
    }

    if app.focus == Field::WidgetList {
        let len = registry::all_widgets().len();
        match key.code {
            KeyCode::Down | KeyCode::Char('j') => {
                app.widget_cursor = (app.widget_cursor + 1).min(len.saturating_sub(1));
            }
            KeyCode::Up | KeyCode::Char('k') => {
                app.widget_cursor = app.widget_cursor.saturating_sub(1);
            }
            KeyCode::Char(' ') | KeyCode::Enter => {
                if let Some(spec) = registry::all_widgets().get(app.widget_cursor) {
                    toggle_selected(app, spec.id);
                }
            }
            _ => {}
        }
        return;
    }

    match key.code {
        KeyCode::Down | KeyCode::Enter => app.focus = app.focus.next(),
        KeyCode::Up => app.focus = app.focus.prev(),
        _ => {
            let field = match app.focus {
                Field::GithubLogin => &mut app.github_login,
                Field::GithubTokenEnv => &mut app.github_token_env,
                Field::CodeforcesHandle => &mut app.codeforces_handle,
                Field::CodewarsUsername => &mut app.codewars_username,
                Field::LeetcodeUsername => &mut app.leetcode_username,
                Field::WidgetList => unreachable!(),
            };
            field.input(key);
        }
    }
}

fn handle_done_key(app: &mut App, key: KeyEvent) {
    match key.code {
        KeyCode::Char('b') | KeyCode::Char('B') => app.screen = Screen::Questionnaire,
        KeyCode::Char('q') | KeyCode::Char('Q') | KeyCode::Esc => app.should_quit = true,
        _ => {}
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn app_with(github_login: &str, github_token_env: &str) -> App {
        let mut app = App::new(None);
        app.github_login = single_line(github_login);
        app.github_token_env = single_line(github_token_env);
        app
    }

    #[test]
    fn no_existing_config_starts_on_questionnaire() {
        let app = App::new(None);
        assert_eq!(app.screen, Screen::Questionnaire);
        assert_eq!(field_text(&app.github_token_env), "GITHUB_TOKEN");
    }

    #[test]
    fn existing_config_starts_on_welcome_and_is_not_applied_until_resumed() {
        let cfg = Config {
            profile: ProfileConfig {
                github_login: Some("octocat".to_string()),
                ..Default::default()
            },
            ..Default::default()
        };
        let app = App::new(Some((PathBuf::from("readme.toml"), cfg)));
        assert_eq!(app.screen, Screen::Welcome);
        assert_eq!(field_text(&app.github_login), "");
    }

    #[test]
    fn resume_prefills_fields_and_selection_from_loaded_config() {
        let cfg = Config {
            profile: ProfileConfig {
                github_login: Some("octocat".to_string()),
                github_token_env: Some("GITHUB_TOKEN".to_string()),
                ..Default::default()
            },
            theme: ThemeChoice::Matrix,
            layout: pack_layout(&HashSet::from(["github-stats"])),
        };
        let mut app = App::new(Some((PathBuf::from("readme.toml"), cfg)));
        handle_key(&mut app, KeyEvent::from(KeyCode::Char('r')));
        assert_eq!(app.screen, Screen::Questionnaire);
        assert_eq!(field_text(&app.github_login), "octocat");
        assert!(app.selected.contains("github-stats"));
    }

    #[test]
    fn new_on_welcome_discards_loaded_config() {
        let cfg = Config {
            profile: ProfileConfig {
                github_login: Some("octocat".to_string()),
                ..Default::default()
            },
            ..Default::default()
        };
        let mut app = App::new(Some((PathBuf::from("readme.toml"), cfg)));
        handle_key(&mut app, KeyEvent::from(KeyCode::Char('n')));
        assert_eq!(app.screen, Screen::Questionnaire);
        assert_eq!(field_text(&app.github_login), "");
    }

    #[test]
    fn widget_requiring_unset_credential_is_not_selectable() {
        let app = app_with("", "");
        let spec = registry::find("cf-rating").unwrap();
        assert!(!is_selectable(&app, spec));
    }

    #[test]
    fn widget_becomes_selectable_once_its_credential_is_set() {
        let app = app_with("", "");
        let mut app = app;
        app.codeforces_handle = single_line("Andezion");
        let spec = registry::find("cf-rating").unwrap();
        assert!(is_selectable(&app, spec));
    }

    #[test]
    fn toggle_selected_is_a_no_op_when_not_selectable() {
        let mut app = app_with("", "");
        toggle_selected(&mut app, "cf-rating");
        assert!(!app.selected.contains("cf-rating"));
    }

    #[test]
    fn toggle_selected_adds_then_removes() {
        let mut app = app_with("octocat", "GITHUB_TOKEN");
        toggle_selected(&mut app, "github-stats");
        assert!(app.selected.contains("github-stats"));
        toggle_selected(&mut app, "github-stats");
        assert!(!app.selected.contains("github-stats"));
    }

    #[test]
    fn pack_layout_is_empty_for_empty_selection() {
        let layout = pack_layout(&HashSet::new());
        assert!(layout.rows.is_empty());
        assert_eq!(layout.canvas_width, CANVAS_WIDTH);
    }

    #[test]
    fn pack_layout_never_exceeds_canvas_width_per_row() {
        let selected: HashSet<&'static str> = registry::all_widgets().iter().map(|w| w.id).collect();
        let layout = pack_layout(&selected);
        assert!(layout.rows.len() > 1, "expected wrapping across multiple rows");
        for row in &layout.rows {
            let ids: HashSet<_> = row.widgets.iter().map(|w| w.id.as_str()).collect();
            assert_eq!(ids.len(), row.widgets.len(), "no duplicate placements within a row");
            for w in &row.widgets {
                let spec = registry::find(&w.id).unwrap();
                assert!(
                    w.x + spec.size.0 <= CANVAS_WIDTH,
                    "{} at x={} width={} overflows canvas",
                    w.id,
                    w.x,
                    spec.size.0
                );
            }
        }
    }

    #[test]
    fn to_config_round_trips_through_load_into() {
        let mut app = app_with("octocat", "GITHUB_TOKEN");
        toggle_selected(&mut app, "github-stats");
        toggle_selected(&mut app, "github-repos");
        let cfg = to_config(&app);

        let mut reloaded = App::new(None);
        load_into(&mut reloaded, &cfg);
        assert_eq!(field_text(&reloaded.github_login), "octocat");
        assert_eq!(reloaded.selected, app.selected);
    }
}
