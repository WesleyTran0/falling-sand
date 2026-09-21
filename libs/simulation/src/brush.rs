use crate::{Board, Cell};
use rand::{Rng, RngExt};

/// The shape of a brush's scatter area.
#[derive(Clone, Copy, PartialEq, Debug)]
enum Shape {
    Circle { radius: i32 },
    Rectangle { half_width: i32, half_height: i32 },
}

/// Returns the `(shape, density)` scatter tuning for `cell`.
///
/// `density` is the probability (0.0-1.0) that a non-center cell within
/// `shape` is placed; the center cell is always placed regardless of density.
fn brush_params(cell: Cell) -> (Shape, f64) {
    match cell {
        Cell::Sand => (Shape::Circle { radius: 3 }, 0.45),
        Cell::Water => (
            Shape::Rectangle {
                half_width: 2,
                half_height: 3,
            },
            0.45,
        ),
        Cell::Stone => (Shape::Circle { radius: 2 }, 0.85),
        Cell::Empty => (Shape::Circle { radius: 3 }, 1.0),
    }
}

/// Scatters `cell` within `shape` around `(cx, cy)`. The center cell is
/// always placed; every other cell within `shape` is placed with
/// probability `density`. Returns the number of cells actually painted.
fn scatter_paint(
    board: &mut Board,
    cx: usize,
    cy: usize,
    cell: Cell,
    shape: Shape,
    density: f64,
    rng: &mut impl Rng,
) -> usize {
    let (half_width, half_height) = match shape {
        Shape::Circle { radius } => (radius, radius),
        Shape::Rectangle {
            half_width,
            half_height,
        } => (half_width, half_height),
    };
    let mut painted = 0;
    for dy in -half_height..=half_height {
        for dx in -half_width..=half_width {
            let in_shape = match shape {
                Shape::Circle { radius } => dx * dx + dy * dy <= radius * radius,
                Shape::Rectangle { .. } => true,
            };
            if !in_shape {
                continue;
            }
            let place = (dx == 0 && dy == 0) || rng.random_bool(density);
            if !place {
                continue;
            }
            let x = cx as i32 + dx;
            let y = cy as i32 + dy;
            if x < 0 || y < 0 {
                continue;
            }
            if board.set(x as usize, y as usize, cell) {
                painted += 1;
            }
        }
    }
    painted
}

/// The brush that draws elements onto the board is represented here
pub struct Brush;

impl Default for Brush {
    fn default() -> Self {
        Self::new()
    }
}

impl Brush {
    /// Creates a brush. The brush itself is stateless; all tuning is looked
    /// up per element in `brush_params`.
    pub fn new() -> Self {
        Self
    }

    /// Paints `cell` onto the `board` with `(cx, cy)` as the center point.
    ///
    /// The scatter shape and density are looked up per `cell` via `brush_params`.
    /// Returns the number of cells actually painted.
    pub fn paint(
        &self,
        board: &mut Board,
        cx: usize,
        cy: usize,
        cell: Cell,
        rng: &mut impl Rng,
    ) -> usize {
        let (shape, density) = brush_params(cell);
        scatter_paint(board, cx, cy, cell, shape, density, rng)
    }

