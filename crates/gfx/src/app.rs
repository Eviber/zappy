use ratatui::widgets::TableState;
use state::State;

pub mod state;

/// Application.
#[derive(Debug, Default)]
pub struct App {
    pub should_quit: bool,
    pub logs: Vec<String>,
    pub state: State,
    pub table_state: TableState,
}

impl App {
    /// Constructs a new instance of [`App`].
    pub fn new() -> App {
        App {
            should_quit: false,
            logs: Vec::new(),
            state: State::default(),
            table_state: Default::default(),
        }
    }

    /// Handles the tick event of the terminal.
    pub fn tick(&self) {}

    /// Set running to false to quit the application.
    pub fn quit(&mut self) {
        self.should_quit = true;
    }
}
