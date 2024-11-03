use crate::app::state::ResourceType;
use std::fmt::Write;
use std::fmt::{Display, Formatter};
use std::ops::{Index, IndexMut};

#[derive(Debug)]
pub struct Map {
    pub x_max: usize,
    pub y_max: usize,
    pub cells: Vec<MapCell>,
}

impl Map {
    pub fn new(x_max: usize, y_max: usize) -> Self {
        let size = x_max * y_max;
        let mut cells = Vec::with_capacity(size);
        for _ in 0..size {
            cells.push(MapCell {
                content: Vec::new(),
            });
        }
        Map {
            x_max,
            y_max,
            cells,
        }
    }
}

impl IndexMut<(usize, usize)> for Map {
    fn index_mut(&mut self, index: (usize, usize)) -> &mut Self::Output {
        &mut self.cells[index.0 * self.x_max + index.1]
    }
}

impl Index<(usize, usize)> for Map {
    type Output = MapCell;

    fn index(&self, index: (usize, usize)) -> &Self::Output {
        &self.cells[index.0 * self.x_max + index.1]
    }
}

impl Index<usize> for Map {
    type Output = MapCell;

    fn index(&self, index: usize) -> &Self::Output {
        &self.cells[index]
    }
}

#[derive(Debug, Default)]
pub struct MapCell {
    pub content: Vec<CellContent>,
}

#[derive(Debug)]
pub enum Rocks {
    Linemate,
    Deraumere,
    Sibur,
    Mendiane,
    Phiras,
    Thystame,
}

impl From<&Rocks> for ResourceType {
    fn from(rocks: &Rocks) -> Self {
        match rocks {
            Rocks::Linemate => ResourceType::Linemate,
            Rocks::Deraumere => ResourceType::Deraumere,
            Rocks::Sibur => ResourceType::Sibur,
            Rocks::Mendiane => ResourceType::Mendiane,
            Rocks::Phiras => ResourceType::Phiras,
            Rocks::Thystame => ResourceType::Thystame,
        }
    }
}

impl Display for Rocks {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

#[derive(Debug)]
pub enum CellContent {
    Rocks(Rocks),
    Food,
    Player(Player),
    Egg,
}

impl Display for CellContent {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            CellContent::Rocks(rocks) => write!(f, "{}", rocks),
            CellContent::Food => write!(f, "Food"),
            CellContent::Player(player) => write!(f, "{}", player),
            CellContent::Egg => write!(f, "Egg"),
        }
    }
}

#[derive(Debug)]
pub struct Player {
    pub id: u32,
    pub level: u32,
    pub inventory: Vec<Rocks>,
    pub orientation: Orientation,
}

impl Display for Player {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let mut buf = String::with_capacity(50);
        writeln!(buf, "P{}", self.id)?;
        writeln!(buf, "Level: {}", self.level)?;
        writeln!(buf, "Inventory: {:#?}", self.inventory)?;
        writeln!(buf, "Orientation: {:#?}", self.orientation)?;
        write!(f, "{}", buf)
    }
}

#[derive(Debug)]
pub enum Orientation {
    North,
    East,
    South,
    West,
}