    /// Paints `cell` along the straight line from `(x0, y0)` to `(x1, y1)`,
    /// stamping the brush at every cell the line passes through.
    ///
    /// The cursor is sampled once per frame, so a fast drag moves many cells
    /// between consecutive samples. Stamping only at the sample points leaves
    /// visible gaps in the stroke; walking the line between them does not, at
    /// any cursor speed. Both endpoints are stamped, so `from == to` stamps
    /// exactly once and a stationary cursor behaves like `paint`.
    ///
    /// Cells outside the board are dropped by `Board::set`'s bounds check.
    /// Returns the number of cells actually painted.
    pub fn paint_line(
        &self,
        board: &mut Board,
        from: (usize, usize),
        to: (usize, usize),
        cell: Cell,
        rng: &mut impl Rng,
    ) -> usize {
        let ((x0, y0), (x1, y1)) = (from, to);
        // Bresenham over i64: the coordinates are `usize`, but the error term
        // and the step deltas are signed.
        let (mut x, mut y) = (x0 as i64, y0 as i64);
        let (tx, ty) = (x1 as i64, y1 as i64);
        let dx = (tx - x).abs();
        let dy = -(ty - y).abs();
        let sx = if x < tx { 1 } else { -1 };
        let sy = if y < ty { 1 } else { -1 };
        let mut err = dx + dy;

        let mut painted = 0;
        loop {
            painted += self.paint(board, x as usize, y as usize, cell, rng);
            if x == tx && y == ty {
                break;
            }
            let e2 = 2 * err;
            if e2 >= dy {
                err += dy;
                x += sx;
            }
            if e2 <= dx {
                err += dx;
                y += sy;
            }
        }
        painted
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn brush() -> Brush {
        Brush::new()
    }
    use rand::SeedableRng;
    use rand::rngs::SmallRng;

    #[test]
    fn brush_params_matches_tuning_table() {
        assert_eq!(
            brush_params(Cell::Sand),
            (Shape::Circle { radius: 3 }, 0.45)
        );
        assert_eq!(
            brush_params(Cell::Water),
            (
                Shape::Rectangle {
                    half_width: 2,
                    half_height: 3
                },
                0.45
            )
        );
        assert_eq!(
            brush_params(Cell::Stone),
            (Shape::Circle { radius: 2 }, 0.85)
        );
        assert_eq!(
            brush_params(Cell::Empty),
            (Shape::Circle { radius: 3 }, 1.0)
        );
    }

    #[test]
    fn scatter_paint_always_places_center_even_at_density_zero() {
        let mut board = Board::new(20, 20);
        let mut rng = SmallRng::seed_from_u64(0xbeef_5eed);
        let painted = scatter_paint(
            &mut board,
            10,
            10,
            Cell::Sand,
            Shape::Circle { radius: 3 },
            0.0,
            &mut rng,
        );
        assert_eq!(painted, 1);
        assert_eq!(board.get(10, 10), Some(Cell::Sand));
    }

    #[test]
    fn scatter_paint_density_zero_places_only_center() {
        let mut board = Board::new(20, 20);
        let mut rng = SmallRng::seed_from_u64(0xbeef_5eed);
        scatter_paint(
            &mut board,
            10,
            10,
            Cell::Water,
            Shape::Circle { radius: 3 },
            0.0,
            &mut rng,
        );
        for dy in -3..=3i32 {
            for dx in -3..=3i32 {
                if (dx, dy) == (0, 0) {
                    continue;
                }
                if dx * dx + dy * dy > 9 {
                    continue;
                }
                let x = (10 + dx) as usize;
                let y = (10 + dy) as usize;
                assert_eq!(
                    board.get(x, y),
                    Some(Cell::Empty),
                    "cell at ({x}, {y}) should not have been painted at density 0.0"
                );
            }
        }
    }

    #[test]
    fn scatter_paint_density_one_fills_entire_circle() {
        let mut board = Board::new(20, 20);
        let mut rng = SmallRng::seed_from_u64(0xbeef_5eed);
        let painted = scatter_paint(
            &mut board,
            10,
            10,
            Cell::Stone,
            Shape::Circle { radius: 3 },
            1.0,
            &mut rng,
        );
        assert_eq!(painted, 29, "radius-3 circle should contain 29 cells");
        for dy in -3..=3i32 {
            for dx in -3..=3i32 {
                if dx * dx + dy * dy > 9 {
                    continue;
                }
                let x = (10 + dx) as usize;
                let y = (10 + dy) as usize;
                assert_eq!(board.get(x, y), Some(Cell::Stone));
            }
        }
    }

    #[test]
    fn paint_uses_brush_params_for_cell() {
        let mut board = Board::new(20, 20);
        let mut rng = SmallRng::seed_from_u64(0xbeef_5eed);
        let brush = Brush::new();
        let painted = brush.paint(&mut board, 10, 10, Cell::Empty, &mut rng);
        assert_eq!(
            painted, 29,
            "Cell::Empty has density 1.0 and radius 3, so paint() should fill the whole circle"
        );
    }

    #[test]
    fn scatter_paint_rectangle_density_one_fills_entire_box_including_corners() {
        let mut board = Board::new(20, 20);
        let mut rng = SmallRng::seed_from_u64(0xbeef_5eed);
        let shape = Shape::Rectangle {
            half_width: 1,
            half_height: 3,
        };
        let painted = scatter_paint(&mut board, 10, 10, Cell::Water, shape, 1.0, &mut rng);
        assert_eq!(painted, 21, "3x7 rectangle should contain 21 cells");
        for dy in -3..=3i32 {
            for dx in -1..=1i32 {
                let x = (10 + dx) as usize;
                let y = (10 + dy) as usize;
                assert_eq!(
                    board.get(x, y),
                    Some(Cell::Water),
                    "cell at ({x}, {y}) should have been painted — rectangle includes corners a circle would exclude"
                );
            }
        }
    }

    #[test]
    fn scatter_paint_rectangle_density_zero_places_only_center() {
        let mut board = Board::new(20, 20);
        let mut rng = SmallRng::seed_from_u64(0xbeef_5eed);
        let shape = Shape::Rectangle {
            half_width: 1,
            half_height: 3,
        };
        scatter_paint(&mut board, 10, 10, Cell::Water, shape, 0.0, &mut rng);
        for dy in -3..=3i32 {
            for dx in -1..=1i32 {
                if (dx, dy) == (0, 0) {
                    continue;
                }
                let x = (10 + dx) as usize;
                let y = (10 + dy) as usize;
                assert_eq!(
                    board.get(x, y),
                    Some(Cell::Empty),
                    "cell at ({x}, {y}) should not have been painted at density 0.0"
                );
            }
        }
        assert_eq!(board.get(10, 10), Some(Cell::Water));
    }

    #[test]
    fn paint_water_stays_within_rectangle_bounding_box() {
        let mut board = Board::new(20, 20);
        let mut rng = SmallRng::seed_from_u64(0xbeef_5eed);
        let brush = Brush::new();
        brush.paint(&mut board, 10, 10, Cell::Water, &mut rng);
        for y in 0..20 {
            for x in 0..20 {
                let dx = x as i32 - 10;
                let dy = y as i32 - 10;
                let within_box = dx.abs() <= 2 && dy.abs() <= 3;
                if !within_box {
                    assert_eq!(
                        board.get(x, y),
                        Some(Cell::Empty),
                        "cell at ({x}, {y}) is outside water's 5x7 bounding box but was painted"
                    );
                }
            }
        }
        assert_eq!(
            board.get(10, 10),
            Some(Cell::Water),
            "center should always be painted"
        );
    }

    /// Walks the same Bresenham line `paint_line` does, so a test can assert
    /// on every cell the stroke should have covered.
    fn line_cells(x0: i64, y0: i64, x1: i64, y1: i64) -> Vec<(usize, usize)> {
        let (mut x, mut y) = (x0, y0);
        let dx = (x1 - x).abs();
        let dy = -(y1 - y).abs();
        let sx = if x < x1 { 1 } else { -1 };
        let sy = if y < y1 { 1 } else { -1 };
        let mut err = dx + dy;
        let mut out = Vec::new();
        loop {
            out.push((x as usize, y as usize));
            if x == x1 && y == y1 {
                break;
            }
            let e2 = 2 * err;
            if e2 >= dy {
                err += dy;
                x += sx;
            }
            if e2 <= dx {
                err += dx;
                y += sy;
            }
        }
        out
    }

    #[test]
    fn paint_line_leaves_no_gap_between_distant_points() {
        // The brush centre is always placed regardless of density, so every
        // cell on the line must be set even though the scatter is random.
        let mut board = Board::new(60, 60);
        let mut rng = SmallRng::seed_from_u64(0xd00d);
        brush().paint_line(&mut board, (5, 5), (50, 30), Cell::Stone, &mut rng);
        for (x, y) in line_cells(5, 5, 50, 30) {
            assert_eq!(
                board.get(x, y),
                Some(Cell::Stone),
                "gap in the stroke at ({x}, {y})"
            );
        }
    }

    #[test]
    fn paint_line_with_identical_endpoints_matches_a_single_paint() {
        let mut a = Board::new(20, 20);
        let mut b = Board::new(20, 20);
        let painted_line = brush().paint_line(
            &mut a,
            (10, 10),
            (10, 10),
            Cell::Sand,
            &mut SmallRng::seed_from_u64(4),
        );
        let painted_dab =
            brush().paint(&mut b, 10, 10, Cell::Sand, &mut SmallRng::seed_from_u64(4));
        assert_eq!(
            painted_line, painted_dab,
            "a zero-length stroke stamped twice"
        );
        for y in 0..20 {
            for x in 0..20 {
                assert_eq!(a.get(x, y), b.get(x, y), "differs at ({x}, {y})");
            }
        }
    }

    #[test]
    fn paint_line_covers_the_same_cells_in_either_direction() {
        let mut fwd = Board::new(40, 40);
        let mut rev = Board::new(40, 40);
        brush().paint_line(
            &mut fwd,
            (3, 7),
            (30, 21),
            Cell::Stone,
            &mut SmallRng::seed_from_u64(9),
        );
        brush().paint_line(
            &mut rev,
            (30, 21),
            (3, 7),
            Cell::Stone,
            &mut SmallRng::seed_from_u64(9),
        );
        // Scatter differs by direction, but every line cell is a brush centre
        // and so must be set in both.
        for (x, y) in line_cells(3, 7, 30, 21) {
            assert_eq!(
                fwd.get(x, y),
                Some(Cell::Stone),
                "forward missed ({x}, {y})"
            );
            assert_eq!(
                rev.get(x, y),
                Some(Cell::Stone),
                "reverse missed ({x}, {y})"
            );
        }
    }

    #[test]
    fn paint_line_handles_pure_horizontal_vertical_and_diagonal() {
        for (x1, y1) in [(40usize, 20usize), (20, 40), (40, 40)] {
            let mut board = Board::new(60, 60);
            let mut rng = SmallRng::seed_from_u64(11);
            brush().paint_line(&mut board, (20, 20), (x1, y1), Cell::Stone, &mut rng);
            for (x, y) in line_cells(20, 20, x1 as i64, y1 as i64) {
                assert_eq!(board.get(x, y), Some(Cell::Stone), "missed ({x}, {y})");
            }
        }
    }

    #[test]
    fn paint_line_off_the_board_does_not_panic() {
        let mut board = Board::new(10, 10);
        let mut rng = SmallRng::seed_from_u64(12);
        // Endpoints far outside the grid: every stamp is dropped by bounds
        // checks, and the walk still terminates.
        brush().paint_line(&mut board, (5, 5), (400, 400), Cell::Sand, &mut rng);
        brush().paint_line(&mut board, (900, 900), (901, 901), Cell::Sand, &mut rng);
        assert_eq!(board.get(5, 5), Some(Cell::Sand));
    }
}
