use crossterm::event::{KeyCode, KeyEvent, KeyModifiers, MouseButton, MouseEvent, MouseEventKind};
use readme_stuff_catalog::BuildOutput;
use readme_stuff_catalog::registry::{self, WidgetSpec};
use readme_stuff_catalog::{DiscoveredWidget, Rect as PixelRect};
use readme_stuff_config::{
    Config, Credential, Layout, PlacedWidget, ProfileConfig, Row, TextCardConfig, ThemeChoice, io,
};
use readme_stuff_draw::{Align, HAlign, PlacedTile, Theme, VAlign};
use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};
use tui_textarea::TextArea;

const CANVAS_WIDTH: u32 = 990;
const LAYOUT_STEP: u32 = 15;
const SCROLL_STEP: u32 = 30;
const README_EXPORT_NAME: &str = "README.svg";
const EXPORT_BG: &str = "#0d1117";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Screen {
    MainMenu,
    Welcome,
    Questionnaire,
    Layout,
    Building,
    Report,
    TextWidgetGen,
    WidgetSelect,
    ReadmeEditor,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Field {
    GithubLogin,
    GithubTokenEnv,
    CodeforcesHandle,
    CodewarsUsername,
    LeetcodeUsername,
    TextCardFile,
    WidgetList,
}

impl Field {
    const ORDER: [Field; 7] = [
        Field::GithubLogin,
        Field::GithubTokenEnv,
        Field::CodeforcesHandle,
        Field::CodewarsUsername,
        Field::LeetcodeUsername,
        Field::TextCardFile,
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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TextWidgetField {
    Filename,
    Content,
    FontSize,
    LineHeight,
    Width,
    Height,
}

impl TextWidgetField {
    const ORDER: [TextWidgetField; 6] = [
        TextWidgetField::Filename,
        TextWidgetField::Content,
        TextWidgetField::FontSize,
        TextWidgetField::LineHeight,
        TextWidgetField::Width,
        TextWidgetField::Height,
    ];

    fn index(self) -> usize {
        TextWidgetField::ORDER
            .iter()
            .position(|f| *f == self)
            .unwrap()
    }

    pub fn next(self) -> TextWidgetField {
        TextWidgetField::ORDER[(self.index() + 1) % TextWidgetField::ORDER.len()]
    }

    pub fn prev(self) -> TextWidgetField {
        let n = TextWidgetField::ORDER.len();
        TextWidgetField::ORDER[(self.index() + n - 1) % n]
    }
}

pub struct TextWidgetForm {
    pub filename: TextArea<'static>,
    pub content: TextArea<'static>,
    pub font_size: TextArea<'static>,
    pub line_height: TextArea<'static>,
    pub width: TextArea<'static>,
    pub height: TextArea<'static>,
    pub focus: TextWidgetField,
    pub halign: HAlign,
    pub valign: VAlign,
}

impl TextWidgetForm {
    pub fn new() -> TextWidgetForm {
        TextWidgetForm {
            filename: TextArea::default(),
            content: TextArea::default(),
            font_size: single_line("16"),
            line_height: single_line("20"),
            width: single_line(&readme_stuff_draw::DEFAULT_WIDTH.to_string()),
            height: single_line(&readme_stuff_draw::DEFAULT_HEIGHT.to_string()),
            focus: TextWidgetField::Filename,
            halign: HAlign::Left,
            valign: VAlign::Top,
        }
    }
}

impl Default for TextWidgetForm {
    fn default() -> Self {
        TextWidgetForm::new()
    }
}

#[derive(Debug, Clone, Copy, Default)]
pub struct ScreenRect {
    pub x: u16,
    pub y: u16,
    pub w: u16,
    pub h: u16,
}

impl ScreenRect {
    pub fn contains(&self, col: u16, row: u16) -> bool {
        col >= self.x && col < self.x + self.w && row >= self.y && row < self.y + self.h
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PlacedItem {
    pub id: String,
    pub x: i32,
    pub y: i32,
    pub w: u32,
    pub h: u32,
}

#[derive(Debug, Clone)]
pub struct DragState {
    pub id: String,
    pub from_sidebar: bool,
    pub grab_dx: i32,
    pub grab_dy: i32,
    pub w: u32,
    pub h: u32,
    pub current: (i32, i32),
    pub valid: bool,
    pub moved: bool,
    pub original: Option<(i32, i32)>,
}

pub struct EditorState {
    pub items: HashMap<String, DiscoveredWidget>,
    pub sidebar: Vec<String>,
    pub placed: Vec<PlacedItem>,
    pub drag: Option<DragState>,
    pub scroll_y: u32,
    pub canvas_width: u32,
    pub canvas_area: ScreenRect,
    pub sidebar_items: Vec<(String, ScreenRect)>,
    pub sidebar_scroll: usize,
    pub sidebar_area: ScreenRect,
}

impl EditorState {
    fn new() -> EditorState {
        EditorState {
            items: HashMap::new(),
            sidebar: Vec::new(),
            placed: Vec::new(),
            drag: None,
            scroll_y: 0,
            canvas_width: CANVAS_WIDTH,
            canvas_area: ScreenRect::default(),
            sidebar_items: Vec::new(),
            sidebar_scroll: 0,
            sidebar_area: ScreenRect::default(),
        }
    }
}

pub struct App {
    pub screen: Screen,
    pub should_quit: bool,

    pub found_config_path: Option<PathBuf>,
    pending_resume: Option<Config>,

    pub pending_build: Option<Config>,
    pub build_output: Option<Result<BuildOutput, String>>,
    pub build_tick: u32,

    pub github_login: TextArea<'static>,
    pub github_token_env: TextArea<'static>,
    pub codeforces_handle: TextArea<'static>,
    pub codewars_username: TextArea<'static>,
    pub leetcode_username: TextArea<'static>,
    pub text_card_file: TextArea<'static>,
    pub focus: Field,

    pub selected: HashSet<&'static str>,
    pub widget_cursor: usize,

    pub layout_positions: HashMap<&'static str, (u32, u32)>,
    pub layout_cursor: usize,

    pub saved_path: Option<PathBuf>,
    pub status: Option<String>,

    pub text_widget: TextWidgetForm,

    pub discovered: Vec<DiscoveredWidget>,
    pub discovered_selected: HashSet<String>,
    pub discovered_cursor: usize,
    pub discovered_scroll: usize,
    pub discovered_rows: Vec<(usize, ScreenRect)>,

    pub editor: EditorState,
}

fn single_line(text: &str) -> TextArea<'static> {
    TextArea::from(vec![text.to_string()])
}

impl App {
    pub fn new(existing: Option<(PathBuf, Config)>) -> App {
        let mut app = App {
            screen: Screen::MainMenu,
            should_quit: false,
            found_config_path: None,
            pending_resume: None,
            pending_build: None,
            build_output: None,
            build_tick: 0,
            github_login: TextArea::default(),
            github_token_env: single_line("GITHUB_TOKEN"),
            codeforces_handle: TextArea::default(),
            codewars_username: TextArea::default(),
            leetcode_username: TextArea::default(),
            text_card_file: TextArea::default(),
            focus: Field::GithubLogin,
            selected: HashSet::new(),
            widget_cursor: 0,
            layout_positions: HashMap::new(),
            layout_cursor: 0,
            saved_path: None,
            status: None,
            text_widget: TextWidgetForm::new(),
            discovered: Vec::new(),
            discovered_selected: HashSet::new(),
            discovered_cursor: 0,
            discovered_scroll: 0,
            discovered_rows: Vec::new(),
            editor: EditorState::new(),
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
    let Some(spec) = registry::find(id) else {
        return;
    };
    if is_selectable(app, spec) {
        app.selected.insert(id);
    }
}

pub fn add_selected(app: &mut App, id: &'static str) {
    let Some(spec) = registry::find(id) else {
        return;
    };
    if is_selectable(app, spec) {
        app.selected.insert(id);
    }
}

pub fn remove_selected(app: &mut App, id: &'static str) {
    app.selected.remove(id);
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
            rows.push(Row {
                widgets: std::mem::take(&mut current),
            });
            x = 0;
        }
        current.push(PlacedWidget {
            id: spec.id.to_string(),
            x,
            y: 0,
        });
        x += w;
    }
    if !current.is_empty() {
        rows.push(Row { widgets: current });
    }

    Layout {
        canvas_width: CANVAS_WIDTH,
        rows,
    }
}

fn auto_positions(selected: &HashSet<&'static str>) -> Vec<(&'static str, u32, u32)> {
    let mut out = Vec::new();
    let mut x = 0u32;
    let mut y = 0u32;
    let mut row_h = 0u32;

    for spec in registry::all_widgets() {
        if !selected.contains(spec.id) {
            continue;
        }
        let (w, h) = spec.size;
        if x > 0 && x + w > CANVAS_WIDTH {
            x = 0;
            y += row_h;
            row_h = 0;
        }
        out.push((spec.id, x, y));
        x += w;
        row_h = row_h.max(h);
    }
    out
}

fn layout_order(app: &App) -> Vec<&'static str> {
    registry::all_widgets()
        .iter()
        .filter(|spec| app.selected.contains(spec.id))
        .map(|spec| spec.id)
        .collect()
}

pub fn sync_layout_positions(app: &mut App) {
    app.layout_positions
        .retain(|id, _| app.selected.contains(id));
    for (id, x, y) in auto_positions(&app.selected) {
        app.layout_positions.entry(id).or_insert((x, y));
    }
    let len = layout_order(app).len();
    if app.layout_cursor >= len {
        app.layout_cursor = len.saturating_sub(1);
    }
}

fn rects_overlap(a: (u32, u32, u32, u32), b: (u32, u32, u32, u32)) -> bool {
    let (ax, ay, aw, ah) = a;
    let (bx, by, bw, bh) = b;
    ax < bx + bw && bx < ax + aw && ay < by + bh && by < ay + ah
}

pub fn move_layout_selected(app: &mut App, dx: i32, dy: i32) {
    let order = layout_order(app);
    let Some(&id) = order.get(app.layout_cursor) else {
        return;
    };
    let Some(spec) = registry::find(id) else {
        return;
    };
    let (w, h) = spec.size;
    let (cur_x, cur_y) = app.layout_positions.get(id).copied().unwrap_or((0, 0));

    let max_x = CANVAS_WIDTH.saturating_sub(w) as i32;
    let new_x = (cur_x as i32 + dx).clamp(0, max_x) as u32;
    let new_y = (cur_y as i32 + dy).max(0) as u32;
    let candidate = (new_x, new_y, w, h);

    for &other_id in &order {
        if other_id == id {
            continue;
        }
        let (Some(other_spec), Some(&(ox, oy))) =
            (registry::find(other_id), app.layout_positions.get(other_id))
        else {
            continue;
        };
        if rects_overlap(candidate, (ox, oy, other_spec.size.0, other_spec.size.1)) {
            app.status = Some(format!("blocked by {other_id}"));
            return;
        }
    }

    app.layout_positions.insert(id, (new_x, new_y));
    app.status = None;
}

fn manual_layout(app: &App) -> Layout {
    let widgets: Vec<PlacedWidget> = layout_order(app)
        .into_iter()
        .filter_map(|id| {
            app.layout_positions.get(id).map(|&(x, y)| PlacedWidget {
                id: id.to_string(),
                x,
                y,
            })
        })
        .collect();

    Layout {
        canvas_width: CANVAS_WIDTH,
        rows: if widgets.is_empty() {
            vec![]
        } else {
            vec![Row { widgets }]
        },
    }
}

fn base_config(app: &App, layout: Layout) -> Config {
    Config {
        profile: ProfileConfig {
            github_login: field_opt(&app.github_login),
            github_token_env: field_opt(&app.github_token_env),
            codeforces_handle: field_opt(&app.codeforces_handle),
            codewars_username: field_opt(&app.codewars_username),
            leetcode_username: field_opt(&app.leetcode_username),
        },
        theme: ThemeChoice::Matrix,
        layout,
        text_card: TextCardConfig {
            file: field_opt(&app.text_card_file),
            ..Default::default()
        },
    }
}

pub fn to_config(app: &App) -> Config {
    base_config(app, pack_layout(&app.selected))
}

pub fn to_config_with_manual_layout(app: &App) -> Config {
    base_config(app, manual_layout(app))
}

pub fn load_into(app: &mut App, cfg: &Config) {
    app.github_login = single_line(cfg.profile.github_login.as_deref().unwrap_or(""));
    app.github_token_env = single_line(
        cfg.profile
            .github_token_env
            .as_deref()
            .unwrap_or("GITHUB_TOKEN"),
    );
    app.codeforces_handle = single_line(cfg.profile.codeforces_handle.as_deref().unwrap_or(""));
    app.codewars_username = single_line(cfg.profile.codewars_username.as_deref().unwrap_or(""));
    app.leetcode_username = single_line(cfg.profile.leetcode_username.as_deref().unwrap_or(""));
    app.text_card_file = single_line(cfg.text_card.file.as_deref().unwrap_or(""));
    app.selected = cfg
        .layout
        .rows
        .iter()
        .flat_map(|r| r.widgets.iter())
        .filter_map(|pw| registry::find(&pw.id).map(|s| s.id))
        .collect();
}

fn save_and_queue_build(app: &mut App) {
    let dir = std::env::current_dir().unwrap_or_default();
    save_and_queue_build_in(app, &dir);
}

fn save_and_queue_build_in(app: &mut App, dir: &Path) {
    save_config_and_queue(app, dir, to_config(app));
}

fn save_and_queue_build_with_layout(app: &mut App) {
    let dir = std::env::current_dir().unwrap_or_default();
    save_and_queue_build_with_layout_in(app, &dir);
}

fn save_and_queue_build_with_layout_in(app: &mut App, dir: &Path) {
    save_config_and_queue(app, dir, to_config_with_manual_layout(app));
}

fn save_config_and_queue(app: &mut App, dir: &Path, cfg: Config) {
    let path = dir.join(io::CONFIG_FILE_NAME);
    match io::save(&path, &cfg) {
        Ok(()) => {
            app.saved_path = Some(path);
            app.status = None;
            app.pending_build = Some(cfg);
            app.build_tick = 0;
            app.screen = Screen::Building;
        }
        Err(e) => app.status = Some(format!("save failed: {e}")),
    }
}

pub fn apply_build_result(app: &mut App, result: Result<BuildOutput, String>) {
    app.build_output = Some(result);
    app.screen = Screen::Report;
}

pub fn tick_building(app: &mut App) {
    app.build_tick = app.build_tick.wrapping_add(1);
}

pub fn handle_key(app: &mut App, key: KeyEvent) {
    match app.screen {
        Screen::MainMenu => handle_main_menu_key(app, key),
        Screen::Welcome => handle_welcome_key(app, key),
        Screen::Questionnaire => handle_questionnaire_key(app, key),
        Screen::Layout => handle_layout_key(app, key),
        Screen::Building => handle_building_key(app, key),
        Screen::Report => handle_report_key(app, key),
        Screen::TextWidgetGen => handle_text_widget_key(app, key),
        Screen::WidgetSelect => handle_widget_select_key(app, key),
        Screen::ReadmeEditor => handle_readme_editor_key(app, key),
    }
}

fn handle_main_menu_key(app: &mut App, key: KeyEvent) {
    match key.code {
        KeyCode::Char('g') | KeyCode::Char('G') => app.screen = Screen::Questionnaire,
        KeyCode::Char('t') | KeyCode::Char('T') => {
            app.text_widget = TextWidgetForm::new();
            app.status = None;
            app.screen = Screen::TextWidgetGen;
        }
        KeyCode::Char('w') | KeyCode::Char('W') => {
            rescan_discovered(app);
            app.discovered_selected.clear();
            app.discovered_cursor = 0;
            app.discovered_scroll = 0;
            app.status = None;
            app.screen = Screen::WidgetSelect;
        }
        KeyCode::Esc | KeyCode::Char('q') | KeyCode::Char('Q') => app.should_quit = true,
        _ => {}
    }
}

fn handle_welcome_key(app: &mut App, key: KeyEvent) {
    match key.code {
        KeyCode::Char('r') | KeyCode::Char('R') => {
            if let Some(cfg) = app.pending_resume.take() {
                load_into(app, &cfg);
            }
            app.screen = Screen::MainMenu;
        }
        KeyCode::Char('n') | KeyCode::Char('N') => {
            app.pending_resume = None;
            app.screen = Screen::MainMenu;
        }
        KeyCode::Esc | KeyCode::Char('q') => app.should_quit = true,
        _ => {}
    }
}

fn handle_questionnaire_key(app: &mut App, key: KeyEvent) {
    if key.code == KeyCode::Char('s') && key.modifiers.contains(KeyModifiers::CONTROL) {
        save_and_queue_build(app);
        return;
    }
    if key.code == KeyCode::Esc {
        app.screen = Screen::MainMenu;
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
            KeyCode::Char('a') | KeyCode::Char('A') => {
                if let Some(spec) = registry::all_widgets().get(app.widget_cursor) {
                    add_selected(app, spec.id);
                }
            }
            KeyCode::Char('d') | KeyCode::Char('D') => {
                if let Some(spec) = registry::all_widgets().get(app.widget_cursor) {
                    remove_selected(app, spec.id);
                }
            }
            KeyCode::Char('l') | KeyCode::Char('L') => {
                if app.selected.is_empty() {
                    app.status = Some("select at least one widget first".to_string());
                } else {
                    sync_layout_positions(app);
                    app.screen = Screen::Layout;
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
                Field::TextCardFile => &mut app.text_card_file,
                Field::WidgetList => unreachable!(),
            };
            field.input(key);
        }
    }
}

fn handle_layout_key(app: &mut App, key: KeyEvent) {
    if key.code == KeyCode::Char('s') && key.modifiers.contains(KeyModifiers::CONTROL) {
        save_and_queue_build_with_layout(app);
        return;
    }

    let len = layout_order(app).len();
    match key.code {
        KeyCode::Esc => app.screen = Screen::Questionnaire,
        KeyCode::Tab if len > 0 => app.layout_cursor = (app.layout_cursor + 1) % len,
        KeyCode::BackTab if len > 0 => app.layout_cursor = (app.layout_cursor + len - 1) % len,
        KeyCode::Left | KeyCode::Char('h') => move_layout_selected(app, -(LAYOUT_STEP as i32), 0),
        KeyCode::Right | KeyCode::Char('l') => move_layout_selected(app, LAYOUT_STEP as i32, 0),
        KeyCode::Up | KeyCode::Char('k') => move_layout_selected(app, 0, -(LAYOUT_STEP as i32)),
        KeyCode::Down | KeyCode::Char('j') => move_layout_selected(app, 0, LAYOUT_STEP as i32),
        _ => {}
    }
}

fn handle_building_key(app: &mut App, key: KeyEvent) {
    if key.code == KeyCode::Esc {
        app.should_quit = true;
    }
}

fn handle_report_key(app: &mut App, key: KeyEvent) {
    match key.code {
        KeyCode::Char('b') | KeyCode::Char('B') => {
            app.screen = Screen::Questionnaire;
            app.build_output = None;
            app.pending_build = None;
            app.build_tick = 0;
        }
        KeyCode::Char('q') | KeyCode::Char('Q') | KeyCode::Esc => app.should_quit = true,
        _ => {}
    }
}

fn next_halign(h: HAlign) -> HAlign {
    match h {
        HAlign::Left => HAlign::Center,
        HAlign::Center => HAlign::Right,
        HAlign::Right => HAlign::Left,
    }
}

fn prev_halign(h: HAlign) -> HAlign {
    match h {
        HAlign::Left => HAlign::Right,
        HAlign::Right => HAlign::Center,
        HAlign::Center => HAlign::Left,
    }
}

fn next_valign(v: VAlign) -> VAlign {
    match v {
        VAlign::Top => VAlign::Center,
        VAlign::Center => VAlign::Bottom,
        VAlign::Bottom => VAlign::Top,
    }
}

fn prev_valign(v: VAlign) -> VAlign {
    match v {
        VAlign::Top => VAlign::Bottom,
        VAlign::Bottom => VAlign::Center,
        VAlign::Center => VAlign::Top,
    }
}

fn handle_text_widget_key(app: &mut App, key: KeyEvent) {
    if key.code == KeyCode::Esc {
        app.screen = Screen::MainMenu;
        return;
    }

    if key.modifiers.contains(KeyModifiers::CONTROL) {
        match key.code {
            KeyCode::Char('s') | KeyCode::Char('S') => {
                generate_text_widget(app);
                return;
            }
            KeyCode::Left => {
                app.text_widget.halign = prev_halign(app.text_widget.halign);
                return;
            }
            KeyCode::Right => {
                app.text_widget.halign = next_halign(app.text_widget.halign);
                return;
            }
            KeyCode::Up => {
                app.text_widget.valign = prev_valign(app.text_widget.valign);
                return;
            }
            KeyCode::Down => {
                app.text_widget.valign = next_valign(app.text_widget.valign);
                return;
            }
            _ => {}
        }
    }

    if key.code == KeyCode::Tab {
        app.text_widget.focus = app.text_widget.focus.next();
        return;
    }
    if key.code == KeyCode::BackTab {
        app.text_widget.focus = app.text_widget.focus.prev();
        return;
    }

    let field = match app.text_widget.focus {
        TextWidgetField::Filename => &mut app.text_widget.filename,
        TextWidgetField::Content => &mut app.text_widget.content,
        TextWidgetField::FontSize => &mut app.text_widget.font_size,
        TextWidgetField::LineHeight => &mut app.text_widget.line_height,
        TextWidgetField::Width => &mut app.text_widget.width,
        TextWidgetField::Height => &mut app.text_widget.height,
    };
    field.input(key);
}

fn sanitize_filename(raw: &str) -> Option<String> {
    let trimmed = raw.trim();
    if trimmed.is_empty() {
        return None;
    }
    let base = Path::new(trimmed).file_name()?.to_str()?.to_string();
    if base.is_empty() || base == "." || base == ".." {
        return None;
    }
    Some(base)
}

fn parse_or(ta: &TextArea, default: f32) -> f32 {
    field_text(ta).trim().parse().unwrap_or(default)
}

fn parse_or_u32(ta: &TextArea, default: u32) -> u32 {
    field_text(ta).trim().parse().unwrap_or(default)
}

fn generate_text_widget(app: &mut App) {
    generate_text_widget_in(app, Path::new("."));
}

fn generate_text_widget_in(app: &mut App, base: &Path) {
    let Some(name) = sanitize_filename(&field_text(&app.text_widget.filename)) else {
        app.status = Some("enter a valid filename".to_string());
        return;
    };

    let font_size = parse_or(&app.text_widget.font_size, 16.0).max(1.0);
    let line_height = parse_or(&app.text_widget.line_height, font_size * 1.25).max(1.0);
    let width = parse_or_u32(&app.text_widget.width, readme_stuff_draw::DEFAULT_WIDTH).max(1);
    let height = parse_or_u32(&app.text_widget.height, readme_stuff_draw::DEFAULT_HEIGHT).max(1);
    let align = Align {
        h: app.text_widget.halign,
        v: app.text_widget.valign,
    };

    let lines: Vec<&str> = app
        .text_widget
        .content
        .lines()
        .iter()
        .map(|s| s.as_str())
        .collect();
    let svg = readme_stuff_draw::render_text_widget(
        &lines,
        font_size,
        line_height,
        align,
        Theme::Dark,
        width,
        height,
    );

    let dir = base.join(readme_stuff_catalog::TEXT_WIDGETS_DIR);
    if let Err(e) = std::fs::create_dir_all(&dir) {
        app.status = Some(format!("cannot create {}: {e}", dir.display()));
        return;
    }
    let path = dir.join(format!("{name}_text.svg"));
    match std::fs::write(&path, svg) {
        Ok(()) => app.status = Some(format!("saved {}", path.display())),
        Err(e) => app.status = Some(format!("write failed: {e}")),
    }
}

pub fn rescan_discovered(app: &mut App) {
    rescan_discovered_in(app, Path::new("."));
}

fn rescan_discovered_in(app: &mut App, base: &Path) {
    let widgets_dir = base.join(readme_stuff_catalog::WIDGETS_DIR);
    let text_dir = base.join(readme_stuff_catalog::TEXT_WIDGETS_DIR);
    app.discovered = readme_stuff_catalog::scan_widgets(&[&widgets_dir, &text_dir]);
    let known: HashSet<String> = app.discovered.iter().map(|w| w.id.clone()).collect();
    app.discovered_selected.retain(|id| known.contains(id));
    if app.discovered_cursor >= app.discovered.len() {
        app.discovered_cursor = app.discovered.len().saturating_sub(1);
    }
}

pub fn toggle_discovered(app: &mut App, id: &str) {
    if !app.discovered_selected.remove(id) {
        app.discovered_selected.insert(id.to_string());
    }
}

fn move_discovered_cursor(app: &mut App, delta: i32) {
    let len = app.discovered.len();
    if len == 0 {
        return;
    }
    let next = (app.discovered_cursor as i32 + delta).clamp(0, len as i32 - 1);
    app.discovered_cursor = next as usize;
}

fn confirm_discovered_selection(app: &mut App) {
    if app.discovered_selected.is_empty() {
        app.status = Some("select at least one widget first".to_string());
    } else {
        init_editor(app);
        app.status = None;
        app.screen = Screen::ReadmeEditor;
    }
}

fn handle_widget_select_key(app: &mut App, key: KeyEvent) {
    match key.code {
        KeyCode::Esc => app.screen = Screen::MainMenu,
        KeyCode::Down | KeyCode::Char('j') => move_discovered_cursor(app, 1),
        KeyCode::Up | KeyCode::Char('k') => move_discovered_cursor(app, -1),
        KeyCode::Char(' ') | KeyCode::Enter => {
            if let Some(w) = app.discovered.get(app.discovered_cursor) {
                let id = w.id.clone();
                toggle_discovered(app, &id);
            }
        }
        KeyCode::Char('l') | KeyCode::Char('L') => confirm_discovered_selection(app),
        _ => {}
    }
}

fn handle_widget_select_mouse(app: &mut App, mouse: MouseEvent) {
    match mouse.kind {
        MouseEventKind::Down(MouseButton::Left) | MouseEventKind::Down(MouseButton::Right) => {
            let hit = app
                .discovered_rows
                .iter()
                .find(|(_, r)| r.contains(mouse.column, mouse.row))
                .map(|&(idx, _)| idx);
            if let Some(idx) = hit {
                app.discovered_cursor = idx;
                if let Some(w) = app.discovered.get(idx) {
                    let id = w.id.clone();
                    toggle_discovered(app, &id);
                }
            }
        }
        MouseEventKind::ScrollUp => move_discovered_cursor(app, -1),
        MouseEventKind::ScrollDown => move_discovered_cursor(app, 1),
        _ => {}
    }
}

pub fn init_editor(app: &mut App) {
    let mut editor = EditorState::new();
    for w in &app.discovered {
        if app.discovered_selected.contains(&w.id) {
            editor.sidebar.push(w.id.clone());
            editor.items.insert(w.id.clone(), w.clone());
        }
    }
    app.editor = editor;
}

fn canvas_scale(area: ScreenRect) -> (f64, f64) {
    let scale_x = area.w as f64 / CANVAS_WIDTH as f64;
    (scale_x, scale_x / 2.0)
}

pub fn cell_to_pixel(app: &App, col: u16, row: u16) -> Option<(i32, i32)> {
    let area = app.editor.canvas_area;
    if !area.contains(col, row) {
        return None;
    }
    let (scale_x, scale_y) = canvas_scale(area);
    if scale_x <= 0.0 || scale_y <= 0.0 {
        return None;
    }
    let px = (col - area.x) as f64 / scale_x;
    let py = (row - area.y) as f64 / scale_y + app.editor.scroll_y as f64;
    Some((px.round() as i32, py.round() as i32))
}

fn point_in(p: &PlacedItem, px: i32, py: i32) -> bool {
    px >= p.x && px < p.x + p.w as i32 && py >= p.y && py < p.y + p.h as i32
}

fn other_rects(app: &App) -> Vec<PixelRect> {
    app.editor
        .placed
        .iter()
        .map(|p| PixelRect {
            x: p.x,
            y: p.y,
            w: p.w,
            h: p.h,
        })
        .collect()
}

fn hit_sidebar(app: &App, col: u16, row: u16) -> Option<String> {
    app.editor
        .sidebar_items
        .iter()
        .find(|(_, r)| r.contains(col, row))
        .map(|(id, _)| id.clone())
}

fn snap_and_validate(app: &App, raw: PixelRect) -> ((i32, i32), bool) {
    let others = other_rects(app);
    let (sx, sy) = readme_stuff_catalog::snap(raw, &others, app.editor.canvas_width);
    let candidate = PixelRect {
        x: sx,
        y: sy,
        w: raw.w,
        h: raw.h,
    };
    let valid = readme_stuff_catalog::fits_canvas(candidate, app.editor.canvas_width)
        && !others
            .iter()
            .any(|o| readme_stuff_catalog::overlaps(candidate, *o));
    ((sx, sy), valid)
}

fn start_drag(app: &mut App, col: u16, row: u16) {
    if app.editor.drag.is_some() {
        return;
    }

    if let Some(id) = hit_sidebar(app, col, row) {
        let Some(item) = app.editor.items.get(&id) else {
            return;
        };
        let (w, h) = item.size;
        
        app.editor.drag = Some(DragState {
            id,
            from_sidebar: true,
            grab_dx: w as i32 / 2,
            grab_dy: h as i32 / 2,
            w,
            h,
            current: (0, 0),
            valid: false,
            moved: false,
            original: None,
        });
        return;
    }

    let Some((px, py)) = cell_to_pixel(app, col, row) else {
        return;
    };
    let Some(idx) = app.editor.placed.iter().position(|p| point_in(p, px, py)) else {
        return;
    };
    let placed = app.editor.placed.remove(idx);
    let grab_dx = px - placed.x;
    let grab_dy = py - placed.y;
    let original = (placed.x, placed.y);
    app.editor.drag = Some(DragState {
        id: placed.id,
        from_sidebar: false,
        grab_dx,
        grab_dy,
        w: placed.w,
        h: placed.h,
        current: original,
        valid: true,
        moved: false,
        original: Some(original),
    });
}

fn update_drag(app: &mut App, col: u16, row: u16) {
    let Some((px, py)) = cell_to_pixel(app, col, row) else {
        return;
    };
    let Some(drag) = &app.editor.drag else {
        return;
    };
    let (grab_dx, grab_dy, w, h) = (drag.grab_dx, drag.grab_dy, drag.w, drag.h);
    let raw = PixelRect {
        x: px - grab_dx,
        y: py - grab_dy,
        w,
        h,
    };
    let (current, valid) = snap_and_validate(app, raw);

    if let Some(drag) = app.editor.drag.as_mut() {
        drag.current = current;
        drag.valid = valid;
        drag.moved = true;
    }
}

fn end_drag(app: &mut App) {
    let Some(drag) = app.editor.drag.take() else {
        return;
    };
    if drag.valid && drag.moved {
        app.editor.placed.push(PlacedItem {
            id: drag.id.clone(),
            x: drag.current.0,
            y: drag.current.1,
            w: drag.w,
            h: drag.h,
        });
        app.editor.sidebar.retain(|id| id != &drag.id);
    } else if !drag.from_sidebar {
        if drag.moved {
            if let Some((ox, oy)) = drag.original {
                app.editor.placed.push(PlacedItem {
                    id: drag.id,
                    x: ox,
                    y: oy,
                    w: drag.w,
                    h: drag.h,
                });
            }
        } else {
            app.editor.sidebar.push(drag.id);
        }
    }
}

pub fn handle_mouse(app: &mut App, mouse: MouseEvent) {
    match app.screen {
        Screen::WidgetSelect => handle_widget_select_mouse(app, mouse),
        Screen::ReadmeEditor => handle_readme_editor_mouse(app, mouse),
        _ => {}
    }
}

fn handle_readme_editor_mouse(app: &mut App, mouse: MouseEvent) {
    match mouse.kind {
        MouseEventKind::Down(MouseButton::Left) | MouseEventKind::Down(MouseButton::Right) => {
            start_drag(app, mouse.column, mouse.row)
        }
        MouseEventKind::Drag(MouseButton::Left) | MouseEventKind::Drag(MouseButton::Right) => {
            update_drag(app, mouse.column, mouse.row)
        }
        MouseEventKind::Up(_) => end_drag(app),
        MouseEventKind::ScrollUp => scroll_editor(app, mouse.column, mouse.row, -1),
        MouseEventKind::ScrollDown => scroll_editor(app, mouse.column, mouse.row, 1),
        _ => {}
    }
}

fn scroll_editor(app: &mut App, col: u16, row: u16, dir: i32) {
    if app.editor.sidebar_area.contains(col, row) {
        app.editor.sidebar_scroll = (app.editor.sidebar_scroll as i32 + dir).max(0) as usize;
    } else if dir < 0 {
        app.editor.scroll_y = app.editor.scroll_y.saturating_sub(SCROLL_STEP);
    } else {
        app.editor.scroll_y += SCROLL_STEP;
    }
}

fn export_readme(app: &mut App) {
    export_readme_in(app, Path::new("."));
}

fn export_readme_in(app: &mut App, base: &Path) {
    if app.editor.placed.is_empty() {
        app.status = Some("place at least one widget before exporting".to_string());
        return;
    }

    let content_bottom = app
        .editor
        .placed
        .iter()
        .map(|p| (p.y + p.h as i32).max(0) as u32)
        .max()
        .unwrap_or(0);

    let mut svgs: Vec<String> = Vec::with_capacity(app.editor.placed.len());
    for item in &app.editor.placed {
        let Some(widget) = app.editor.items.get(&item.id) else {
            continue;
        };
        match std::fs::read_to_string(&widget.path) {
            Ok(content) => svgs.push(content),
            Err(e) => {
                app.status = Some(format!("cannot read {}: {e}", widget.path.display()));
                return;
            }
        }
    }

    let tiles: Vec<PlacedTile> = app
        .editor
        .placed
        .iter()
        .zip(svgs.iter())
        .map(|(item, svg)| PlacedTile {
            svg,
            x: item.x.max(0) as u32,
            y: item.y.max(0) as u32,
        })
        .collect();

    let svg = match readme_stuff_draw::compose_freeform(
        app.editor.canvas_width,
        content_bottom,
        EXPORT_BG,
        &tiles,
    ) {
        Ok(svg) => svg,
        Err(e) => {
            app.status = Some(format!("compose failed: {e}"));
            return;
        }
    };

    let dir = base.join(readme_stuff_catalog::WIDGETS_DIR);
    if let Err(e) = std::fs::create_dir_all(&dir) {
        app.status = Some(format!("cannot create {}: {e}", dir.display()));
        return;
    }
    let path = dir.join(README_EXPORT_NAME);
    match std::fs::write(&path, svg) {
        Ok(()) => app.status = Some(format!("exported {}", path.display())),
        Err(e) => app.status = Some(format!("write failed: {e}")),
    }
}

fn handle_readme_editor_key(app: &mut App, key: KeyEvent) {
    if key.code == KeyCode::Char('s') && key.modifiers.contains(KeyModifiers::CONTROL) {
        export_readme(app);
        return;
    }
    match key.code {
        KeyCode::Esc => app.screen = Screen::MainMenu,
        KeyCode::PageUp => {
            app.editor.scroll_y = app.editor.scroll_y.saturating_sub(SCROLL_STEP * 5);
        }
        KeyCode::PageDown => app.editor.scroll_y += SCROLL_STEP * 5,
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
    fn no_existing_config_starts_on_main_menu() {
        let app = App::new(None);
        assert_eq!(app.screen, Screen::MainMenu);
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
    fn resume_returns_to_main_menu_with_the_choice_of_generate_text_or_build() {
        let cfg = Config {
            profile: ProfileConfig {
                github_login: Some("octocat".to_string()),
                ..Default::default()
            },
            ..Default::default()
        };
        let mut app = App::new(Some((PathBuf::from("readme.toml"), cfg)));
        handle_key(&mut app, KeyEvent::from(KeyCode::Char('r')));
        assert_eq!(app.screen, Screen::MainMenu);
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
            ..Default::default()
        };
        let mut app = App::new(Some((PathBuf::from("readme.toml"), cfg)));
        handle_key(&mut app, KeyEvent::from(KeyCode::Char('r')));
        assert_eq!(app.screen, Screen::MainMenu);
        assert_eq!(field_text(&app.github_login), "octocat");
        assert!(app.selected.contains("github-stats"));

        handle_key(&mut app, KeyEvent::from(KeyCode::Char('g')));
        assert_eq!(app.screen, Screen::Questionnaire);
        assert_eq!(field_text(&app.github_login), "octocat");
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
        assert_eq!(app.screen, Screen::MainMenu);
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
    fn add_selected_is_idempotent_and_never_removes() {
        let mut app = app_with("octocat", "GITHUB_TOKEN");
        add_selected(&mut app, "github-stats");
        assert!(app.selected.contains("github-stats"));
        add_selected(&mut app, "github-stats");
        assert!(
            app.selected.contains("github-stats"),
            "second Add must not remove it"
        );
    }

    #[test]
    fn add_selected_is_a_no_op_when_not_selectable() {
        let mut app = app_with("", "");
        add_selected(&mut app, "cf-rating");
        assert!(!app.selected.contains("cf-rating"));
    }

    #[test]
    fn remove_selected_is_idempotent() {
        let mut app = app_with("octocat", "GITHUB_TOKEN");
        add_selected(&mut app, "github-stats");
        remove_selected(&mut app, "github-stats");
        assert!(!app.selected.contains("github-stats"));
        remove_selected(&mut app, "github-stats");
        assert!(
            !app.selected.contains("github-stats"),
            "second Delete must stay a no-op"
        );
    }

    #[test]
    fn pack_layout_is_empty_for_empty_selection() {
        let layout = pack_layout(&HashSet::new());
        assert!(layout.rows.is_empty());
        assert_eq!(layout.canvas_width, CANVAS_WIDTH);
    }

    #[test]
    fn pack_layout_never_exceeds_canvas_width_per_row() {
        let selected: HashSet<&'static str> =
            registry::all_widgets().iter().map(|w| w.id).collect();
        let layout = pack_layout(&selected);
        assert!(
            layout.rows.len() > 1,
            "expected wrapping across multiple rows"
        );
        for row in &layout.rows {
            let ids: HashSet<_> = row.widgets.iter().map(|w| w.id.as_str()).collect();
            assert_eq!(
                ids.len(),
                row.widgets.len(),
                "no duplicate placements within a row"
            );
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
    fn sync_layout_positions_seeds_new_selections_and_drops_deselected() {
        let mut app = app_with("octocat", "GITHUB_TOKEN");
        toggle_selected(&mut app, "github-stats");
        toggle_selected(&mut app, "github-repos");
        sync_layout_positions(&mut app);
        assert!(app.layout_positions.contains_key("github-stats"));
        assert!(app.layout_positions.contains_key("github-repos"));

        toggle_selected(&mut app, "github-repos");
        sync_layout_positions(&mut app);
        assert!(app.layout_positions.contains_key("github-stats"));
        assert!(!app.layout_positions.contains_key("github-repos"));
    }

    #[test]
    fn sync_layout_positions_keeps_a_manually_moved_widget_in_place() {
        let mut app = app_with("octocat", "GITHUB_TOKEN");
        toggle_selected(&mut app, "github-stats");
        sync_layout_positions(&mut app);
        move_layout_selected(&mut app, 15, 15);
        let moved = app.layout_positions["github-stats"];

        sync_layout_positions(&mut app);
        assert_eq!(app.layout_positions["github-stats"], moved);
    }

    #[test]
    fn move_layout_selected_clamps_to_the_canvas() {
        let mut app = app_with("octocat", "GITHUB_TOKEN");
        toggle_selected(&mut app, "github-stats");
        sync_layout_positions(&mut app);

        move_layout_selected(&mut app, -1000, -1000);
        let (x, y) = app.layout_positions["github-stats"];
        assert_eq!(x, 0);
        assert_eq!(y, 0);
    }

    #[test]
    fn move_layout_selected_is_blocked_by_an_overlap() {
        let mut app = app_with("octocat", "GITHUB_TOKEN");
        toggle_selected(&mut app, "github-stats");
        toggle_selected(&mut app, "github-repos");
        sync_layout_positions(&mut app);
        let before = app.layout_positions["github-stats"];

        let target = app.layout_positions["github-repos"];
        let dx = target.0 as i32 - before.0 as i32;
        move_layout_selected(&mut app, dx, 0);

        assert_eq!(app.layout_positions["github-stats"], before);
        assert!(app.status.is_some());
    }

    #[test]
    fn manual_layout_config_is_a_single_row_with_saved_positions() {
        let mut app = app_with("octocat", "GITHUB_TOKEN");
        toggle_selected(&mut app, "github-stats");
        sync_layout_positions(&mut app);
        move_layout_selected(&mut app, 15, 30);

        let cfg = to_config_with_manual_layout(&app);
        assert_eq!(cfg.layout.rows.len(), 1);
        let placed = &cfg.layout.rows[0].widgets[0];
        assert_eq!(placed.id, "github-stats");
        assert_eq!((placed.x, placed.y), app.layout_positions["github-stats"]);
    }

    #[test]
    fn save_and_queue_build_queues_a_build_and_moves_to_building() {
        let dir =
            std::env::temp_dir().join(format!("readme-stuff-tui-save-ok-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let mut app = app_with("octocat", "GITHUB_TOKEN");
        toggle_selected(&mut app, "github-stats");

        save_and_queue_build_in(&mut app, &dir);

        assert_eq!(app.screen, Screen::Building);
        assert!(app.pending_build.is_some());
        assert!(app.status.is_none());
        assert!(dir.join(io::CONFIG_FILE_NAME).exists());
        std::fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn save_and_queue_build_reports_an_error_and_does_not_queue_on_write_failure() {
        let dir = std::env::temp_dir()
            .join(format!(
                "readme-stuff-tui-save-missing-{}",
                std::process::id()
            ))
            .join("nested-that-does-not-exist");
        let mut app = app_with("octocat", "GITHUB_TOKEN");
        app.screen = Screen::Questionnaire;

        save_and_queue_build_in(&mut app, &dir);

        assert_eq!(app.screen, Screen::Questionnaire);
        assert!(app.pending_build.is_none());
        assert!(app.status.is_some());
    }

    #[test]
    fn apply_build_result_moves_to_report_with_the_payload() {
        let mut app = App::new(None);
        apply_build_result(&mut app, Err("boom".to_string()));
        assert_eq!(app.screen, Screen::Report);
        assert!(matches!(app.build_output, Some(Err(ref e)) if e == "boom"));
    }

    #[test]
    fn tick_building_increments_and_wraps() {
        let mut app = App::new(None);
        assert_eq!(app.build_tick, 0);
        tick_building(&mut app);
        assert_eq!(app.build_tick, 1);
        app.build_tick = u32::MAX;
        tick_building(&mut app);
        assert_eq!(app.build_tick, 0);
    }

    #[test]
    fn back_from_report_clears_stale_build_state() {
        let mut app = App::new(None);
        apply_build_result(&mut app, Err("boom".to_string()));
        app.pending_build = Some(to_config(&app));

        handle_key(&mut app, KeyEvent::from(KeyCode::Char('b')));

        assert_eq!(app.screen, Screen::Questionnaire);
        assert!(app.build_output.is_none());
        assert!(app.pending_build.is_none());
        assert_eq!(app.build_tick, 0);
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

    fn temp_dir(name: &str) -> PathBuf {
        let dir = std::env::temp_dir().join(format!(
            "readme-stuff-tui-app-{name}-{}",
            std::process::id()
        ));
        std::fs::create_dir_all(&dir).unwrap();
        dir
    }

    #[test]
    fn main_menu_g_enters_questionnaire() {
        let mut app = App::new(None);
        handle_key(&mut app, KeyEvent::from(KeyCode::Char('g')));
        assert_eq!(app.screen, Screen::Questionnaire);
    }

    #[test]
    fn main_menu_t_enters_text_widget_screen_with_a_fresh_form() {
        let mut app = App::new(None);
        app.text_widget.filename = single_line("stale");
        handle_key(&mut app, KeyEvent::from(KeyCode::Char('t')));
        assert_eq!(app.screen, Screen::TextWidgetGen);
        assert_eq!(field_text(&app.text_widget.filename), "");
    }

    #[test]
    fn questionnaire_esc_returns_to_main_menu() {
        let mut app = App::new(None);
        app.screen = Screen::Questionnaire;
        handle_key(&mut app, KeyEvent::from(KeyCode::Esc));
        assert_eq!(app.screen, Screen::MainMenu);
    }

    #[test]
    fn generate_text_widget_in_sanitizes_path_traversal_and_writes_the_file() {
        let dir = temp_dir("text-widget-ok");
        let mut app = App::new(None);
        app.text_widget.filename = single_line("../evil/name");
        app.text_widget.content = TextArea::from(vec!["hello".to_string()]);

        generate_text_widget_in(&mut app, &dir);

        let expected = dir
            .join(readme_stuff_catalog::TEXT_WIDGETS_DIR)
            .join("name_text.svg");
        assert!(expected.exists(), "status: {:?}", app.status);
        std::fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn generate_text_widget_in_rejects_an_empty_filename() {
        let dir = temp_dir("text-widget-empty");
        let mut app = App::new(None);

        generate_text_widget_in(&mut app, &dir);

        assert!(app.status.is_some());
        assert!(!dir.join(readme_stuff_catalog::TEXT_WIDGETS_DIR).exists());
        std::fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn toggle_discovered_adds_then_removes() {
        let mut app = App::new(None);
        toggle_discovered(&mut app, "readme_test/a.svg");
        assert!(app.discovered_selected.contains("readme_test/a.svg"));
        toggle_discovered(&mut app, "readme_test/a.svg");
        assert!(!app.discovered_selected.contains("readme_test/a.svg"));
    }

    #[test]
    fn rescan_discovered_in_finds_generated_widgets() {
        let dir = temp_dir("rescan");
        let widgets_dir = dir.join(readme_stuff_catalog::WIDGETS_DIR);
        std::fs::create_dir_all(&widgets_dir).unwrap();
        std::fs::write(
            widgets_dir.join("a.svg"),
            r#"<svg width="10" height="10" xmlns="http://www.w3.org/2000/svg"></svg>"#,
        )
        .unwrap();

        let mut app = App::new(None);
        rescan_discovered_in(&mut app, &dir);

        assert_eq!(app.discovered.len(), 1);
        std::fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn init_editor_seeds_sidebar_from_the_selection_only() {
        let mut app = App::new(None);
        app.discovered = vec![
            DiscoveredWidget {
                id: "a".to_string(),
                path: PathBuf::from("a"),
                size: (10, 10),
            },
            DiscoveredWidget {
                id: "b".to_string(),
                path: PathBuf::from("b"),
                size: (20, 20),
            },
        ];
        app.discovered_selected.insert("b".to_string());

        init_editor(&mut app);

        assert_eq!(app.editor.sidebar, vec!["b".to_string()]);
        assert!(app.editor.items.contains_key("b"));
        assert!(!app.editor.items.contains_key("a"));
        assert!(app.editor.placed.is_empty());
    }

    fn discovered_app(ids: &[&str]) -> App {
        let mut app = App::new(None);
        app.screen = Screen::WidgetSelect;
        app.discovered = ids
            .iter()
            .map(|id| DiscoveredWidget {
                id: id.to_string(),
                path: PathBuf::from(id),
                size: (10, 10),
            })
            .collect();
        app
    }

    #[test]
    fn clicking_a_row_toggles_that_widget_regardless_of_cursor_position() {
        let mut app = discovered_app(&["a", "b", "c"]);
        app.discovered_cursor = 0;
        app.discovered_rows = vec![
            (0, ScreenRect { x: 0, y: 0, w: 20, h: 1 }),
            (1, ScreenRect { x: 0, y: 1, w: 20, h: 1 }),
            (2, ScreenRect { x: 0, y: 2, w: 20, h: 1 }),
        ];

        handle_mouse(
            &mut app,
            MouseEvent {
                kind: MouseEventKind::Down(MouseButton::Left),
                column: 5,
                row: 1,
                modifiers: KeyModifiers::empty(),
            },
        );

        assert_eq!(app.discovered_cursor, 1);
        assert!(app.discovered_selected.contains("b"));
        assert!(!app.discovered_selected.contains("a"));
    }

    #[test]
    fn wheel_moves_the_cursor_on_the_widget_select_screen() {
        let mut app = discovered_app(&["a", "b", "c"]);
        app.discovered_cursor = 0;

        handle_mouse(
            &mut app,
            MouseEvent {
                kind: MouseEventKind::ScrollDown,
                column: 0,
                row: 0,
                modifiers: KeyModifiers::empty(),
            },
        );
        assert_eq!(app.discovered_cursor, 1);

        handle_mouse(
            &mut app,
            MouseEvent {
                kind: MouseEventKind::ScrollUp,
                column: 0,
                row: 0,
                modifiers: KeyModifiers::empty(),
            },
        );
        assert_eq!(app.discovered_cursor, 0);
    }

    fn editor_test_app(widgets: &[(&str, u32, u32)]) -> App {
        let mut app = App::new(None);
        app.editor.canvas_area = ScreenRect {
            x: 0,
            y: 0,
            w: CANVAS_WIDTH as u16,
            h: 2000,
        };
        for (id, w, h) in widgets {
            app.editor.items.insert(
                id.to_string(),
                DiscoveredWidget {
                    id: id.to_string(),
                    path: PathBuf::from(id),
                    size: (*w, *h),
                },
            );
            app.editor.sidebar.push(id.to_string());
        }
        app
    }

    #[test]
    fn drag_from_sidebar_snaps_to_the_canvas_corner_and_commits_on_release() {
        let mut app = editor_test_app(&[("w1", 100, 50)]);
        app.editor.sidebar_items = vec![(
            "w1".to_string(),
            ScreenRect {
                x: 0,
                y: 0,
                w: 5,
                h: 2,
            },
        )];

        start_drag(&mut app, 2, 1);
        assert!(app.editor.drag.is_some());

        update_drag(&mut app, 52, 13);
        end_drag(&mut app);

        assert!(app.editor.drag.is_none());
        assert!(!app.editor.sidebar.contains(&"w1".to_string()));
        let placed = app
            .editor
            .placed
            .iter()
            .find(|p| p.id == "w1")
            .expect("w1 should have been committed");
        assert_eq!((placed.x, placed.y), (0, 0));
    }

    #[test]
    fn drag_starts_from_the_sidebar_even_though_sidebar_and_canvas_are_disjoint_columns() {
        let mut app = editor_test_app(&[("w1", 100, 50)]);
        app.editor.canvas_area = ScreenRect {
            x: 50,
            y: 0,
            w: CANVAS_WIDTH as u16,
            h: 2000,
        };
        app.editor.sidebar_items = vec![(
            "w1".to_string(),
            ScreenRect {
                x: 0,
                y: 0,
                w: 5,
                h: 2,
            },
        )];

        start_drag(&mut app, 2, 1);
        assert!(
            app.editor.drag.is_some(),
            "a sidebar click must start a drag even when it falls outside canvas_area"
        );

        update_drag(&mut app, 250, 200);
        end_drag(&mut app);

        assert!(
            app.editor.placed.iter().any(|p| p.id == "w1"),
            "the widget should have been placed once the drag reached the canvas"
        );
    }

    #[test]
    fn repositioning_a_placed_widget_reverts_to_its_original_spot_on_collision() {
        let mut app = editor_test_app(&[]);
        app.editor.sidebar.clear();
        app.editor.placed.push(PlacedItem {
            id: "w1".to_string(),
            x: 0,
            y: 0,
            w: 100,
            h: 50,
        });
        app.editor.placed.push(PlacedItem {
            id: "w2".to_string(),
            x: 200,
            y: 0,
            w: 100,
            h: 50,
        });

        start_drag(&mut app, 50, 13);
        assert!(app.editor.drag.is_some());

        update_drag(&mut app, 250, 13);
        assert!(!app.editor.drag.as_ref().unwrap().valid);
        end_drag(&mut app);

        let w1 = app
            .editor
            .placed
            .iter()
            .find(|p| p.id == "w1")
            .expect("w1 must still be placed after a rejected move");
        assert_eq!((w1.x, w1.y), (0, 0));
        assert!(!app.editor.sidebar.contains(&"w1".to_string()));
    }

    #[test]
    fn clicking_a_placed_widget_without_dragging_returns_it_to_the_sidebar() {
        let mut app = editor_test_app(&[]);
        app.editor.placed.push(PlacedItem {
            id: "w1".to_string(),
            x: 10,
            y: 10,
            w: 100,
            h: 50,
        });

        start_drag(&mut app, 50, 10);
        end_drag(&mut app);

        assert!(app.editor.placed.is_empty());
        assert!(app.editor.sidebar.contains(&"w1".to_string()));
    }

    #[test]
    fn wheel_scrolls_the_sidebar_or_the_canvas_depending_on_where_the_pointer_is() {
        let mut app = editor_test_app(&[]);
        app.screen = Screen::ReadmeEditor;
        app.editor.sidebar_area = ScreenRect {
            x: 0,
            y: 0,
            w: 20,
            h: 30,
        };
        app.editor.canvas_area = ScreenRect {
            x: 20,
            y: 0,
            w: CANVAS_WIDTH as u16,
            h: 2000,
        };

        handle_mouse(
            &mut app,
            MouseEvent {
                kind: MouseEventKind::ScrollDown,
                column: 5,
                row: 5,
                modifiers: KeyModifiers::empty(),
            },
        );
        assert_eq!(app.editor.sidebar_scroll, 1);
        assert_eq!(app.editor.scroll_y, 0);

        handle_mouse(
            &mut app,
            MouseEvent {
                kind: MouseEventKind::ScrollDown,
                column: 50,
                row: 5,
                modifiers: KeyModifiers::empty(),
            },
        );
        assert_eq!(app.editor.sidebar_scroll, 1);
        assert_eq!(app.editor.scroll_y, SCROLL_STEP);
    }

    #[test]
    fn a_right_button_drag_still_commits_when_the_release_is_reported_as_the_wrong_button() {
        
        let mut app = editor_test_app(&[("w1", 100, 50)]);
        app.screen = Screen::ReadmeEditor;
        app.editor.sidebar_items = vec![(
            "w1".to_string(),
            ScreenRect {
                x: 0,
                y: 0,
                w: 5,
                h: 2,
            },
        )];

        handle_mouse(
            &mut app,
            MouseEvent {
                kind: MouseEventKind::Down(MouseButton::Right),
                column: 2,
                row: 1,
                modifiers: KeyModifiers::empty(),
            },
        );
        assert!(app.editor.drag.is_some());

        // move onto the canvas, away from any edge, so the drop position is valid
        handle_mouse(
            &mut app,
            MouseEvent {
                kind: MouseEventKind::Drag(MouseButton::Right),
                column: 200,
                row: 200,
                modifiers: KeyModifiers::empty(),
            },
        );
        assert!(app.editor.drag.as_ref().unwrap().valid);

        handle_mouse(
            &mut app,
            MouseEvent {
                kind: MouseEventKind::Up(MouseButton::Left),
                column: 200,
                row: 200,
                modifiers: KeyModifiers::empty(),
            },
        );

        assert!(app.editor.drag.is_none());
        assert!(app.editor.placed.iter().any(|p| p.id == "w1"));
    }

    #[test]
    fn left_button_drag_from_sidebar_places_a_widget_on_the_canvas() {
        let mut app = editor_test_app(&[("w1", 100, 50)]);
        app.screen = Screen::ReadmeEditor;
        app.editor.sidebar_items = vec![(
            "w1".to_string(),
            ScreenRect {
                x: 0,
                y: 0,
                w: 5,
                h: 2,
            },
        )];

        handle_mouse(
            &mut app,
            MouseEvent {
                kind: MouseEventKind::Down(MouseButton::Left),
                column: 2,
                row: 1,
                modifiers: KeyModifiers::empty(),
            },
        );
        assert!(app.editor.drag.is_some());

        handle_mouse(
            &mut app,
            MouseEvent {
                kind: MouseEventKind::Drag(MouseButton::Left),
                column: 200,
                row: 200,
                modifiers: KeyModifiers::empty(),
            },
        );
        assert!(app.editor.drag.as_ref().unwrap().valid);

        handle_mouse(
            &mut app,
            MouseEvent {
                kind: MouseEventKind::Up(MouseButton::Left),
                column: 200,
                row: 200,
                modifiers: KeyModifiers::empty(),
            },
        );

        assert!(app.editor.drag.is_none());
        assert!(app.editor.placed.iter().any(|p| p.id == "w1"));
    }

    #[test]
    fn left_button_click_without_dragging_on_a_placed_widget_removes_it() {
        let mut app = editor_test_app(&[]);
        app.screen = Screen::ReadmeEditor;
        app.editor.placed.push(PlacedItem {
            id: "w1".to_string(),
            x: 10,
            y: 10,
            w: 100,
            h: 50,
        });

        handle_mouse(
            &mut app,
            MouseEvent {
                kind: MouseEventKind::Down(MouseButton::Left),
                column: 50,
                row: 10,
                modifiers: KeyModifiers::empty(),
            },
        );
        handle_mouse(
            &mut app,
            MouseEvent {
                kind: MouseEventKind::Up(MouseButton::Left),
                column: 50,
                row: 10,
                modifiers: KeyModifiers::empty(),
            },
        );

        assert!(app.editor.placed.is_empty());
        assert!(app.editor.sidebar.contains(&"w1".to_string()));
    }

    #[test]
    fn export_readme_in_requires_at_least_one_placed_widget() {
        let dir = temp_dir("export-empty");
        let mut app = App::new(None);

        export_readme_in(&mut app, &dir);

        assert!(app.status.is_some());
        assert!(
            !dir.join(readme_stuff_catalog::WIDGETS_DIR)
                .join(README_EXPORT_NAME)
                .exists()
        );
        std::fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn export_readme_in_crops_to_the_lowest_placed_widget() {
        let dir = temp_dir("export-crop");
        let widget_path = dir.join("w1.svg");
        std::fs::write(
            &widget_path,
            r#"<svg width="100" height="50" xmlns="http://www.w3.org/2000/svg"><rect width="100" height="50"/></svg>"#,
        )
        .unwrap();

        let mut app = App::new(None);
        app.editor.items.insert(
            "w1".to_string(),
            DiscoveredWidget {
                id: "w1".to_string(),
                path: widget_path,
                size: (100, 50),
            },
        );
        app.editor.placed.push(PlacedItem {
            id: "w1".to_string(),
            x: 0,
            y: 0,
            w: 100,
            h: 50,
        });

        export_readme_in(&mut app, &dir);

        let out = dir
            .join(readme_stuff_catalog::WIDGETS_DIR)
            .join(README_EXPORT_NAME);
        let content = std::fs::read_to_string(&out).unwrap_or_else(|e| {
            panic!(
                "expected {} to exist (status: {:?}): {e}",
                out.display(),
                app.status
            )
        });
        assert!(content.contains(r#"height="50""#));
        assert!(content.contains(r#"translate(0,0)"#));
        std::fs::remove_dir_all(&dir).ok();
    }
}
