use crate::Vec;
use crate::player::PlayerInventory;
use crate::rng::Rng;
use core::ops::{Index, IndexMut};

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

    pub fn try_pick_up_object(
        cell: &mut WorldCell,
        inventory: &mut PlayerInventory,
        object: ObjectClass,
    ) -> bool {
        if cell[object] > 0 {
            cell[object] -= 1;
            match object {
                // Food is represented as 126 time_to_live in PlayerInventory
                ObjectClass::Food => inventory.time_to_live += 126,
                _ => inventory[object] += 1,
            }
            return true;
        }
        false
    }

    pub fn try_drop_object(
        inventory: &mut PlayerInventory,
        cell: &mut WorldCell,
        object: ObjectClass,
    ) -> bool {
        match object {
            // Food is represented as 126 time_to_live in PlayerInventory
            ObjectClass::Food => {
                if inventory.time_to_live >= 126 {
                    inventory.time_to_live -= 126;
                    cell[object] += 1;
                    return true;
                }
                false
            }
            _ => {
                if inventory[object] > 0 {
                    inventory[object] -= 1;
                    cell[object] += 1;
                    return true;
                }
                false
            }
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
    pub fn new(rng: &mut Rng) -> Self {
        Self {
            food: Self::random_item(rng),
            linemate: Self::random_item(rng),
            deraumere: Self::random_item(rng),
            sibur: Self::random_item(rng),
            mendiane: Self::random_item(rng),
            phiras: Self::random_item(rng),
            thystame: Self::random_item(rng),
            player_count: 0,
            egg_count: 0,
        }
    }

    fn random_item(rng: &mut Rng) -> u32 {
        let random = rng.next_u64() % 16;
        if random < 4 {
            return random as u32;
        }
        0
    }
}

impl Index<ObjectClass> for WorldCell {
    type Output = u32;

    fn index(&self, object: ObjectClass) -> &Self::Output {
        match object {
            ObjectClass::Food => &self.food,
            ObjectClass::Linemate => &self.linemate,
            ObjectClass::Deraumere => &self.deraumere,
            ObjectClass::Sibur => &self.sibur,
            ObjectClass::Mendiane => &self.mendiane,
            ObjectClass::Phiras => &self.phiras,
            ObjectClass::Thystame => &self.thystame,
        }
    }
}

impl IndexMut<ObjectClass> for WorldCell {
    fn index_mut(&mut self, object: ObjectClass) -> &mut Self::Output {
        match object {
            ObjectClass::Food => &mut self.food,
            ObjectClass::Linemate => &mut self.linemate,
            ObjectClass::Deraumere => &mut self.deraumere,
            ObjectClass::Sibur => &mut self.sibur,
            ObjectClass::Mendiane => &mut self.mendiane,
            ObjectClass::Phiras => &mut self.phiras,
            ObjectClass::Thystame => &mut self.thystame,
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
    pub fn new(width: usize, height: usize, rng: &mut Rng) -> Self {
        let cells_count = width * height;
        let cells = (0..cells_count).map(|_| WorldCell::new(rng)).collect();
        Self {
            width,
            height,
            cells,
        }
    }
}
