# fix: interpolate brush strokes between cursor samples

## What changed

- `libs/simulation/src/brush.rs` — new `Brush::paint_line(board, from, to,
  cell, rng)`. Walks a Bresenham line between the two cells and stamps the
  existing `paint` at every cell along it, including both endpoints. Returns
  the total painted. Five new tests.
- `app/src/main.rs` — tracks `stroke_from: Option<(usize, usize)>`, the cursor
  cell from the previous frame. While the button is held it paints a line from
  the previous sample to the current one; on the first frame of a press it
  paints a single dab. Released button *or* cursor outside the window resets it
  to `None`.

## Why

Reported: "when I move the mouse too fast, it skips some spots."

The report came with a suggested fix — increase how often `get_mouse_down`
runs — and that diagnosis was wrong in a way worth recording, because the
obvious fix would have made things worse.

`get_mouse_down` is already polled every frame. The gaps were not missed polls:
each poll stamped the brush at exactly **one point**, so a drag covering 300 px
in a second genuinely moves ~5 px between frames at 60fps and leaves a row of
separated blobs. The poll saw every position; it just never filled the space
between them.

Raising the frame rate fails twice. It only narrows the gaps — move faster and
they return — and `board.step` is called once per frame (`main.rs`), so the
frame rate *is* the simulation's tick rate. Doubling it to smooth painting
would have made sand fall twice as fast, changing how the whole sim feels to
fix an input artifact.

Interpolating between consecutive samples makes the stroke continuous at any
cursor speed and is completely independent of frame rate.

### Why this lives in `Brush`, not in `main.rs`

"Stamp the brush along a line on the board" needs no windowing: it is a board,
two cell coordinates, an element and an `Rng`. Putting it in `simulation` keeps
`app` owning only what it actually knows — where the cursor was — and puts the
logic where the crate's test culture already is. `app` had no tests before the
HUD; `brush.rs` had eight.

The endpoints are tuples rather than four `usize` arguments because
`clippy::too_many_arguments` fires at 8/7. Tightening the signature was the
right answer rather than an `#[allow]`, and it reads better at the call site.

## Verification

- `cargo test --workspace` — 67 passed (20 app, 47 simulation; was 62).
  `cargo clippy --all-targets` zero warnings, `cargo fmt --check` clean.
- The new tests lean on an existing guarantee: `scatter_paint` always places
  the centre cell regardless of density, so every cell on the line is a brush
  centre and must be set even though the surrounding scatter is random. That
  makes "no gaps" assertable without depending on the RNG stream.
  - no gap between distant points, checked against an independently written
    walk of the same line
  - identical endpoints produce a board byte-identical to a single `paint`
    with the same seed, and the same painted count — a zero-length stroke must
    not stamp twice
  - forward and reverse strokes cover the same cells
  - pure horizontal, pure vertical and exact-diagonal cases
  - endpoints far off the board neither panic nor loop forever

Not covered by tests: that the stroke *looks* continuous in the window, and the
stroke-break behavior on release. Those need the manual test below.

## Manual test

`cargo run -p app --release`. Drag as fast as you can across the window with
sand — the stroke should be a continuous ribbon with no dotted gaps at any
speed. Then release the button, move to the far side of the window, and press
again: it must start a fresh dab, **not** draw a line across the board from
where you released. Same check dragging off the window edge and back in.
