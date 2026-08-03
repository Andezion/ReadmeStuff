pub mod theme;

mod building;
mod layout;
mod main_menu;
mod questionnaire;
mod readme_editor;
mod report;
mod text_widget;
mod welcome;
mod widget_select;

use crate::app::{App, Screen};
use ratatui::Frame;

pub fn draw(frame: &mut Frame, app: &mut App) {
    match app.screen {
        Screen::MainMenu => main_menu::draw(frame, app),
        Screen::Welcome => welcome::draw(frame, app),
        Screen::Questionnaire => questionnaire::draw(frame, app),
        Screen::Layout => layout::draw(frame, app),
        Screen::Building => building::draw(frame, app),
        Screen::Report => report::draw(frame, app),
        Screen::TextWidgetGen => text_widget::draw(frame, app),
        Screen::WidgetSelect => widget_select::draw(frame, app),
        Screen::ReadmeEditor => readme_editor::draw(frame, app),
    }
}
