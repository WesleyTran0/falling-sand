use crate::{Board, Cell};

/// The brush that draws elements onto the board is represented here
pub struct Brush {
    pub radius: i32,
    pub density: f64,
}

#[rustfmt::skip]
const HANDFUL_OFFSETS: &[(i32, i32)] = &[
    (0, 0), (-2, -1), (1, -2), (2, 1), (-1, 2),
    (-3, 0), (3, -1), (0, -3), (-1, -2), (2, -2),
    (-2, 2), (1, 3), (3, 2),
];

impl Brush {
    pub fn new(radius: i32, density: f64) -> Self {
        Self { radius, density }
    }

    /// Paints `cell` onto the `board` with `(cx, cy)` as the center point.
    pub fn paint(&self, board: &mut Board, cx: usize, cy: usize, cell: Cell) -> usize {
        // TODO: stone should hoenstly be put static but sand and water should be "random"
        let mut painted = 0;
        for &(dx, dy) in HANDFUL_OFFSETS {
            let x = cx as i32 + dx;
            let y = cy as i32 + dy;
            if x < 0 || y < 0 {
                continue;
            }
            if board.set(x as usize, y as usize, cell) {
                painted += 1;
            }
        }
        painted
    }
}
// TODO: write tests for brush
