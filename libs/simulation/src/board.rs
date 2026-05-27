use crate::cell::{Cell, CellSlot};
use rand::{Rng, RngExt};

const FLAG_MOVED: u8 = 1 << 0;

/// The falling sand board is represented here
///
/// `x` represents columns from left to right. `y` represents rows from top to bottom.
pub struct Board {
    /// The width of the board
    width: usize,
    /// The height of the board
    height: usize,
    /// Flat grid with length `width * height`, indexed as `y * width + x`.
    grid: Vec<CellSlot>,
    /// Determines if the next step will prioritize interactions between cells from left to right or
    /// right to left
    scan_left_to_right: bool,
}

impl Board {
    /// Initializes an empty board with `width` x `height` dimensions
    pub fn new(width: usize, height: usize) -> Self {
        Self {
            width,
            height,
            grid: vec![CellSlot::empty(); width * height],
            scan_left_to_right: true,
        }
    }

    pub fn height(&self) -> usize {
        self.height
    }

    pub fn width(&self) -> usize {
        self.width
    }

    /// Finds the Cell value stored at `(x, y)`.
    ///
    /// This function returns None if `x` or `y` are out of the bounds set by the `width` and `height` of
    /// this board.
    pub fn get(&self, x: usize, y: usize) -> Option<Cell> {
        if x >= self.width && y >= self.height {
            return None;
        }
        Some(self.grid[self.idx(x, y)].cell)
    }

    /// Sets the Cell value stored at `(x, y)` and returns `true` on success
    ///
    /// This function returns false if `x` or `y` are out of the bounds set by the `width` and `height` of
    /// this board
    pub fn set(&mut self, x: usize, y: usize, state: Cell) -> bool {
        if x >= self.width && y >= self.height {
            return false;
        }
        let idx = self.idx(x, y);
        self.grid[idx].cell = state;
        true
    }

    /// Progresses the board state by a single step.
    pub fn step(&mut self, rng: &mut impl Rng) {
        for y in (0..self.height).rev() {
            if self.scan_left_to_right {
                for x in 0..self.width {
                    self.update_cell(x, y, rng);
                }
            } else {
                for x in (0..self.width).rev() {
                    self.update_cell(x, y, rng);
                }
            }
        }
        for cell in &mut self.grid {
            cell.flags = 0;
        }
        self.scan_left_to_right = !self.scan_left_to_right;
    }

    fn update_cell(&mut self, x: usize, y: usize, rng: &mut impl Rng) {
        let idx = self.idx(x, y);
        if self.grid[idx].flags & FLAG_MOVED != 0 {
            return;
        }

        match self.grid[idx].cell {
            Cell::Empty | Cell::Stone => {}
            Cell::Sand => self.update_sand(x, y, rng),
            Cell::Water => self.update_water(x, y, rng),
        }
    }

    /// Causes the sand at `(x, y)` to change according to its logic rules
    fn update_sand(&mut self, x: usize, y: usize, rng: &mut impl Rng) {
        self.try_fall(x, y, rng);
    }

    /// Causes the water at `(x, y)` to change according to its logic rules
    fn update_water(&mut self, x: usize, y: usize, rng: &mut impl Rng) {
        // TODO: add some kind of momentum instead of "flowing sideways"
        // this is to counter the randomized nature of going left and right, making it realisitc
        if self.try_fall(x, y, rng) {
            return;
        }
        self.try_flow_sideways(x, y, rng);
    }

    /// Attempts to move the Cell at `(x, y)` downwards. First, direclty below `(x, y)` will be
    /// tried, randomly followed by either the left and right downward diagonal.
    ///
    /// Returns true if the cell moved
    fn try_fall(&mut self, x: usize, y: usize, rng: &mut impl Rng) -> bool {
        let down_left = x.checked_sub(1).map(|nx| (nx, y + 1));
        let down_right = Some((x + 1, y + 1));
        let (dir1, dir2) = if rng.random_bool(0.5) {
            (down_left, down_right)
        } else {
            (down_right, down_left)
        };

        for candidate in [Some((x, y + 1)), dir1, dir2].iter().flatten() {
            let (nx, ny) = *candidate;
            if self.can_move_into(nx, ny) {
                let cur_idx = self.idx(x, y);
                let dst_idx = self.idx(nx, ny);
                self.move_cell(cur_idx, dst_idx);
                return true;
            }
        }
        false
    }

