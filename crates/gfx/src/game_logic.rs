use std::ops::{Index, IndexMut};

#[derive(Debug)]
pub struct Map {
    pub x_max: usize,
    pub y_max: usize,
    pub cells: Vec<Cell>,
}

impl Map {
    pub fn new(x_max: usize, y_max: usize) -> Self {
        let size = x_max * y_max;
        let mut cells = Vec::with_capacity(size);
        for _ in 0..size {
            cells.push(Cell {
                content: Vec::new(),
            });
        }
        Map { x_max, y_max, cells }
    }
}

impl IndexMut<(usize, usize)> for Map {
    fn index_mut(&mut self, index: (usize, usize)) -> &mut Self::Output {
        &mut self.cells[index.0 * self.x_max + index.1]
    }
}

impl Index<(usize, usize)> for Map {
    type Output = Cell;

    fn index(&self, index: (usize, usize)) -> &Self::Output {
        &self.cells[index.0 * self.x_max + index.1]
    }
}

impl Index<usize> for Map {
    type Output = Cell;

    fn index(&self, index: usize) -> &Self::Output {
        &self.cells[index]
    }
}

#[derive(Debug, Default)]
pub struct Cell {
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

#[derive(Debug)]
pub enum CellContent {
    Rocks(Rocks),
    Food,
    Player(Player),
    Egg,
}

#[derive(Debug)]
pub struct Player {
    pub id: u32,
    pub level: u32,
    pub inventory: Vec<Rocks>,
    pub orientation: Orientation,
}

#[derive(Debug)]
pub enum Orientation {
    North,
    East,
    South,
    West,
}
