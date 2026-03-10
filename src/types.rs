use serde::Deserialize;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Direction {
    Right,
    Left,
    Up,
    Down,
}

impl Direction {
    /// Returns (row_delta, col_delta) for stepping in this direction.
    /// Row increases downward, col increases rightward.
    pub fn to_offset(self) -> (i32, i32) {
        match self {
            Direction::Right => (0, 1),
            Direction::Left => (0, -1),
            Direction::Down => (1, 0),
            Direction::Up => (-1, 0),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct GridPos {
    pub row: i32,
    pub col: i32,
}

impl GridPos {
    pub fn new(row: i32, col: i32) -> Self {
        Self { row, col }
    }

    pub fn step(self, dir: Direction) -> Self {
        let (dr, dc) = dir.to_offset();
        Self {
            row: self.row + dr,
            col: self.col + dc,
        }
    }

    pub fn in_bounds(self, size: i32) -> bool {
        self.row >= 0 && self.row < size && self.col >= 0 && self.col < size
    }
}
