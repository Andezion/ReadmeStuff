mod app;
mod ui;

use app::{App, Screen};
use crossterm::event::{DisableMouseCapture, EnableMouseCapture, Event, KeyEventKind};
use crossterm::execute;
use ratatui::DefaultTerminal;
use readme_stuff_catalog::BuildOutput;
use std::path::PathBuf;
use std::sync::mpsc;
use std::time::Duration;

fn output_dir() -> PathBuf {
    PathBuf::from(
        std::env::var("OUTPUT_DIR").unwrap_or_else(|_| readme_stuff_catalog::WIDGETS_DIR.into()),
    )
}

fn main() -> std::io::Result<()> {
    readme_stuff_config::io::load_dotenv();

    let existing = readme_stuff_config::io::find_config().and_then(|path| {
        readme_stuff_config::io::load(&path)
            .ok()
            .map(|cfg| (path, cfg))
    });
    let mut state = App::new(existing);

    let mut terminal = ratatui::init();
    execute!(std::io::stdout(), EnableMouseCapture)?;
    let result = run(&mut terminal, &mut state);
    let _ = execute!(std::io::stdout(), DisableMouseCapture);
    ratatui::restore();
    result
}

fn run(terminal: &mut DefaultTerminal, app: &mut App) -> std::io::Result<()> {
    let (tx, rx) = mpsc::channel::<Result<BuildOutput, String>>();

    while !app.should_quit {
        terminal.draw(|frame| ui::draw(frame, app))?;

        if app.screen == Screen::Building {
            app::tick_building(app);
        }

        if let Some(cfg) = app.pending_build.take() {
            let tx = tx.clone();
            let out_dir = output_dir();
            std::thread::spawn(move || {
                let result = match tokio::runtime::Runtime::new() {
                    Ok(rt) => rt.block_on(readme_stuff_catalog::pipeline::build(&cfg, &out_dir)),
                    Err(e) => Err(format!("could not start build runtime: {e}")),
                };
                let _ = tx.send(result);
            });
        }

        if let Ok(result) = rx.try_recv() {
            app::apply_build_result(app, result);
        }

        if crossterm::event::poll(Duration::from_millis(100))? {
            match crossterm::event::read()? {
                Event::Key(key) if key.kind == KeyEventKind::Press => {
                    app::handle_key(app, key);
                }
                Event::Mouse(mouse) => app::handle_mouse(app, mouse),
                _ => {}
            }
        }
    }
    Ok(())
}
