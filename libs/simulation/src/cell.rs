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
    /// Per-step scratch flags (e.g. `FLAG_MOVED`). Cleared at the end of every
    /// `Board::step`, so this must not hold state that outlives a single step.
    pub flags: u8,
    /// Persistent horizontal momentum for flowing cells: `-1` (left), `0`
    /// (none), or `+1` (right). Unlike `flags`, this survives across steps and
    /// travels with the cell when it moves, so water keeps flowing coherently
    /// in one direction instead of re-randomizing every step.
    pub dir: i8,
}

impl CellSlot {
    pub fn empty() -> Self {
        Self {
            cell: Cell::Empty,
            flags: 0,
            dir: 0,
        }
    }
}
