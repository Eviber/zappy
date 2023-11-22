/// Application.
#[derive(Debug, Default)]
pub struct App {
    /// should the application exit?
    pub should_quit: bool,
    pub active_tab: usize,
    pub grid: Vec<Vec<char>>,
    pub selected_position: (usize, usize),
    pub popup: Option<PopupState>,
}

#[derive(Debug, Default)]
pub struct PopupState {
    pub selected: usize,
    pub command: PopupCommand,
}

#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub enum PopupCommand {
    #[default]
    None,
    Command1,
    Command2,
    Command3,
}

impl From<usize> for PopupCommand {
    fn from(index: usize) -> Self {
        match index {
            0 => PopupCommand::Command1,
            1 => PopupCommand::Command2,
            2 => PopupCommand::Command3,
            _ => PopupCommand::None,
        }
    }
}

impl From<PopupCommand> for PopupState {
    fn from(command: PopupCommand) -> Self {
        PopupState {
            selected: 0,
            command,
        }
    }
}

impl App {

    pub fn new() -> App {
        let grid_size = 10; // Example size
        let grid = vec![vec![' '; grid_size]; grid_size];

        App {
            grid,
            ..Default::default()
        }
    }    /// Constructs a new instance of [`App`].

    /// Handles the tick event of the terminal.
    pub fn tick(&self) {}

    /// Set running to false to quit the application.
    pub fn quit(&mut self) {
        self.should_quit = true;
    }

}