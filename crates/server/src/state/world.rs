use crate::Vec;
use alloc::vec;

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

/// The world state.
pub struct World {
    /// The width of the world.
    pub width: u32,
    /// The height of the world.
    pub height: u32,
    pub cells: Vec<[u32; 7]>,
}

impl World {
    /// Creates a new [`World`] with the specified dimensions.
    pub fn new(width: u32, height: u32, rng: &mut Rng) -> Self {
        let mut cells = vec![[0; 7]; (width * height) as usize];
        for i in 0..(width * height) as usize {
            for j in 0..7 {
                let random = rng.next_u64() % 32;
                if random < 8 {
                    cells[i][j] = (random / 2) as u32;
                }
            }
        }
        Self {
            width,
            height,
            cells,
        }
    }
}
