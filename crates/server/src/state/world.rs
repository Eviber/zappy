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
    width: u32,
    height: u32,
}

impl World {
    /// Creates a new [`World`] with the specified dimensions.
    pub fn new(width: u32, height: u32) -> Self {
        Self { width, height }
    }

    /// Returns the width of the world.
    #[inline]
    pub fn width(&self) -> u32 {
        self.width
    }

    /// Returns the height of the world.
    #[inline]
    pub fn height(&self) -> u32 {
        self.height
    }
}
