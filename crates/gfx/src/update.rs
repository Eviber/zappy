use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use crate::app::App;
use crate::app::state::{MapState, PopupCommand, State};
use crate::game_logic::Map;

pub fn update(app: &mut App, key_event: KeyEvent) {
    match key_event.code {
        KeyCode::Char('q') => app.quit(),
        KeyCode::Char('c') | KeyCode::Char('C') => {
            if key_event.modifiers == KeyModifiers::CONTROL {
                app.quit()
            }
        },
        KeyCode::Tab => {
            // Change the state
            app.state = match app.state {
                State::Map { .. } => {
                    State::Admin
                },
                State::Admin => {
                    State::Options
                },
                State::Options => {
                    // Temporary
                    State::Map { map: Map::new(10, 10), state: MapState::default(), vertical_scroll: 0 }
                },
            }
        },
        _ => {}
    }
    if let State::Map { state, .. } = &mut app.state {
        update_map(state, key_event);
    }
}

fn update_map(map_state: &mut MapState, key_event: KeyEvent) {
    match map_state {
        MapState::Selecting((pos_x, pos_y)) => {
            match key_event.code {
                KeyCode::Enter => {
                    *map_state = MapState::Selected {
                        selected_cell: (*pos_x, *pos_y),
                        selected_command: 0,
                        command: PopupCommand::Command1,
                    };
                },
                KeyCode::Right => {
                    // Move the cursor to the right
                    *pos_x = pos_x.checked_add(1).unwrap_or(*pos_x);
                },
                KeyCode::Left => {
                    // Move the cursor to the left
                    *pos_x = pos_x.checked_sub(1).unwrap_or(*pos_x);
                },
                KeyCode::Up => {
                    // Move the cursor up
                    *pos_y = pos_y.checked_sub(1).unwrap_or(*pos_y);
                },
                KeyCode::Down => {
                    // Move the cursor down
                    *pos_y = pos_y.checked_add(1).unwrap_or(*pos_y);
                },
                _ => {}
            }
        },
        MapState::Selected { selected_command, command, selected_cell } => {
            match key_event.code {
                KeyCode::Up => {
                    if *selected_command == 0 {
                        *selected_command = 2;
                    } else {
                        *selected_command -= 1;
                    }
                },
                KeyCode::Down => {
                    *selected_command = (*selected_command + 1) % 3;
                },
                KeyCode::Enter => {
                    *command = PopupCommand::from(*selected_command);
                    match command {
                        PopupCommand::Command1 => {
                            // Do something
                        },
                        PopupCommand::Command2 => {
                            // Do something
                        },
                        PopupCommand::Command3 => {
                            // Do something
                        },
                        _ => {}
                    }
                },
                KeyCode::Esc => {
                    *map_state = MapState::Selecting(*selected_cell);
                },
                _ => {}
            }
        }
    }
}