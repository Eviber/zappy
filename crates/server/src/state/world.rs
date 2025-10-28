use crate::Vec;
use alloc::vec;

/// The class of an object.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ObjectClass {
    /// Food.
    Food,
    /// Linemate.
    Linemate,
    /// Deraumere.
    Deraumere,
    /// Sibur.
    Sibur,
    /// Mendiane.
    Mendiane,
    /// Phiras.
    Phiras,
    /// Thystame.
    Thystame,
}

impl ObjectClass {
    /// Parses an object class from the provided argument.
    pub fn from_arg(bytes: &[u8]) -> Option<Self> {
        match bytes {
            b"nourriture" => Some(Self::Food),
            b"linemate" => Some(Self::Linemate),
            b"deraumere" => Some(Self::Deraumere),
            b"sibur" => Some(Self::Sibur),
            b"mendiane" => Some(Self::Mendiane),
            b"phiras" => Some(Self::Phiras),
            b"thystame" => Some(Self::Thystame),
            _ => None,
        }
    }
}

#[derive(Clone, Copy, Debug, Default)]
pub struct WorldCell {
    /// Food.
    pub food: u32,
    /// Linemate.
    pub linemate: u32,
    /// Deraumere.
    pub deraumere: u32,
    /// Sibur.
    pub sibur: u32,
    /// Mendiane.
    pub mendiane: u32,
    /// Phiras.
    pub phiras: u32,
    /// Thystame.
    pub thystame: u32,
    /// Player count.
    pub player_count: u32,
    /// Egg count.
    pub egg_count: u32,
}

impl WorldCell {
    pub fn count(&self, object: ObjectClass) -> u32 {
        match object {
            ObjectClass::Food => self.food,
            ObjectClass::Linemate => self.linemate,
            ObjectClass::Deraumere => self.deraumere,
            ObjectClass::Sibur => self.sibur,
            ObjectClass::Mendiane => self.mendiane,
            ObjectClass::Phiras => self.phiras,
            ObjectClass::Thystame => self.thystame,
        }
    }

    pub fn add_one(&mut self, object: ObjectClass) {
        match object {
            ObjectClass::Food => self.food += 1,
            ObjectClass::Linemate => self.linemate += 1,
            ObjectClass::Deraumere => self.deraumere += 1,
            ObjectClass::Sibur => self.sibur += 1,
            ObjectClass::Mendiane => self.mendiane += 1,
            ObjectClass::Phiras => self.phiras += 1,
            ObjectClass::Thystame => self.thystame += 1,
        }
    }

    pub fn remove_one(&mut self, object: ObjectClass) {
        match object {
            ObjectClass::Food => self.food -= 1,
            ObjectClass::Linemate => self.linemate -= 1,
            ObjectClass::Deraumere => self.deraumere -= 1,
            ObjectClass::Sibur => self.sibur -= 1,
            ObjectClass::Mendiane => self.mendiane -= 1,
            ObjectClass::Phiras => self.phiras -= 1,
            ObjectClass::Thystame => self.thystame -= 1,
        }
    }
}

/// The world state.
pub struct World {
    /// The width of the world.
    pub width: usize,
    /// The height of the world.
    pub height: usize,
    /// The contents of the world
    pub cells: Vec<WorldCell>,
}

impl World {
    /// Creates a new [`World`] with the specified dimensions.
    pub fn new(width: usize, height: usize) -> Self {
        Self {
            width,
            height,
            cells: vec![WorldCell::default(); width * height],
        }
    }
}
