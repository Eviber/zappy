use crate::app::state::{MapState, PlayerAction, PopupState, ResourceType, State};
use crate::app::App;
use crate::game_logic::{CellContent, Map};
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use ratatui::widgets::TableState;

pub fn update(app: &mut App, key_event: KeyEvent) {
    match &app.state {
        State::Map { state, .. } => match state {
            MapState::Selecting(_) => handle_map_navigation(app, key_event),
            MapState::Selected { .. } => handle_popup_navigation(app, key_event),
        },
        State::Admin => handle_admin(app, key_event),
        State::Options => handle_options(app, key_event),
    }
}

fn handle_map_navigation(app: &mut App, key_event: KeyEvent) {
    if key_event.modifiers == KeyModifiers::CONTROL && key_event.code == KeyCode::Char('c')
        || key_event.code == KeyCode::Char('q')
    {
        app.quit();
        return;
    }

    let map = match &app.state {
        State::Map { map, .. } => map,
        _ => return,
    };

    match key_event.code {
        KeyCode::Left => {
            if let Some(col) = app.table_state.selected_column() {
                if col > 0 {
                    app.table_state.select_column(Some(col - 1));
                    update_selected_cell(app);
                }
            } else {
                app.table_state.select_column(Some(0));
                if app.table_state.selected().is_none() {
                    app.table_state.select(Some(0));
                }
            }
        }
        KeyCode::Right => {
            if let Some(col) = app.table_state.selected_column() {
                if col < map.x_max - 1 {
                    app.table_state.select_column(Some(col + 1));
                    update_selected_cell(app);
                }
            } else {
                app.table_state.select_column(Some(0));
                if app.table_state.selected().is_none() {
                    app.table_state.select(Some(0));
                }
            }
        }
        KeyCode::Up => {
            if let Some(row) = app.table_state.selected() {
                if row > 0 {
                    app.table_state.select(Some(row - 1));
                    update_selected_cell(app);
                }
            } else {
                app.table_state.select(Some(0));
                if app.table_state.selected_column().is_none() {
                    app.table_state.select_column(Some(0));
                }
            }
        }
        KeyCode::Down => {
            if let Some(row) = app.table_state.selected() {
                if row < map.y_max - 1 {
                    app.table_state.select(Some(row + 1));
                    update_selected_cell(app);
                }
            } else {
                app.table_state.select(Some(0));
                if app.table_state.selected_column().is_none() {
                    app.table_state.select_column(Some(0));
                }
            }
        }
        KeyCode::Enter => {
            if let (Some(row), Some(col)) = (
                app.table_state.selected(),
                app.table_state.selected_column(),
            ) {
                app.state = State::Map {
                    map: match std::mem::take(&mut app.state) {
                        State::Map { map, .. } => map,
                        _ => unreachable!(),
                    },
                    state: MapState::Selected {
                        selected_cell: (row, col),
                        popup_state: PopupState::MainMenu { selected_item: 0 },
                    },
                    vertical_scroll: 0,
                };
            }
        }
        KeyCode::Tab => {
            app.state = State::Admin;
            app.table_state = TableState::default();
        }
        _ => {}
    }
}

fn update_selected_cell(app: &mut App) {
    if let (Some(row), Some(col)) = (
        app.table_state.selected(),
        app.table_state.selected_column(),
    ) {
        if let State::Map { state, .. } = &mut app.state {
            *state = MapState::Selecting((row, col));
        }
    }
}

fn handle_popup_navigation(app: &mut App, key_event: KeyEvent) {
    if let State::Map {
        state: MapState::Selected {
            selected_cell,
            popup_state,
        },
        map,
        ..
    } = &app.state
    {
        let new_state = match popup_state {
            PopupState::MainMenu { selected_item } => {
                handle_main_menu(key_event, *selected_item, *selected_cell, map)
            }
            PopupState::ResourceMenu {
                resource_type,
                current_amount,
            } => handle_resource_menu(key_event, *resource_type, *current_amount, *selected_cell),
            PopupState::PlayerMenu {
                player_id,
                selected_action,
            } => handle_player_menu(key_event, *player_id, *selected_cell, selected_action),
        };

        if let Some(new_state) = new_state {
            if let State::Map { state, .. } = &mut app.state {
                *state = new_state;
            }
        }
    }
}

