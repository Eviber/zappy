use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use crate::app::{App, PopupCommand};

pub fn update(app: &mut App, key_event: KeyEvent) {
    match key_event.code {
        KeyCode::Esc => {
            app.popup = None;
        },
        KeyCode::Char('q') => app.quit(),
        KeyCode::Char('c') | KeyCode::Char('C') => {
            if key_event.modifiers == KeyModifiers::CONTROL {
                app.quit()
            }
        },
        KeyCode::Tab => {
            app.active_tab = (app.active_tab + 1) % 3; // Assuming 3 tabs
        },
        KeyCode::Right => {
            // Move the cursor to the right
            if app.popup.is_none() {
                app.selected_position.1 = (app.selected_position.1 + 1) % app.grid[0].len();
            }
        },
        KeyCode::Left => {
            // Move the cursor to the left
            if app.popup.is_none() {
                app.selected_position.1 = if app.selected_position.1 > 0 {
                    app.selected_position.1 - 1
                } else {
                    app.grid[0].len() - 1
                };
            }
        },
        KeyCode::Up => {
            // Move the cursor up
            if let Some(ref mut popup) = &mut app.popup {
                popup.selected = if popup.selected > 0 {
                    popup.selected - 1
                } else {
                    3
                };
            }
            else {
                app.selected_position.0 = if app.selected_position.0 > 0 {
                    app.selected_position.0 - 1
                } else {
                    app.grid.len() - 1
                };
            }
        },
        KeyCode::Down => {
            // Move the cursor down
            if let Some(ref mut popup) = &mut app.popup {
                popup.selected = (popup.selected + 1) % 4;
            }
            else {
                app.selected_position.0 = (app.selected_position.0 + 1) % app.grid.len();
            }
        },
        KeyCode::Enter => {
            if let Some(popup) = &app.popup {
                match popup.command {
                    PopupCommand::Command1 => {
                        app.logs.push("Command 1 executed".to_string());
                        app.popup = None;
                    },
                    PopupCommand::Command2 => {
                        // Do something
                        app.popup = None;
                    },
                    PopupCommand::Command3 => {
                        // Do something
                        app.popup = None;
                    },
                    _ => {}
                }
            }
            else {
                app.popup = Some(PopupCommand::None.into());
            }
        }
        KeyCode::Char('+') => {
            // Add a new row
            if key_event.modifiers == KeyModifiers::SHIFT {
                app.grid.push(vec![' '; app.grid[0].len()]);
            }
            else {
                // Add a new column
                for row in &mut app.grid {
                    row.push(' ');
                }
            }
        }
        KeyCode::Char('-') => {
            // Remove a row
            if key_event.modifiers == KeyModifiers::SHIFT {
                app.grid.pop();
            }
            else {
                // Remove a column
                for row in &mut app.grid {
                    row.pop();
                }
            }
        }

        _ => {}
    }
}
