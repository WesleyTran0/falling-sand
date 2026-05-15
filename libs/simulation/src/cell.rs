#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Cell {
    Empty,
    Sand,
    Water,
    Stone,
}

#[derive(Clone, Copy, Debug)]
pub struct CellSlot {
    pub cell: Cell,
    pub flags: u8,
}

impl CellSlot {
    pub fn empty() -> Self {
        Self {
            cell: Cell::Empty,
            flags: 0,
        }
    }
}
