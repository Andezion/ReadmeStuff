pub mod theme;

mod building;
mod questionnaire;
mod report;
mod welcome;

use crate::app::{App, Screen};
use ratatui::Frame;

pub fn draw(frame: &mut Frame, app: &App) {
    match app.screen {
        Screen::Welcome => welcome::draw(frame, app),
        Screen::Questionnaire => questionnaire::draw(frame, app),
        Screen::Building => building::draw(frame, app),
        Screen::Report => report::draw(frame, app),
    }
}
