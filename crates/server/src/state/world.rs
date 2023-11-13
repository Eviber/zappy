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
