use crate::Vec;

use crate::state::rng::Rng;

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

struct Inventory {
    /// Food.
    food: u32,
    /// Linemate.
    linemate: u32,
    /// Deraumere.
    deraumere: u32,
    /// Sibur.
    sibur: u32,
    /// Mendiane.
    mendiane: u32,
    /// Phiras.
    phiras: u32,
    /// Thystame.
    thystame: u32,
}

#[derive(Clone, Copy, Debug)]
pub struct WorldCell {
    /// Food.
    food: u32,
    /// Linemate.
    linemate: u32,
    /// Deraumere.
    deraumere: u32,
    /// Sibur.
    sibur: u32,
    /// Mendiane.
    mendiane: u32,
    /// Phiras.
    phiras: u32,
    /// Thystame.
    thystame: u32,
    /// Player count.
    player_count: u32,
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

/// The world state.
pub struct World {
    /// The width of the world.
    pub width: u32,
    /// The height of the world.
    pub height: u32,
    pub cells: Vec<WorldCell>,
}

impl World {
    /// Creates a new [`World`] with the specified dimensions.
    pub fn new(width: u32, height: u32, rng: &mut Rng) -> Self {
        let cells_count = (width * height) as usize;
        let cells = (0..cells_count).map(|_| WorldCell::new(rng)).collect();
        Self {
            width,
            height,
            cells,
        }
    }
}