fn handle_main_menu(
    key_event: KeyEvent,
    selected_item: usize,
    selected_cell: (usize, usize),
    map: &Map,
) -> Option<MapState> {
    let cell = &map.cells[selected_cell.0 * map.x_max + selected_cell.1];
    let item_count = cell.content.len();

    match key_event.code {
        KeyCode::Esc => Some(MapState::Selecting(selected_cell)),
        KeyCode::Up if selected_item > 0 => Some(MapState::Selected {
            selected_cell,
            popup_state: PopupState::MainMenu {
                selected_item: selected_item - 1,
            },
        }),
        KeyCode::Down if selected_item < item_count - 1 => Some(MapState::Selected {
            selected_cell,
            popup_state: PopupState::MainMenu {
                selected_item: selected_item + 1,
            },
        }),
        KeyCode::Enter => {
            if let Some(content) = cell.content.get(selected_item) {
                Some(MapState::Selected {
                    selected_cell,
                    popup_state: match content {
                        CellContent::Food => PopupState::ResourceMenu {
                            resource_type: ResourceType::Food,
                            current_amount: 1,
                        },
                        CellContent::Rocks(rock) => PopupState::ResourceMenu {
                            resource_type: rock.into(),
                            current_amount: 1,
                        },
                        CellContent::Player(player) => PopupState::PlayerMenu {
                            player_id: player.id,
                            selected_action: PlayerAction::ViewInventory,
                        },
                        _ => return None,
                    },
                })
            } else {
                None
            }
        }
        _ => None,
    }
}

fn handle_resource_menu(
    key_event: KeyEvent,
    _resource_type: ResourceType,
    amount: u32,
    selected_cell: (usize, usize),
) -> Option<MapState> {
    match key_event.code {
        KeyCode::Char('+') => Some(MapState::Selected {
            selected_cell,
            popup_state: PopupState::ResourceMenu {
                resource_type: _resource_type,
                current_amount: amount + 1,
            },
        }),
        KeyCode::Char('-') if amount > 0 => Some(MapState::Selected {
            selected_cell,
            popup_state: PopupState::ResourceMenu {
                resource_type: _resource_type,
                current_amount: amount - 1,
            },
        }),
        KeyCode::Char('b') | KeyCode::Esc => Some(MapState::Selected {
            selected_cell,
            popup_state: PopupState::MainMenu { selected_item: 0 },
        }),
        _ => None,
    }
}

fn handle_player_menu(
    key_event: KeyEvent,
    player_id: u32,
    selected_cell: (usize, usize),
    selected_action: &PlayerAction,
) -> Option<MapState> {
    match key_event.code {
        KeyCode::Up | KeyCode::Down => Some(MapState::Selected {
            selected_cell,
            popup_state: PopupState::PlayerMenu {
                player_id,
                selected_action: match key_event.code {
                    KeyCode::Up => selected_action.previous(),
                    KeyCode::Down => selected_action.next(),
                    _ => unreachable!(),
                },
            },
        }),
        KeyCode::Enter => {
            Some(MapState::Selected {
                selected_cell,
                popup_state: PopupState::PlayerMenu {
                    player_id,
                    selected_action: PlayerAction::ViewInventory, // Keep current action
                },
            })
        }
        KeyCode::Char('b') | KeyCode::Esc => Some(MapState::Selected {
            selected_cell,
            popup_state: PopupState::MainMenu { selected_item: 0 },
        }),
        _ => None,
    }
}

fn handle_admin(app: &mut App, key_event: KeyEvent) {
    match key_event.code {
        KeyCode::Tab => {
            app.state = State::Options;
        }
        KeyCode::Esc => {
            app.state = State::Map {
                map: Map::new(10, 10),
                state: MapState::default(),
                vertical_scroll: 0,
            };
        }
        _ => {}
    }
}

fn handle_options(app: &mut App, key_event: KeyEvent) {
    match key_event.code {
        KeyCode::Tab => {
            app.state = State::Map {
                map: Map::new(10, 10),
                state: MapState::default(),
                vertical_scroll: 0,
            };
        }
        KeyCode::Esc => {
            app.state = State::Map {
                map: Map::new(10, 10),
                state: MapState::default(),
                vertical_scroll: 0,
            };
        }
        _ => {}
    }
}
