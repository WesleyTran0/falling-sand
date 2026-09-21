use crate::cell::Cell;

/// Material density of each element. A cell can sink through a cell of
/// strictly lower density.
///
/// This is *material* density, not the brush scatter probability also called
/// `density` in `brush.rs`.
///
/// The gaps between values are deliberate: a future element that belongs
/// between air and water, or between water and sand, gets a value without
/// renumbering the existing ones. `Stone` uses `u8::MAX` as a *sentinel*
/// rather than a measurement — nothing can be strictly denser, so stone can
/// never be displaced by any element added later.
pub(crate) fn density(cell: Cell) -> u8 {
    match cell {
        Cell::Empty => 0,
        Cell::Water => 1,
        Cell::Sand => 2,
        Cell::Stone => u8::MAX,
    }
}