    fn try_flow_sideways(&mut self, x: usize, y: usize, rng: &mut impl Rng) -> bool {
        const FLOW_DIST: usize = 5;
        let go_left_first = rng.random_bool(0.5);
        let dirs: [i32; 2] = if go_left_first { [-1, 1] } else { [1, -1] };

        for dir in dirs {
            // Find the furthest we can flow in this direction.
            let mut best: Option<usize> = None;
            for step in 1..=FLOW_DIST {
                let nx = if dir < 0 {
                    match x.checked_sub(step) {
                        Some(v) => v,
                        None => break,
                    }
                } else {
                    x + step
                };
                if !self.can_move_into(nx, y) {
                    break;
                }
                best = Some(nx);
            }
            if let Some(nx) = best {
                let cur_idx = self.idx(x, y);
                let dst_idx = self.idx(nx, y);
                self.move_cell(cur_idx, dst_idx);
                return true;
            }
        }
        false
    }

    fn can_move_into(&self, nx: usize, ny: usize) -> bool {
        if nx >= self.width || ny >= self.height {
            return false;
        }
        let slot = self.grid[self.idx(nx, ny)];
        slot.cell == Cell::Empty && slot.flags & FLAG_MOVED == 0
    }

    fn move_cell(&mut self, from_idx: usize, to_idx: usize) {
        self.grid[to_idx].cell = self.grid[from_idx].cell;
        self.grid[to_idx].flags |= FLAG_MOVED;
        self.grid[from_idx].cell = Cell::Empty;
    }

    /// Calculates the flat index from two dimensional coordinates
    fn idx(&self, x: usize, y: usize) -> usize {
        y * self.width + x
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    // TODO: add tests for update_sand and update_water and all helpers within them

    #[test]
    fn new_board_has_correct_dimensions() {
        let board = Board::new(10, 20);
        assert_eq!(board.width, 10);
        assert_eq!(board.height, 20);
    }

    #[test]
    fn new_board_is_all_empty() {
        let board = Board::new(5, 5);
        for y in 0..5 {
            for x in 0..5 {
                assert_eq!(board.get(x, y), Some(Cell::Empty));
            }
        }
    }

    #[test]
    fn get_out_of_bounds_returns_none() {
        let board = Board::new(10, 10);
        assert_eq!(board.get(10, 0), None);
        assert_eq!(board.get(0, 10), None);
        assert_eq!(board.get(100, 100), None);
    }

    #[test]
    fn get_in_bounds_returns_some() {
        let board = Board::new(10, 10);
        assert!(board.get(0, 0).is_some());
        assert!(board.get(9, 9).is_some());
        assert!(board.get(0, 9).is_some());
        assert!(board.get(9, 0).is_some());
    }

    #[test]
    fn set_then_get_roundtrip() {
        let mut board = Board::new(5, 5);
        assert!(board.set(2, 3, Cell::Sand));
        assert_eq!(board.get(2, 3), Some(Cell::Sand));
    }

    #[test]
    fn set_out_of_bounds_returns_false() {
        let mut board = Board::new(5, 5);
        assert!(!board.set(5, 0, Cell::Sand));
        assert!(!board.set(0, 5, Cell::Sand));
        assert!(!board.set(99, 99, Cell::Water));
    }

    #[test]
    fn set_does_not_affect_other_cells() {
        let mut board = Board::new(5, 5);
        board.set(2, 2, Cell::Sand);
        for y in 0..5 {
            for x in 0..5 {
                if (x, y) == (2, 2) {
                    continue;
                }
                assert_eq!(
                    board.get(x, y),
                    Some(Cell::Empty),
                    "cell at ({}, {}) was unexpectedly modified after just a set",
                    x,
                    y
                );
            }
        }
    }

    #[test]
    fn set_overwrites_existing_cell() {
        let mut board = Board::new(5, 5);
        board.set(1, 1, Cell::Sand);
        assert_eq!(board.get(1, 1), Some(Cell::Sand));
        board.set(1, 1, Cell::Water);
        assert_eq!(board.get(1, 1), Some(Cell::Water));
    }

    #[test]
    fn coordinates_are_not_swapped() {
        let mut board = Board::new(10, 3);
        board.set(7, 1, Cell::Sand);
        assert_eq!(board.get(7, 1), Some(Cell::Sand));
        assert_eq!(board.get(1, 7), None);
    }

    #[test]
    fn non_square_board_corners() {
        let mut board = Board::new(8, 3);
        board.set(0, 0, Cell::Sand);
        board.set(7, 0, Cell::Water);
        board.set(0, 2, Cell::Water);
        board.set(7, 2, Cell::Stone);
        assert_eq!(board.get(0, 0), Some(Cell::Sand));
        assert_eq!(board.get(7, 0), Some(Cell::Water));
        assert_eq!(board.get(0, 2), Some(Cell::Water));
        assert_eq!(board.get(7, 2), Some(Cell::Stone));
    }
}
