use crate::cell::{Cell, CellSlot};
use crate::rules;
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
        if self.try_fall(x, y, rng) {
            return;
        }
        self.try_slump(x, y);
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
    /// Each candidate is first tried as a plain move into an empty slot, then as
    /// a density displacement: a swap with a strictly less dense occupant. Both
    /// checks happen per candidate, so a cell prefers to sink straight down
    /// through lighter material over sliding into an empty diagonal.
    ///
    /// Returns true if the cell moved
    fn try_fall(&mut self, x: usize, y: usize, rng: &mut impl Rng) -> bool {
        let mover = self.grid[self.idx(x, y)].cell;
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
            } else if self.can_displace(mover, nx, ny) {
                let cur_idx = self.idx(x, y);
                let dst_idx = self.idx(nx, ny);
                self.swap_cells(cur_idx, dst_idx);
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

    /// Attempts to slump the supported cell at `(x, y)` one cell sideways: the
    /// lower angle of repose that submerged sand has and dry sand does not.
    ///
    /// Only called after `try_fall` has failed, so the cell is resting on
    /// something. Two conditions must both hold:
    ///
    /// * The destination `(x + dir, y)` must hold a strictly lighter,
    ///   **non-`Empty`** material — `can_displace`. That is the physical
    ///   trigger: buoyancy and lubrication come from being in contact with a
    ///   fluid, so the contact is what is tested, not whether the grain is
    ///   submerged. Because `can_displace` rejects `Cell::Empty`, this rule is
    ///   structurally incapable of firing in air, which leaves dry sand's ~45°
    ///   repose untouched.
    /// * The look-ahead `(x + 2 * dir, y + 1)` — one further along and one row
    ///   **down** — must be empty or lighter, i.e. `try_fall`'s diagonal
    ///   precondition evaluated at the destination.
    ///
    /// The look-ahead is what sets the angle and what prevents oscillation.
    /// Without it (or with a same-row look-ahead) the terminal state is a flat
    /// bed and two surface columns whose heights differ by one trade that
    /// difference back and forth forever. Requiring a *downward* continuation
    /// means every successful slump is followed by a descent, material only
    /// ever moves down-slope, and the stable surface is one where columns two
    /// apart differ by at most one row: 1 row per 2 columns, ~26.6°.
    ///
    /// Out-of-bounds coordinates fail both checks, so nothing slides off the
    /// edge of the board. Direction order comes from `self.scan_left_to_right`,
    /// which flips every step, rather than from the RNG: both directions
    /// qualify only for a grain that is a local peak, so the choice is a
    /// tie-break whose bias cancels step to step.
    ///
    /// Returns true if the cell moved.
    fn try_slump(&mut self, x: usize, y: usize) -> bool {
        // Read from the grid rather than hardcoding `Cell::Sand`: the density
        // comparisons below must use the mover's own density. Only sand is
        // routed here today, but a denser element added later and pointed at
        // this rule would otherwise be compared using sand's density and
        // silently refuse to slump.
        let mover = self.grid[self.idx(x, y)].cell;
        let dirs: [i32; 2] = if self.scan_left_to_right {
            [-1, 1]
        } else {
            [1, -1]
        };

        for dir in dirs {
            // `checked_sub` for leftward moves; the `>= width` guards in
            // `can_displace` / `can_move_into` cover the rightward side.
            let Some(nx) = offset_x(x, dir) else {
                continue;
            };
            if !self.can_displace(mover, nx, y) {
                continue;
            }
            let Some(lx) = offset_x(nx, dir) else {
                continue;
            };
            let ly = y + 1;
            // Note this predicate is `FLAG_MOVED`-sensitive, not purely
            // geometric: row `y + 1` is scanned before row `y`, so the target
            // may already be flagged by this step's motion. That only ever
            // makes the check stricter — it delays a slump by a step, it can
            // never permit an illegal one — and measurement shows the
            // anti-oscillation guarantee does not depend on it. It matters
            // when ticket S1 extracts a shared "legal descent" predicate:
            // `try_fall` needs the flag check because it is about to move,
            // this look-ahead only predicts, so which semantics the shared
            // helper takes must be a decision rather than an accident.
            if !(self.can_move_into(lx, ly) || self.can_displace(mover, lx, ly)) {
                continue;
            }
            self.swap_cells(self.idx(x, y), self.idx(nx, y));
            return true;
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

    /// Returns true if a cell of `mover` can sink into the occupied cell at
    /// `(nx, ny)` by swapping with it.
    ///
    /// Requires the destination to be in bounds, to hold a non-`Empty` cell that
    /// has not already moved this step, and to be strictly less dense than
    /// `mover`. The comparison is strictly greater so that same-density cells
    /// never churn against each other. `Empty` destinations are excluded on
    /// purpose: those are a plain move, handled by `can_move_into` / `move_cell`.
    fn can_displace(&self, mover: Cell, nx: usize, ny: usize) -> bool {
        if nx >= self.width || ny >= self.height {
            return false;
        }
        let slot = self.grid[self.idx(nx, ny)];
        slot.cell != Cell::Empty
            && slot.flags & FLAG_MOVED == 0
            && rules::density(mover) > rules::density(slot.cell)
    }

    fn move_cell(&mut self, from_idx: usize, to_idx: usize) {
        self.grid[to_idx].cell = self.grid[from_idx].cell;
        self.grid[to_idx].flags |= FLAG_MOVED;
        self.grid[from_idx].cell = Cell::Empty;
    }

    /// Exchanges the cells in two slots and marks **both** as moved.
    ///
    /// Both slots are flagged because a swap moves the displaced cell *upward*,
    /// into the slot the bottom-up scan is currently processing. Without the
    /// source flag, the denser cell on the row above finds that lighter cell
    /// unflagged and swaps with it again, so it climbs several rows in one step.
    ///
    /// This is why an `Empty` destination must stay on `move_cell`, which
    /// deliberately leaves its source unflagged so the cell above can fall into
    /// the slot just vacated.
    ///
    /// Only the `cell` fields are exchanged; the flags are not. That is
    /// deliberate but currently unobservable, since `FLAG_MOVED` is the only
    /// bit and both slots receive it either way. It stops being unobservable as
    /// soon as a flag describes the *particle* rather than the slot — ticket
    /// W1's momentum bit, which persists across steps. Such a bit must travel
    /// with the cell, or displaced water would inherit the momentum of the sand
    /// that pushed past it. Revisit this function when W1 lands.
    fn swap_cells(&mut self, from_idx: usize, to_idx: usize) {
        let from_cell = self.grid[from_idx].cell;
        self.grid[from_idx].cell = self.grid[to_idx].cell;
        self.grid[to_idx].cell = from_cell;
        self.grid[from_idx].flags |= FLAG_MOVED;
        self.grid[to_idx].flags |= FLAG_MOVED;
    }

    /// Calculates the flat index from two dimensional coordinates
    fn idx(&self, x: usize, y: usize) -> usize {
        y * self.width + x
    }
}

/// Offsets an unsigned column index by `dir`, which is expected to be `-1` or
/// `1`.
///
/// Returns `None` only on underflow at the left edge; the right edge is left to
/// the callers' `>= width` bounds checks, which is the asymmetry unsigned
/// coordinates force.
fn offset_x(x: usize, dir: i32) -> Option<usize> {
    if dir < 0 {
        x.checked_sub(dir.unsigned_abs() as usize)
    } else {
        Some(x + dir as usize)
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
    /// observe this flag. `swap_cells` is where it becomes load-bearing: a swap
    /// moves the displaced cell *up* into the row being scanned. That case is
    /// pinned by `displaced_water_does_not_move_twice_in_one_step`.
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

    /// Pins that sand sinks through water: `can_displace` accepts a strictly
    /// less dense occupant and `swap_cells` exchanges the two.
    #[test]
    fn sand_sinks_through_water() {
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

        assert_eq!(board.get(1, 1), Some(Cell::Sand));
        assert_eq!(board.get(1, 0), Some(Cell::Water));
    }

    /// Pins that `swap_cells` flags **both** slots, not just the destination.
    ///
    /// A swap pushes the displaced water *upward*, into the row the bottom-up
    /// scan is about to reach. Without the source flag the sand at `(1, 0)`
    /// swaps with that same water again on the same step, giving `(1, 0)` water
    /// and `(1, 1)` sand: the water would climb two rows in one step.
    #[test]
    fn displaced_water_does_not_move_twice_in_one_step() {
        let mut board = Board::new(3, 4);
        let mut rng = SmallRng::seed_from_u64(18);
        // Stone floor plus full side walls, so no diagonal and no sideways flow
        // is ever legal and the coin flips cannot affect the outcome.
        for x in 0..3 {
            board.set(x, 3, Cell::Stone);
        }
        for y in 0..3 {
            board.set(0, y, Cell::Stone);
            board.set(2, y, Cell::Stone);
        }
        board.set(1, 2, Cell::Water);
        board.set(1, 1, Cell::Sand);
        board.set(1, 0, Cell::Sand);

        board.step(&mut rng);

        assert_eq!(board.get(1, 0), Some(Cell::Sand));
        assert_eq!(
            board.get(1, 1),
            Some(Cell::Water),
            "the displaced water must rise exactly one row per step"
        );
        assert_eq!(board.get(1, 2), Some(Cell::Sand));
    }

    /// Pins that displacement applies to the downward diagonals too, not just
    /// straight down.
    #[test]
    fn sand_displaces_water_diagonally() {
        let mut board = Board::new(3, 3);
        let mut rng = SmallRng::seed_from_u64(19);
        for x in 0..3 {
            board.set(x, 2, Cell::Stone);
        }
        // Straight down and down-left are stone, so down-right is the only legal
        // candidate whichever way the coin lands.
        board.set(1, 1, Cell::Stone);
        board.set(0, 1, Cell::Stone);
        board.set(2, 1, Cell::Water);
        board.set(1, 0, Cell::Sand);

        board.step(&mut rng);

        assert_eq!(board.get(2, 1), Some(Cell::Sand));
        assert_eq!(board.get(1, 0), Some(Cell::Water));
    }

    /// Pins the `u8::MAX` density sentinel on stone: nothing can be strictly
    /// denser, so stone is never displaced.
    #[test]
    fn sand_does_not_displace_stone() {
        let mut board = Board::new(3, 3);
        let mut rng = SmallRng::seed_from_u64(20);
        // Boxed in: every downward candidate is stone.
        board.set(0, 1, Cell::Stone);
        board.set(1, 1, Cell::Stone);
        board.set(2, 1, Cell::Stone);
        board.set(1, 0, Cell::Sand);

        board.step(&mut rng);

        assert_eq!(board.get(1, 0), Some(Cell::Sand));
        assert_eq!(board.get(1, 1), Some(Cell::Stone));
    }

    /// Pins the *direction* of the density comparison: lighter material never
    /// sinks through heavier material.
    #[test]
    fn water_does_not_displace_sand() {
        let mut board = Board::new(3, 3);
        let mut rng = SmallRng::seed_from_u64(21);
        for x in 0..3 {
            board.set(x, 2, Cell::Stone);
        }
        board.set(0, 1, Cell::Stone);
        board.set(2, 1, Cell::Stone);
        board.set(1, 1, Cell::Sand);
        // Walls beside the water too, so it cannot flow sideways either.
        board.set(0, 0, Cell::Stone);
        board.set(2, 0, Cell::Stone);
        board.set(1, 0, Cell::Water);

        board.step(&mut rng);

        assert_eq!(board.get(1, 0), Some(Cell::Water));
        assert_eq!(board.get(1, 1), Some(Cell::Sand));
    }

    /// Pins that the density comparison is strictly `>`: equal densities do not
    /// swap.
    ///
    /// A sand/sand swap is invisible in the cell values, so `FLAG_MOVED` is the
    /// only observable. That is why this calls `update_cell` directly instead of
    /// `step`, which clears every flag before returning.
    #[test]
    fn same_density_cells_do_not_swap() {
        let mut board = Board::new(3, 3);
        let mut rng = SmallRng::seed_from_u64(22);
        // Boxed in, so the only candidate that is not stone is the sand below.
        board.set(0, 1, Cell::Stone);
        board.set(2, 1, Cell::Stone);
        board.set(1, 1, Cell::Sand);
        board.set(1, 0, Cell::Sand);

        board.update_cell(1, 0, &mut rng);

        let upper = board.idx(1, 0);
        let lower = board.idx(1, 1);
        assert_eq!(board.get(1, 0), Some(Cell::Sand));
        assert_eq!(board.get(1, 1), Some(Cell::Sand));
        assert_eq!(
            board.grid[upper].flags & FLAG_MOVED,
            0,
            "sand must not swap with equally dense sand"
        );
        assert_eq!(
            board.grid[lower].flags & FLAG_MOVED,
            0,
            "sand must not swap with equally dense sand"
        );
    }

    /// The ticket's headline behavior end to end: sand poured on water ends up
    /// underneath it, and the settled stack is then static.
    #[test]
    fn sand_settles_below_water_over_many_steps() {
        let mut board = Board::new(3, 10);
        let mut rng = SmallRng::seed_from_u64(23);
        // Full side walls and a floor: the interior is a one-cell-wide column
        // with no lateral freedom at all, so the coin flips cannot matter.
        for y in 0..10 {
            board.set(0, y, Cell::Stone);
            board.set(2, y, Cell::Stone);
        }
        for x in 0..3 {
            board.set(x, 9, Cell::Stone);
        }
        board.set(1, 6, Cell::Water);
        board.set(1, 7, Cell::Water);
        board.set(1, 8, Cell::Water);
        board.set(1, 4, Cell::Sand);
        board.set(1, 5, Cell::Sand);

        for _ in 0..20 {
            board.step(&mut rng);
        }

        let column: Vec<Option<Cell>> = (0..10).map(|y| board.get(1, y)).collect();
        assert_eq!(
            column,
            vec![
                Some(Cell::Empty),
                Some(Cell::Empty),
                Some(Cell::Empty),
                Some(Cell::Empty),
                Some(Cell::Water),
                Some(Cell::Water),
                Some(Cell::Water),
                Some(Cell::Sand),
                Some(Cell::Sand),
                Some(Cell::Stone),
            ]
        );

        // A settled stack must be static, not churning: water above sand can
        // never displace it.
        for _ in 0..20 {
            board.step(&mut rng);
        }
        let settled: Vec<Option<Cell>> = (0..10).map(|y| board.get(1, y)).collect();
        assert_eq!(settled, column, "a settled stack must not keep moving");
    }

    // --- Sand: `try_slump`, the submerged angle of repose ---

    /// The shared 6x5 fixture for the `try_slump` scenarios.
    ///
    /// ```text
    ///   y=0  . . . . . .
    ///   y=1  . . . . . .
    ///   y=2  . # o D # .     D = the slide destination, `dest`   (3,2)
    ///   y=3  . # # # L #     L = the look-ahead cell, `look`     (4,3)
    ///   y=4  # # # # # #
    /// ```
    ///
    /// The grain at `(2,2)` has stone straight down and on both downward
    /// diagonals, so `try_fall` fails whichever way its coin lands. Stone at
    /// `(1,2)` blocks the leftward slump outright, so only one direction is ever
    /// legal and the scan direction cannot change the outcome either. Stone at
    /// `(4,2)` pens `D` in so it cannot flow away on its own.
    fn slump_fixture(dest: Cell, look: Cell) -> Board {
        let mut board = Board::new(6, 5);
        for x in 0..6 {
            board.set(x, 4, Cell::Stone);
        }
        for x in [1, 2, 3, 5] {
            board.set(x, 3, Cell::Stone);
        }
        board.set(1, 2, Cell::Stone);
        board.set(4, 2, Cell::Stone);
        board.set(2, 2, Cell::Sand);
        board.set(3, 2, dest);
        board.set(4, 3, look);
        board
    }

    /// A: the headline rule. Supported sand in lateral contact with water
    /// slides one cell toward it, but only because it could then descend from
    /// there.
    ///
    /// The `(4,3)` assertion is the one that catches a missing destination flag:
    /// unflagged, the grain that lands on `(3,2)` is picked up again by the same
    /// left-to-right scan and falls on into `(4,3)` within a single step.
    #[test]
    fn submerged_sand_slumps_sideways_when_it_can_descend_beyond() {
        let mut board = slump_fixture(Cell::Water, Cell::Water);
        let mut rng = SmallRng::seed_from_u64(24);

        board.step(&mut rng);

        assert_eq!(board.get(3, 2), Some(Cell::Sand), "the grain should slump");
        assert_eq!(
            board.get(2, 2),
            Some(Cell::Water),
            "the displaced water takes the grain's old slot"
        );
        assert_eq!(
            board.get(4, 3),
            Some(Cell::Water),
            "a slump is one cell per step: the grain must not also descend"
        );
    }

    /// B: a slump is never a dead end. The look-ahead is exactly `try_fall`'s
    /// diagonal precondition at the destination, so the descent follows.
    #[test]
    fn slumped_sand_descends_on_the_next_step() {
        let mut board = slump_fixture(Cell::Water, Cell::Water);
        let mut rng = SmallRng::seed_from_u64(25);

        board.step(&mut rng);
        board.step(&mut rng);

        assert_eq!(board.get(4, 3), Some(Cell::Sand));
        assert_eq!(board.get(3, 2), Some(Cell::Water));
    }

    /// C: the look-ahead test. This is what pins the ~26.6° angle and the
    /// no-oscillation property — without the look-ahead the grain slides into
    /// any adjacent water and the terminal slope is flat.
    #[test]
    fn sand_does_not_slump_without_room_to_descend() {
        let mut board = slump_fixture(Cell::Water, Cell::Stone);
        let mut rng = SmallRng::seed_from_u64(26);

        board.step(&mut rng);

        assert_eq!(
            board.get(2, 2),
            Some(Cell::Sand),
            "nothing should have moved"
        );
        assert_eq!(board.get(3, 2), Some(Cell::Water));
    }

    /// D: the dry-sand guard, stated as a unit. `can_displace` rejects
    /// `Cell::Empty`, so the rule cannot fire in air at all.
    #[test]
    fn dry_supported_sand_does_not_slump_into_air() {
        let mut board = slump_fixture(Cell::Empty, Cell::Empty);
        let mut rng = SmallRng::seed_from_u64(27);

        board.step(&mut rng);

        assert_eq!(board.get(2, 2), Some(Cell::Sand));
        assert_eq!(board.get(3, 2), Some(Cell::Empty));
    }

    /// E: sand never slumps into sand — the density comparison stays strict.
    ///
    /// A sand-into-sand swap leaves both cell *values* unchanged, so position
    /// alone cannot see it and `FLAG_MOVED` is the only observable. That is why
    /// this drives `update_cell` directly instead of `step`, which clears every
    /// flag before returning: asserting on positions after a `step` still
    /// passes when `can_displace` is relaxed to `>=`.
    #[test]
    fn sand_does_not_slump_into_sand() {
        let mut board = slump_fixture(Cell::Sand, Cell::Water);
        let mut rng = SmallRng::seed_from_u64(28);

        board.update_cell(2, 2, &mut rng);

        assert_eq!(
            board.get(2, 2),
            Some(Cell::Sand),
            "the grain must not slump"
        );
        assert_eq!(board.get(3, 2), Some(Cell::Sand));
        let grain = board.idx(2, 2);
        let dest = board.idx(3, 2);
        assert_eq!(
            board.grid[grain].flags & FLAG_MOVED,
            0,
            "sand must not slump into equally dense sand"
        );
        assert_eq!(
            board.grid[dest].flags & FLAG_MOVED,
            0,
            "sand must not slump into equally dense sand"
        );
    }

    /// F: the source flag on `swap_cells`. Row `y-1` is scanned after row `y`,
    /// so a grain sitting directly above the displaced water would otherwise
    /// swap down into it and the water would travel sideways *and* up in one
    /// step, ending at `(2,1)`.
    #[test]
    fn displaced_water_does_not_move_twice_when_sand_slumps() {
        let mut board = slump_fixture(Cell::Water, Cell::Water);
        let mut rng = SmallRng::seed_from_u64(29);
        board.set(2, 1, Cell::Sand);

        board.step(&mut rng);

        assert_eq!(board.get(3, 2), Some(Cell::Sand), "the lower grain slumps");
        assert_eq!(
            board.get(2, 2),
            Some(Cell::Water),
            "the displaced water must stay in the slot it was pushed into"
        );
        assert_eq!(
            board.get(2, 1),
            Some(Cell::Sand),
            "the grain above must not swap with water that already moved"
        );
    }

    /// The G/H staircase fixture, 10 wide and 8 tall. `fluid` fills every free
    /// cell: `Cell::Empty` for the dry case, `Cell::Water` for the submerged one.
    ///
    /// ```text
    ///   y=0  # # ~ ~ ~ ~ ~ ~ ~ #
    ///   y=1  # # ~ ~ ~ ~ ~ ~ ~ #
    ///   y=2  # # ~ ~ ~ ~ ~ ~ ~ #
    ///   y=3  # # ~ ~ ~ ~ ~ ~ ~ #
    ///   y=4  # # o ~ ~ ~ ~ ~ ~ #
    ///   y=5  # # o o ~ ~ ~ ~ ~ #
    ///   y=6  # # o o o ~ ~ ~ ~ #
    ///   y=7  # # # # # # # # # #
    /// ```
    ///
    /// The `x = 1` wall is load-bearing: without it the staircase's left face is
    /// a vertical drop and even the dry pile collapses, which would make the dry
    /// test a test of the wrong thing.
    fn staircase_fixture(fluid: Cell) -> Board {
        let mut board = Board::new(10, 8);
        for y in 0..8 {
            for x in 2..9 {
                board.set(x, y, fluid);
            }
            board.set(0, y, Cell::Stone);
            board.set(1, y, Cell::Stone);
            board.set(9, y, Cell::Stone);
        }
        for x in 0..10 {
            board.set(x, 7, Cell::Stone);
        }
        for (x, y) in [(2, 4), (2, 5), (2, 6), (3, 5), (3, 6), (4, 6)] {
            board.set(x, y, Cell::Sand);
        }
        board
    }

    /// Collects every sand cell as `(x, y)` in reading order: rows top to
    /// bottom, and left to right within a row.
    fn sand_cells(board: &Board) -> Vec<(usize, usize)> {
        (0..board.height())
            .flat_map(|y| (0..board.width()).map(move |x| (x, y)))
            .filter(|&(x, y)| board.get(x, y) == Some(Cell::Sand))
            .collect()
    }

    /// G: the direct regression guard on the user's dry 45° pile. A dry
    /// staircase whose columns differ by one is already at its angle of repose
    /// and must not budge.
    #[test]
    fn a_dry_forty_five_degree_slope_is_stable() {
        let mut board = staircase_fixture(Cell::Empty);
        let mut rng = SmallRng::seed_from_u64(30);
        let before = sand_cells(&board);

        for _ in 0..50 {
            board.step(&mut rng);
        }

        assert_eq!(
            sand_cells(&board),
            before,
            "dry sand at 45° must not slump: the rule cannot fire in air"
        );
    }

    /// I: the anti-oscillation guarantee, asserted **per step** rather than on
    /// the settled state.
    ///
    /// A submerged bed whose column heights are `2 2 2 1 1 1` is already at the
    /// 1-row-per-2-columns angle, so no grain may move at all. The box is
    /// completely full — every free cell is water, so no cell is `Empty` — which
    /// means the water cannot move either and the *whole board* must be
    /// byte-stable on every single step.
    ///
    /// This is the shape of failure a settled-state test cannot see: drop the
    /// downward look-ahead and the one-cell surface step trades back and forth
    /// forever, which a 50-step-then-compare assertion can pass by landing on an
    /// even beat.
    #[test]
    fn a_submerged_bed_at_the_slump_angle_never_moves_on_any_step() {
        let mut board = Board::new(8, 6);
        let mut rng = SmallRng::seed_from_u64(32);
        for y in 0..6 {
            for x in 1..7 {
                board.set(x, y, Cell::Water);
            }
            board.set(0, y, Cell::Stone);
            board.set(7, y, Cell::Stone);
        }
        for x in 0..8 {
            board.set(x, 5, Cell::Stone);
        }
        // Column heights 2 2 2 1 1 1: a single one-cell surface step, which is
        // legal at 26.6° and would oscillate under a rule with no look-ahead.
        for x in 1..7 {
            board.set(x, 4, Cell::Sand);
        }
        for x in 1..4 {
            board.set(x, 3, Cell::Sand);
        }

        let before: Vec<Cell> = board.grid.iter().map(|slot| slot.cell).collect();
        for step in 1..=50 {
            board.step(&mut rng);
            let now: Vec<Cell> = board.grid.iter().map(|slot| slot.cell).collect();
            assert_eq!(
                now, before,
                "the board changed on step {step}: a bed at the slump angle must be static"
            );
        }
    }

    /// H: the same sand geometry, the only difference being water, must settle
    /// differently — three rows of 45° staircase relax into two rows at 1 row
    /// per 2 columns.
    ///
    /// G and H together are the whole ticket.
    #[test]
    fn a_submerged_forty_five_degree_slope_flattens() {
        let mut board = staircase_fixture(Cell::Water);
        let mut rng = SmallRng::seed_from_u64(31);

        for _ in 0..100 {
            board.step(&mut rng);
        }

        assert_eq!(
            sand_cells(&board),
            vec![(2, 5), (3, 5), (2, 6), (3, 6), (4, 6), (5, 6)],
            "the submerged slope should relax to 1 row per 2 columns"
        );
    }
}
