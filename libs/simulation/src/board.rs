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
    use rand::SeedableRng;
    use rand::rngs::SmallRng;

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

    // --- Sand: `try_fall` / `update_sand` ---

    #[test]
    fn sand_falls_straight_down_one_row_per_step() {
        let mut board = Board::new(3, 10);
        let mut rng = SmallRng::seed_from_u64(1);
        board.set(1, 0, Cell::Sand);

        board.step(&mut rng);

        // `(x, y + 1)` is tried before either diagonal, so the coin flip in
        // `try_fall` cannot affect this outcome.
        assert_eq!(board.get(1, 1), Some(Cell::Sand));
        assert_eq!(board.get(1, 0), Some(Cell::Empty));
    }

    #[test]
    fn sand_falls_one_row_per_step_not_more() {
        let mut board = Board::new(3, 10);
        let mut rng = SmallRng::seed_from_u64(2);
        board.set(1, 0, Cell::Sand);

        for y in 1..10 {
            board.step(&mut rng);
            assert_eq!(
                board.get(1, y),
                Some(Cell::Sand),
                "sand should descend exactly one row per step, expected it at (1, {y})"
            );
            assert_eq!(
                board.get(1, y - 1),
                Some(Cell::Empty),
                "sand was duplicated: (1, {}) is still occupied",
                y - 1
            );
        }

        // On the bottom row there is nowhere left to go.
        board.step(&mut rng);
        assert_eq!(board.get(1, 9), Some(Cell::Sand));
    }

    #[test]
    fn sand_on_bottom_row_does_not_move() {
        let mut board = Board::new(3, 3);
        let mut rng = SmallRng::seed_from_u64(3);
        board.set(1, 2, Cell::Sand);

        board.step(&mut rng);

        // All three candidates have `ny == height`, so `can_move_into` rejects them.
        assert_eq!(board.get(1, 2), Some(Cell::Sand));
    }

    #[test]
    fn sand_slides_to_the_only_open_diagonal() {
        let mut board = Board::new(3, 3);
        let mut rng = SmallRng::seed_from_u64(4);
        board.set(1, 0, Cell::Sand);
        board.set(1, 1, Cell::Stone);
        board.set(2, 1, Cell::Stone);

        board.step(&mut rng);

        // Down-left is the only legal candidate, so the coin flip order is irrelevant.
        assert_eq!(board.get(0, 1), Some(Cell::Sand));
        assert_eq!(board.get(1, 0), Some(Cell::Empty));
    }

    #[test]
    fn sand_fully_blocked_does_not_move() {
        let mut board = Board::new(3, 3);
        let mut rng = SmallRng::seed_from_u64(5);
        board.set(1, 0, Cell::Sand);
        board.set(0, 1, Cell::Stone);
        board.set(1, 1, Cell::Stone);
        board.set(2, 1, Cell::Stone);

        board.step(&mut rng);

        assert_eq!(board.get(1, 0), Some(Cell::Sand));
    }

    #[test]
    fn sand_column_falls_without_gaps() {
        let mut board = Board::new(3, 10);
        let mut rng = SmallRng::seed_from_u64(6);
        board.set(1, 0, Cell::Sand);
        board.set(1, 1, Cell::Sand);
        board.set(1, 2, Cell::Sand);

        board.step(&mut rng);

        // The bottom-up scan frees each destination before the cell above is
        // considered, so the column stays contiguous. Under a top-down scan the
        // top cell is processed first, finds `(1, 1)` still occupied, and slides
        // diagonally to `(0, 1)` instead — which is why this pins scan order.
        assert_eq!(board.get(1, 0), Some(Cell::Empty));
        assert_eq!(board.get(1, 1), Some(Cell::Sand));
        assert_eq!(board.get(1, 2), Some(Cell::Sand));
        assert_eq!(board.get(1, 3), Some(Cell::Sand));
    }

    // --- Water: `update_water` / `try_flow_sideways` ---

    #[test]
    fn water_prefers_falling_over_flowing() {
        let mut board = Board::new(3, 10);
        let mut rng = SmallRng::seed_from_u64(7);
        board.set(1, 0, Cell::Water);

        board.step(&mut rng);

        // `update_water` returns as soon as `try_fall` succeeds.
        assert_eq!(board.get(1, 1), Some(Cell::Water));
        assert_eq!(board.get(1, 0), Some(Cell::Empty));
    }

    #[test]
    fn water_flows_sideways_when_it_cannot_fall() {
        let mut board = Board::new(11, 10);
        let mut rng = SmallRng::seed_from_u64(8);
        board.set(5, 9, Cell::Water);

        board.step(&mut rng);

        // Both directions are genuinely legal here, so assert membership in the
        // allowed set rather than one exact coordinate.
        let found: Vec<(usize, usize)> = (0..board.width())
            .flat_map(|x| (0..board.height()).map(move |y| (x, y)))
            .filter(|&(x, y)| board.get(x, y) == Some(Cell::Water))
            .collect();
        assert_eq!(found.len(), 1, "water should not be duplicated: {found:?}");
        let (x, y) = found[0];
        assert_eq!(y, 9, "water must not change rows when it flows sideways");
        assert!(
            x == 0 || x == 10,
            "water flowed to x = {x}, expected 0 or 10"
        );
    }

    #[test]
    fn water_blocked_on_both_sides_does_not_move() {
        let mut board = Board::new(11, 10);
        let mut rng = SmallRng::seed_from_u64(9);
        board.set(5, 9, Cell::Water);
        board.set(4, 9, Cell::Stone);
        board.set(6, 9, Cell::Stone);

        board.step(&mut rng);

        // Both direction loops break on their first step, leaving `best == None`.
        assert_eq!(board.get(5, 9), Some(Cell::Water));
    }

    #[test]
    fn water_flow_stops_before_an_obstacle() {
        let mut board = Board::new(20, 10);
        let mut rng = SmallRng::seed_from_u64(10);
        board.set(5, 9, Cell::Water);
        board.set(2, 9, Cell::Stone);
        board.set(6, 9, Cell::Stone);

        board.step(&mut rng);

        // Rightward yields `None`, leftward yields 3 — deterministic either way.
        assert_eq!(board.get(3, 9), Some(Cell::Water));
        assert_eq!(board.get(2, 9), Some(Cell::Stone));
        assert_eq!(board.get(5, 9), Some(Cell::Empty));
    }

    /// Pins `FLOW_DIST = 5`: water teleports up to five cells sideways in a
    /// single step.
    ///
    /// This is a current limitation, not desired behavior. Ticket W2 replaces
    /// the teleport with per-step momentum and must delete or rewrite this test
    /// rather than loosen the assertion.
    #[test]
    fn water_flows_at_most_flow_dist_cells_per_step() {
        let mut board = Board::new(20, 10);
        let mut rng = SmallRng::seed_from_u64(11);
        board.set(9, 9, Cell::Water);
        board.set(10, 9, Cell::Stone);

        board.step(&mut rng);

        // Rightward yields `None`, so the move is deterministic: exactly five
        // cells to the left, no further.
        assert_eq!(board.get(4, 9), Some(Cell::Water));
        assert_eq!(board.get(3, 9), Some(Cell::Empty));
        assert_eq!(board.get(9, 9), Some(Cell::Empty));
    }

    // --- Step invariants ---

    #[test]
    fn step_conserves_cell_counts() {
        /// Counts each non-`Empty` variant as `(sand, water, stone)`.
        fn counts(board: &Board) -> (usize, usize, usize) {
            let mut out = (0, 0, 0);
            for slot in &board.grid {
                match slot.cell {
                    Cell::Sand => out.0 += 1,
                    Cell::Water => out.1 += 1,
                    Cell::Stone => out.2 += 1,
                    Cell::Empty => {}
                }
            }
            out
        }

        let mut board = Board::new(40, 40);
        let mut rng = SmallRng::seed_from_u64(12);
        for y in 0..40 {
            for x in 0..40 {
                let cell = match (x * 7 + y * 13) % 5 {
                    0 => Cell::Sand,
                    1 => Cell::Water,
                    2 => Cell::Stone,
                    _ => Cell::Empty,
                };
                board.set(x, y, cell);
            }
        }

        let before = counts(&board);
        assert!(before.0 > 0 && before.1 > 0 && before.2 > 0);

        for step in 1..=50 {
            board.step(&mut rng);
            assert_eq!(
                counts(&board),
                before,
                "material was created or destroyed on step {step}"
            );
        }
    }

    /// Pins that `move_cell` sets `FLAG_MOVED` on the *destination* slot, and
    /// that `can_move_into` then rejects that slot.
    ///
    /// Asserted directly rather than through falling behavior: `step` scans
    /// bottom-up, so a cell that moves down always lands in an already-scanned
    /// row and is never revisited, which means no amount of sand-falling can
    /// observe this flag. Ticket D1's `swap_cells` moves the displaced cell
    /// *up* into the row being scanned, where it becomes load-bearing.
    #[test]
    fn move_cell_flags_the_destination() {
        let mut board = Board::new(3, 10);
        let mut rng = SmallRng::seed_from_u64(16);
        board.set(1, 0, Cell::Sand);

        board.update_cell(1, 0, &mut rng);

        assert_eq!(board.get(1, 1), Some(Cell::Sand));
        let dst = board.idx(1, 1);
        assert_eq!(
            board.grid[dst].flags & FLAG_MOVED,
            FLAG_MOVED,
            "move_cell must flag the destination slot"
        );
        assert!(
            !board.can_move_into(1, 1),
            "a slot flagged FLAG_MOVED must be rejected as a destination"
        );
    }

    /// Pins that `FLAG_MOVED` stops a cell moving twice in one step.
    ///
    /// Sideways flow is the only motion that can expose this: it stays in the
    /// row being scanned, so on a fresh board (`scan_left_to_right == true`)
    /// water that flows *right* lands on a column the scan has not reached yet.
    /// Without the flag the scan picks it up again and it skates across the row
    /// in a single step.
    #[test]
    fn water_does_not_flow_twice_in_one_step() {
        let mut board = Board::new(30, 10);
        let mut rng = SmallRng::seed_from_u64(17);
        board.set(0, 9, Cell::Water);

        // Rightward is the only legal direction from x = 0, so the coin flip in
        // `try_flow_sideways` cannot affect this outcome.
        assert!(board.scan_left_to_right);
        board.step(&mut rng);

        let found: Vec<usize> = (0..board.width())
            .filter(|&x| board.get(x, 9) == Some(Cell::Water))
            .collect();
        assert_eq!(
            found,
            vec![5],
            "water should flow FLOW_DIST = 5 cells once, not repeatedly"
        );
    }

    /// Pins that `step` clears *every* flag bit at the end of the step, not
    /// just `FLAG_MOVED`.
    ///
    /// A spare bit is set by hand before stepping, because `FLAG_MOVED` is the
    /// only bit production code ever sets: asserting on `FLAG_MOVED` alone
    /// cannot distinguish `flags = 0` from `flags &= !FLAG_MOVED`.
    ///
    /// Ticket W1 narrows the clear to `flags &= !FLAG_MOVED` so that a momentum
    /// bit can persist across steps. That change *will* fail this test, which is
    /// the point — W1 must rewrite it to assert the momentum bit survives while
    /// `FLAG_MOVED` does not.
    #[test]
    fn step_clears_all_flags() {
        const SPARE_FLAG: u8 = 1 << 1;

        let mut board = Board::new(3, 10);
        let mut rng = SmallRng::seed_from_u64(13);
        board.set(1, 0, Cell::Sand);
        let spare_idx = board.idx(2, 5);
        board.grid[spare_idx].flags |= SPARE_FLAG;

        board.step(&mut rng);

        // The sand definitely moved, so `FLAG_MOVED` was definitely set too.
        assert_eq!(board.get(1, 1), Some(Cell::Sand));
        for (i, slot) in board.grid.iter().enumerate() {
            assert_eq!(slot.flags, 0, "flags left set at index {i}");
        }
    }

    #[test]
    fn step_alternates_scan_direction() {
        let mut board = Board::new(3, 3);
        let mut rng = SmallRng::seed_from_u64(14);

        assert!(board.scan_left_to_right);
        board.step(&mut rng);
        assert!(!board.scan_left_to_right);
        board.step(&mut rng);
        assert!(board.scan_left_to_right);
    }

    #[test]
    fn stone_never_moves() {
        let mut board = Board::new(5, 10);
        let mut rng = SmallRng::seed_from_u64(15);
        board.set(2, 0, Cell::Stone);

        for _ in 0..20 {
            board.step(&mut rng);
        }

        assert_eq!(board.get(2, 0), Some(Cell::Stone));
        // Counted board-wide so a `move_cell` that duplicates instead of moving
        // cannot pass by leaving the original in place.
        let stone = (0..board.width())
            .flat_map(|x| (0..board.height()).map(move |y| (x, y)))
            .filter(|&(x, y)| board.get(x, y) == Some(Cell::Stone))
            .count();
        assert_eq!(stone, 1, "stone was duplicated");
    }

    /// Pins that sand does *not* sink through water: `can_move_into` only
    /// accepts `Cell::Empty` destinations, so denser material cannot displace
    /// lighter material.
    ///
    /// This is a current limitation, not desired behavior. Ticket D1 (density
    /// and displacement) inverts it — after D1 this same setup should end with
    /// sand at `(1, 1)` and water at `(1, 0)` — and must replace this test with
    /// that mirror image rather than loosen the assertion.
    #[test]
    fn sand_does_not_yet_sink_through_water() {
        let mut board = Board::new(3, 3);
        let mut rng = SmallRng::seed_from_u64(16);
        // Floor and walls, so the water is boxed in and can neither fall nor flow.
        board.set(0, 2, Cell::Stone);
        board.set(1, 2, Cell::Stone);
        board.set(2, 2, Cell::Stone);
        board.set(0, 1, Cell::Stone);
        board.set(2, 1, Cell::Stone);
        board.set(1, 1, Cell::Water);
        board.set(1, 0, Cell::Sand);

        board.step(&mut rng);

        assert_eq!(board.get(1, 0), Some(Cell::Sand));
        assert_eq!(board.get(1, 1), Some(Cell::Water));
    }
}
