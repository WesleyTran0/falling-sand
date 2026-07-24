use crate::{Board, Cell};
use rand::{Rng, RngExt};

/// Returns the `(radius, density)` scatter tuning for `cell`.
///
/// `radius` is the circular scatter radius in cells. `density` is the
/// probability (0.0-1.0) that a non-center cell within the radius is placed;
/// the center cell is always placed regardless of density.
fn brush_params(cell: Cell) -> (i32, f64) {
    match cell {
        Cell::Sand => (3, 0.45),
        Cell::Water => (3, 0.45),
        Cell::Stone => (2, 0.85),
        Cell::Empty => (3, 1.0),
    }
}

/// Scatters `cell` within a circular `radius` around `(cx, cy)`. The center
/// cell is always placed; every other cell within the radius is placed with
/// probability `density`. Returns the number of cells actually painted.
fn scatter_paint(
    board: &mut Board,
    cx: usize,
    cy: usize,
    cell: Cell,
    radius: i32,
    density: f64,
    rng: &mut impl Rng,
) -> usize {
    let mut painted = 0;
    for dy in -radius..=radius {
        for dx in -radius..=radius {
            if dx * dx + dy * dy > radius * radius {
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
    /// The scatter radius and density are looked up per `cell` via `brush_params`.
    /// Returns the number of cells actually painted.
    pub fn paint(
        &self,
        board: &mut Board,
        cx: usize,
        cy: usize,
        cell: Cell,
        rng: &mut impl Rng,
    ) -> usize {
        let (radius, density) = brush_params(cell);
        scatter_paint(board, cx, cy, cell, radius, density, rng)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rand::SeedableRng;
    use rand::rngs::SmallRng;

    #[test]
    fn brush_params_matches_tuning_table() {
        assert_eq!(brush_params(Cell::Sand), (3, 0.45));
        assert_eq!(brush_params(Cell::Water), (3, 0.45));
        assert_eq!(brush_params(Cell::Stone), (2, 0.85));
        assert_eq!(brush_params(Cell::Empty), (3, 1.0));
    }

    #[test]
    fn scatter_paint_always_places_center_even_at_density_zero() {
        let mut board = Board::new(20, 20);
        let mut rng = SmallRng::seed_from_u64(0xbeef_5eed);
        let painted = scatter_paint(&mut board, 10, 10, Cell::Sand, 3, 0.0, &mut rng);
        assert_eq!(painted, 1);
        assert_eq!(board.get(10, 10), Some(Cell::Sand));
    }

    #[test]
    fn scatter_paint_density_zero_places_only_center() {
        let mut board = Board::new(20, 20);
        let mut rng = SmallRng::seed_from_u64(0xbeef_5eed);
        scatter_paint(&mut board, 10, 10, Cell::Water, 3, 0.0, &mut rng);
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
        let painted = scatter_paint(&mut board, 10, 10, Cell::Stone, 3, 1.0, &mut rng);
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
}
