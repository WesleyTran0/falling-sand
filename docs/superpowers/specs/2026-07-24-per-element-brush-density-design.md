# Per-element brush density/radius design

## Problem

`Brush::paint` currently applies the same fixed `HANDFUL_OFFSETS` scatter pattern
(13 hardcoded offsets within roughly a 3-cell radius) to every element. This
gives sand and water a pleasant "placing particles" feel, but looks wrong for
stone: a scattered, speckled clump of stone reads as broken rock/gravel rather
than a solid material.

The `Brush` struct already carries unused `radius: i32` and `density: f64`
fields (`libs/simulation/src/brush.rs`) — `paint()` ignores them entirely.
`main.rs` also has a stray commented-out call
(`// brush.paint(&mut board, cx, cy, current_element, &mut rng);`) directly
above the real call, evidence that a randomized, RNG-driven brush was the
original intent but never wired up.

## Goal

Replace the fixed offset table with randomized, per-element radius/density
sampling, so each `Cell` variant gets a scatter pattern tuned to how that
material should feel when placed.

## Per-element parameters

| Element | Radius | Density | Feel |
|---|---|---|---|
| `Cell::Sand` | 3 | 0.45 | Unchanged from today's default (`Brush::new(3, 0.45)`) |
| `Cell::Water` | 3 | 0.45 | Unchanged from today's default |
| `Cell::Stone` | 2 | 0.85 | Tighter radius, much higher fill — dense clump instead of scattered dots |
| `Cell::Empty` (eraser) | 3 | 1.0 | Solid full clear — erasing should not leave speckled leftover cells |

These live in a small function `fn brush_params(cell: Cell) -> (i32, f64)` in
`libs/simulation/src/brush.rs`, replacing the `Brush` struct's stored
`radius`/`density` fields (which were never actually per-element and are
currently dead).

## Algorithm

For a paint call at center `(cx, cy)` with element `cell`:

1. Look up `(radius, density) = brush_params(cell)`.
2. For every integer offset `(dx, dy)` with `dx, dy` in `-radius..=radius`
   where `dx*dx + dy*dy <= radius*radius` (circular mask, matching the rough
   shape of the current fixed table):
   - If `(dx, dy) == (0, 0)`: always place (guarantees a click always does
     something, even at density 0.0).
   - Otherwise: place only if `rng.gen::<f64>() < density`.
3. For each offset selected for placement, compute `(x, y) = (cx as i32 + dx,
   cy as i32 + dy)`; skip if `x < 0 || y < 0` (as today); otherwise call
   `board.set(x as usize, y as usize, cell)`. `Board::set`'s existing bounds
   check (recently fixed to use `||` instead of `&&`) safely rejects any
   coordinate beyond the board's width/height, so no additional bounds
   handling is needed in `Brush::paint`.

## API changes

- `Brush::paint` gains an `rng: &mut impl Rng` parameter.
- `Brush` becomes a stateless unit struct — no more stored `radius`/`density`
  fields, since tuning now varies per element rather than per brush instance.
  `Brush::new()` takes no arguments.
- `main.rs` updates its call site to pass `&mut rng` into `paint()` and drops
  the `Brush::new(3, 0.45)` arguments in favor of `Brush::new()`. The stray
  commented-out line at `main.rs:52` is removed (superseded by the real,
  now-correct call).

## Testing

Add unit tests in `libs/simulation/src/brush.rs` (replacing the existing
`// TODO: write tests for brush`), using a seeded `SmallRng` for determinism:

- Center cell is always placed regardless of density, including at
  density 0.0.
- Density 1.0 fills every cell within the radius circle exactly (deterministic
  full coverage — used to verify the eraser's solid-clear behavior).
- Density 0.0 places only the center cell.
- `brush_params` returns the exact `(radius, density)` tuple documented in the
  table above for each `Cell` variant, so a future accidental edit to the
  tuning table is caught by a test failure.

## Out of scope

- User-configurable brush size/density (e.g. a UI slider) — not requested,
  YAGNI for now.
- Distance-weighted density falloff (denser near center, sparser toward the
  edge of the radius) — the current design uses uniform density within the
  circular mask, matching the simplicity of the existing approach.
