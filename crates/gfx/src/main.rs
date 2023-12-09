/// Application.
pub mod app;

/// Terminal events handler.
pub mod event;

/// Widget renderer.
pub mod ui;

/// Terminal user interface.
pub mod tui;

/// Application updater.
pub mod update;

use anyhow::Result;
use app::App;
use event::{Event, EventHandler};
use ratatui::{backend::CrosstermBackend, Terminal};
use tui::Tui;
use update::update;

fn main() -> Result<()> {
    // Create an application.
    let mut app = App::new();

    // Initialize the terminal user interface.
    let backend = CrosstermBackend::new(std::io::stderr());
    let terminal = Terminal::new(backend)?;
    let events = EventHandler::new(250);
    let mut tui = Tui::new(terminal, events);
    tui.enter()?;

    let logs = vec![
        "This is a log message".to_string(),
        "This is another log message".to_string(),
        "This is a third log message".to_string(),
        "This is a fourth log message".to_string(),
        "This is a fifth log message".to_string(),
        "This is a sixth log message".to_string(),
        "This is a seventh log message".to_string(),
        "This is a eighth log message".to_string(),
        "This is a ninth log message".to_string(),
        "This is a tenth log message".to_string(),
        "This is a eleventh log message".to_string(),
        "This is a twelfth log message".to_string(),
    ];

    app.logs = logs;
    // Start the main loop.
    while !app.should_quit {
        // Render the user interface.
        tui.draw(&mut app)?;
        // Handle events.
        match tui.events.next()? {
            Event::Tick => {},
            Event::Key(key_event) => update(&mut app, key_event),
            Event::Mouse(_) => {},
            Event::Resize(_, _) => {},
        };
    }

    // Exit the user interface.
    tui.exit()?;
    Ok(())
}