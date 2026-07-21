mod app;
mod ui;

use app::App;
use crossterm::event::{Event, KeyEventKind};
use ratatui::DefaultTerminal;

fn main() -> std::io::Result<()> {
    let existing = readme_stuff_config::io::find_config()
        .and_then(|path| readme_stuff_config::io::load(&path).ok().map(|cfg| (path, cfg)));
    let mut state = App::new(existing);

    let mut terminal = ratatui::init();
    let result = run(&mut terminal, &mut state);
    ratatui::restore();
    result
}

fn run(terminal: &mut DefaultTerminal, state: &mut App) -> std::io::Result<()> {
    while !state.should_quit {
        terminal.draw(|frame| ui::draw(frame, state))?;
        if let Event::Key(key) = crossterm::event::read()? {
            if key.kind == KeyEventKind::Press {
                app::handle_key(state, key);
            }
        }
    }
    Ok(())
}
