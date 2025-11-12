use crate::game_logic::Map;

#[derive(Debug)]
pub enum State {
    Map {
        map: Map,
        state: MapState,
        vertical_scroll: usize,
    },
    Admin,
    Options,
}

impl State {
    pub fn map(&self) -> Option<&Map> {
        match self {
            State::Map { map, .. } => Some(map),
            _ => None,
        }
    }

    pub fn map_mut(&mut self) -> Option<&mut Map> {
        match self {
            State::Map { map, .. } => Some(map),
            _ => None,
        }
    }

    pub fn is_popup(&self) -> bool {
        match self {
            State::Map { state, .. } => {
                matches!(state, MapState::Selected { .. })
            }
            _ => false,
        }
    }

    pub fn selected_cell(&self) -> Option<(usize, usize)> {
        match self {
            State::Map {
                state: MapState::Selected { selected_cell, .. },
                ..
            } => Some(*selected_cell),
            _ => None,
        }
    }
}

impl Default for State {
    fn default() -> Self {
        State::Map {
            map: Map::new(10, 10),
            state: MapState::default(),
            vertical_scroll: 0,
        }
    }
}

#[derive(Debug)]
pub enum PopupState {
    MainMenu {
        selected_item: usize,
    },
    ResourceMenu {
        resource_type: ResourceType,
        current_amount: u32,
    },
    PlayerMenu {
        player_id: u32,
        selected_action: PlayerAction,
    },
}

#[derive(Debug, Clone, Copy)]
pub enum ResourceType {
    Food,
    Linemate,
    Deraumere,
    Sibur,
    Mendiane,
    Phiras,
    Thystame,
}

#[derive(Debug, Clone, Copy)]
pub enum PlayerAction {
    ViewInventory,
    ViewFOV,
    Back,
}

impl PlayerAction {
    pub fn next(&self) -> Self {
        match self {
            Self::ViewInventory => Self::ViewFOV,
            Self::ViewFOV => Self::Back,
            Self::Back => Self::ViewInventory,
        }
    }

    pub fn previous(&self) -> Self {
        match self {
            Self::ViewInventory => Self::Back,
            Self::ViewFOV => Self::ViewInventory,
            Self::Back => Self::ViewFOV,
        }
    }
}

#[derive(Debug)]
pub enum MapState {
    Selecting((usize, usize)),
    Selected {
        selected_cell: (usize, usize),
        popup_state: PopupState,
    },
}

impl Default for MapState {
    fn default() -> Self {
        MapState::Selecting((0, 0))
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub enum PopupCommand {
    #[default]
    Command1,
    Command2,
    Command3,
}

impl From<usize> for PopupCommand {
    fn from(mut index: usize) -> Self {
        index %= 3;
        match index {
            0 => PopupCommand::Command1,
            1 => PopupCommand::Command2,
            2 => PopupCommand::Command3,
            _ => unreachable!("Invalid index for PopupCommand"),
        }
    }
}
