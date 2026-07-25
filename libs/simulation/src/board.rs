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
        if x >= self.width || y >= self.height {
            return None;
        }
        Some(self.grid[self.idx(x, y)].cell)
    }

    /// Sets the Cell value stored at `(x, y)` and returns `true` on success
    ///
    /// This function returns false if `x` or `y` are out of the bounds set by the `width` and `height` of
    /// this board
    pub fn set(&mut self, x: usize, y: usize, state: Cell) -> bool {
        if x >= self.width || y >= self.height {
            return false;
        }
        let idx = self.idx(x, y);
        self.grid[idx].cell = state;
        // A freshly placed cell inherits no momentum from whatever occupied the
        // slot before it.
        self.grid[idx].dir = 0;
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

    /// Causes the water at `(x, y)` to change according to its logic rules.
    ///
    /// Water falls first; if it cannot, it flows one cell sideways in its
    /// stored horizontal direction, carrying that momentum so the motion looks
    /// like flow rather than random left/right jitter (see `try_flow_sideways`).
    fn update_water(&mut self, x: usize, y: usize, rng: &mut impl Rng) {
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

    /// Attempts to flow the cell at `(x, y)` one cell sideways, honoring its
    /// stored horizontal momentum.
    ///
    /// The cell prefers its current momentum direction; if that neighbor is
    /// blocked it tries the opposite. On a move it adopts that direction as its
    /// momentum, so a body of water flows coherently instead of re-randomizing
    /// left/right every step. Falling is always attempted first (in
    /// `update_water`), so this only runs when the cell cannot descend; moving
    /// one cell per step lets it slide onto and then fall down any slope, which
    /// is what levels a mound of water. If both sides are blocked the cell rests
    /// and its momentum is cleared so it re-picks a direction if things later
    /// open up.
    ///
    /// Returns true if the cell moved.
    fn try_flow_sideways(&mut self, x: usize, y: usize, rng: &mut impl Rng) -> bool {
        let idx = self.idx(x, y);
        let mut dir = self.grid[idx].dir;
        if dir == 0 {
            dir = if rng.random_bool(0.5) { 1 } else { -1 };
        }

        // Try the momentum direction first, then the opposite.
        for d in [dir, -dir] {
            let Some(nx) = offset_x(x, d) else { continue };
            if self.can_move_into(nx, y) {
                self.grid[idx].dir = d;
                let dst_idx = self.idx(nx, y);
                self.move_cell(idx, dst_idx);
                return true;
            }
        }

        // Boxed in on both sides: settle and drop momentum.
        self.grid[idx].dir = 0;
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
        // Momentum travels with the cell; the vacated slot is now empty and
        // carries no direction.
        self.grid[to_idx].dir = self.grid[from_idx].dir;
        self.grid[from_idx].cell = Cell::Empty;
        self.grid[from_idx].dir = 0;
    }

    /// Calculates the flat index from two dimensional coordinates
    fn idx(&self, x: usize, y: usize) -> usize {
        y * self.width + x
    }
}

/// Offsets grid column `x` by horizontal direction `d` (`-1` or `+1`),
/// returning `None` if the result would be negative (off the left edge). The
/// right-edge / height bounds are left to the storage boundary
/// (`can_move_into`).
fn offset_x(x: usize, d: i8) -> Option<usize> {
    if d < 0 { x.checked_sub(1) } else { Some(x + 1) }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rand::SeedableRng;
    use rand::rngs::SmallRng;

    /// Convenience: read the persistent momentum stored at `(x, y)`.
    fn dir_at(board: &Board, x: usize, y: usize) -> i8 {
        board.grid[board.idx(x, y)].dir
    }

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

    // --- physics: falling ---

    #[test]
    fn water_falls_straight_down() {
        let mut board = Board::new(5, 5);
        let mut rng = SmallRng::seed_from_u64(7);
        board.set(2, 0, Cell::Water);
        board.step(&mut rng);
        assert_eq!(board.get(2, 1), Some(Cell::Water));
        assert_eq!(board.get(2, 0), Some(Cell::Empty));
    }

    #[test]
    fn water_falls_diagonally_when_blocked_below() {
        let mut board = Board::new(5, 5);
        let mut rng = SmallRng::seed_from_u64(7);
        board.set(2, 2, Cell::Water);
        board.set(2, 3, Cell::Stone); // block straight down
        board.step(&mut rng);
        let went_left = board.get(1, 3) == Some(Cell::Water);
        let went_right = board.get(3, 3) == Some(Cell::Water);
        assert!(
            went_left ^ went_right,
            "water should fall to exactly one diagonal"
        );
        assert_eq!(board.get(2, 2), Some(Cell::Empty));
        assert_eq!(board.get(2, 3), Some(Cell::Stone));
    }

    // --- physics: momentum-driven sideways flow ---

    #[test]
    fn move_cell_carries_momentum_and_clears_source() {
        let mut board = Board::new(5, 5);
        board.set(2, 2, Cell::Water);
        let from = board.idx(2, 2);
        board.grid[from].dir = 1;
        let to = board.idx(2, 3);
        board.move_cell(from, to);
        assert_eq!(board.get(2, 3), Some(Cell::Water));
        assert_eq!(board.get(2, 2), Some(Cell::Empty));
        assert_eq!(
            dir_at(&board, 2, 3),
            1,
            "momentum should travel with the cell"
        );
        assert_eq!(dir_at(&board, 2, 2), 0, "vacated slot carries no momentum");
    }

    #[test]
    fn set_resets_momentum() {
        let mut board = Board::new(5, 5);
        let idx = board.idx(1, 1);
        board.grid[idx].dir = -1;
        board.set(1, 1, Cell::Water);
        assert_eq!(
            dir_at(&board, 1, 1),
            0,
            "a freshly set cell has no momentum"
        );
    }

    #[test]
    fn water_flows_exactly_one_cell_sideways() {
        // A cell that cannot fall steps a single cell in its momentum
        // direction — flowing, not teleporting across the open run.
        let mut board = Board::new(8, 4);
        let mut rng = SmallRng::seed_from_u64(1);
        for x in 0..8 {
            board.set(x, 2, Cell::Stone);
        }
        board.set(1, 1, Cell::Water);
        let idx = board.idx(1, 1);
        board.grid[idx].dir = 1;

        let moved = board.try_flow_sideways(1, 1, &mut rng);

        assert!(moved);
        assert_eq!(
            board.get(2, 1),
            Some(Cell::Water),
            "flowed exactly one cell, not teleported"
        );
        assert_eq!(board.get(1, 1), Some(Cell::Empty));
        assert_eq!(dir_at(&board, 2, 1), 1, "kept its momentum direction");
    }

    #[test]
    fn momentum_direction_wins_when_both_sides_are_open() {
        // Both neighbors are open; a rightward-moving cell must go right.
        let mut board = Board::new(9, 4);
        let mut rng = SmallRng::seed_from_u64(2);
        for x in 0..9 {
            board.set(x, 2, Cell::Stone);
        }
        board.set(4, 1, Cell::Water);
        let idx = board.idx(4, 1);
        board.grid[idx].dir = 1;

        assert!(board.try_flow_sideways(4, 1, &mut rng));
        assert_eq!(
            board.get(5, 1),
            Some(Cell::Water),
            "momentum should carry it right"
        );
        assert_eq!(board.get(3, 1), Some(Cell::Empty));
    }

    #[test]
    fn lone_water_on_flat_floor_slides_with_its_momentum() {
        // The simple rule has no "reachable drop" gate, so a lone cell that
        // cannot fall just slides one cell in its momentum direction. This
        // minor wander is the accepted trade-off for the rule that lets water
        // flow down and level its own slopes (see the hill it otherwise forms).
        let mut board = Board::new(7, 3);
        let mut rng = SmallRng::seed_from_u64(3);
        for x in 0..7 {
            board.set(x, 2, Cell::Stone);
        }
        board.set(3, 1, Cell::Water);
        let idx = board.idx(3, 1);
        board.grid[idx].dir = 1;

        board.update_water(3, 1, &mut rng);

        assert_eq!(board.get(4, 1), Some(Cell::Water), "slides one cell right");
        assert_eq!(board.get(3, 1), Some(Cell::Empty));
        assert_eq!(dir_at(&board, 4, 1), 1, "keeps its momentum");
    }

    #[test]
    fn water_filling_the_floor_wall_to_wall_is_stable() {
        // A one-deep sheet spanning the whole container has nowhere to fall and
        // nowhere to spread, so it must rest.
        let mut board = Board::new(6, 3);
        let mut rng = SmallRng::seed_from_u64(4);
        for x in 0..6 {
            board.set(x, 2, Cell::Stone);
            board.set(x, 1, Cell::Water);
        }

        board.step(&mut rng);

        for x in 0..6 {
            assert_eq!(
                board.get(x, 1),
                Some(Cell::Water),
                "water at ({x}, 1) should stay put"
            );
        }
    }

    #[test]
    fn water_column_levels_by_flowing_sideways() {
        // A column wedged against a wall with a stone floor cannot fall or fall
        // diagonally (both are blocked), so only sideways flow can level it.
        // After settling it must be one cell deep — no vertical stack — which
        // is exactly the leveling that keeps water from piling into a hill.
        let mut board = Board::new(6, 4);
        let mut rng = SmallRng::seed_from_u64(5);
        for x in 0..6 {
            board.set(x, 3, Cell::Stone); // floor blocks straight + diagonal falls
        }
        board.set(0, 1, Cell::Water);
        board.set(0, 2, Cell::Water); // a two-tall column in the left corner

        for _ in 0..12 {
            board.step(&mut rng);
        }

        let water: Vec<(usize, usize)> = (0..6)
            .flat_map(|x| (0..4).map(move |y| (x, y)))
            .filter(|&(x, y)| board.get(x, y) == Some(Cell::Water))
            .collect();
        assert_eq!(water.len(), 2, "conservation: still two water cells");
        assert!(
            water.iter().all(|&(_, y)| y == 2),
            "column has leveled to a single layer resting on the floor: {water:?}"
        );
        assert!(
            water[0].0 != water[1].0,
            "the two cells sit in different columns, not stacked: {water:?}"
        );
    }
}
