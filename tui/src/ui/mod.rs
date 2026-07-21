pub mod theme;

mod done;
mod questionnaire;
mod welcome;

use crate::app::{App, Screen};
use ratatui::Frame;

pub fn draw(frame: &mut Frame, app: &App) {
    match app.screen {
        Screen::Welcome => welcome::draw(frame, app),
        Screen::Questionnaire => questionnaire::draw(frame, app),
        Screen::Done => done::draw(frame, app),
    }
}
