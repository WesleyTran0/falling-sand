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

impl Brush {
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
}

#[cfg(test)]
mod tests {
    use super::*;
    use rand::SeedableRng;
    use rand::rngs::SmallRng;

    #[test]
    fn brush_params_matches_tuning_table() {
        assert_eq!(brush_params(Cell::Sand), (Shape::Circle { radius: 3 }, 0.45));
        assert_eq!(
            brush_params(Cell::Water),
            (Shape::Rectangle { half_width: 1, half_height: 3 }, 0.45)
        );
        assert_eq!(brush_params(Cell::Stone), (Shape::Circle { radius: 2 }, 0.85));
        assert_eq!(brush_params(Cell::Empty), (Shape::Circle { radius: 3 }, 1.0));
    }

    #[test]
    fn scatter_paint_always_places_center_even_at_density_zero() {
        let mut board = Board::new(20, 20);
        let mut rng = SmallRng::seed_from_u64(0xbeef_5eed);
        let painted = scatter_paint(
            &mut board, 10, 10, Cell::Sand, Shape::Circle { radius: 3 }, 0.0, &mut rng,
        );
        assert_eq!(painted, 1);
        assert_eq!(board.get(10, 10), Some(Cell::Sand));
    }

    #[test]
    fn scatter_paint_density_zero_places_only_center() {
        let mut board = Board::new(20, 20);
        let mut rng = SmallRng::seed_from_u64(0xbeef_5eed);
        scatter_paint(
            &mut board, 10, 10, Cell::Water, Shape::Circle { radius: 3 }, 0.0, &mut rng,
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
            &mut board, 10, 10, Cell::Stone, Shape::Circle { radius: 3 }, 1.0, &mut rng,
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
        let shape = Shape::Rectangle { half_width: 1, half_height: 3 };
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
        let shape = Shape::Rectangle { half_width: 1, half_height: 3 };
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
                let within_box = dx.abs() <= 1 && dy.abs() <= 3;
                if !within_box {
                    assert_eq!(
                        board.get(x, y),
                        Some(Cell::Empty),
                        "cell at ({x}, {y}) is outside water's 3x7 bounding box but was painted"
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
}
