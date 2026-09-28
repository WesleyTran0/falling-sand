# feat: add on-screen controls HUD

## What changed

- `app/src/hud.rs` (new) — self-contained HUD module.
  - A baked **5x7 bitmap font**, 53 glyphs (`A-Z`, `0-9`, space, and
    `: . , - + = / ! ? ' ( ) [ ]`). Each glyph is a `[u8; 7]`, one byte per
    row, top row first so it matches the framebuffer's y-down orientation, and
    written as `0b01110`-style literals so each glyph reads as a picture of the
    character in the source. 5x7 rather than 3x5 because 3x5 has no room for a
    legible `S`, `0` or `W`. No font dependency was added.
  - `Frame<'a>` — a checked view over the window framebuffer. `Frame::new`
    returns `None` on a size mismatch, and every primitive clips against the
    view, so drawing past an edge drops pixels instead of panicking or wrapping
    onto the next row.
  - `blend` — per-channel `(s*a + d*(255-a) + 127) / 255`, computed in `u32` so
    no channel can overflow.
  - The legend content, and `HudStyle` (font scale, padding, panel colour and
    alpha) with a `const fn panel_height()`.
- `app/src/main.rs` — `mod hud;`, named geometry constants, `hud_visible`
  state, an edge-triggered toggle, and a `hud::draw` call after the upscale
  loop. The upscale loop body is byte-identical; only its bounds now read
  `BOARD_PIXEL_*` instead of `WINDOW_*`.
- `libs/simulation/src/lib.rs` — one line: `pub use render::cell_color;`.

## Why

Requested: an on-screen key for the controls, "a transparent box at the top
would be fine for now, but later it probably should be below everything."

The HUD shows the key bindings and — the part that matters most day to day —
**which element is currently selected**, as text plus a colour swatch. Before
this there was no way to tell what you were about to paint.

**Toggle key is `H`**, edge-triggered via `get_keys_pressed(KeyRepeat::No)`.
`is_key_down` polls every frame, so at 60fps a toggle bound to it would flip
about thirty times per keypress. The element keys keep using `is_key_down`
because assigning the same value repeatedly is idempotent.

**Stage two is three constants, not a rewrite.** Geometry is always supplied by
the caller; nothing in `hud.rs` knows where the panel lives. To move the HUD
below the board: set `HUD_STRIP_HEIGHT` to the panel height (the window grows,
the board does not), set `HUD_ORIGIN_Y` to `BOARD_PIXEL_HEIGHT`, and set the
panel alpha to opaque. Two tests already pin that path —
`draw_at_a_lower_origin_moves_the_whole_panel` asserts nothing above the origin
is touched, and the panel-rect test asserts nothing below it is.

### The one deviation from the brief, and why

The brief said `libs/simulation` must not be touched at all. Followed literally,
the HUD's swatch colours had to be copied literals mirroring `cell_color`, free
to drift the moment an element's colour changes. `cell_color` is already a
`pub fn`; only its module is private. Exporting it is a pure re-export
returning `[u8; 4]` — no windowing type crosses the boundary and the layer rule
in `CLAUDE.md` is not weakened — so the export was the better call and the
duplication is gone. `element_swatch` now derives from `cell_color`, with
`Cell::Empty` the deliberate exception: its sim colour is the near-black
background and would be invisible on the panel, so the eraser gets a neutral
grey. Both facts are pinned by tests.

The other deviation is cosmetic: the brief said to leave the upscale loop
as-is, but with a taller window it must not iterate the HUD strip. The body is
unchanged; only the bounds constants moved.

## Verification

- `cargo test --workspace` — 20 passed in `app` (new), 42 in `simulation`
  (untouched). `cargo clippy --all-targets` zero warnings, `cargo fmt --check`
  clean, `cargo build -p app --release` clean.
- The app tests cover the pure logic, not the window loop: every character in
  the real HUD strings has a glyph (so adding a string with an unsupported
  character fails the test rather than rendering a blank), glyph orientation
  via an asymmetric `L`, pen advance, alpha blending across all 256 alpha
  values including endpoints, `fill_rect` clipping with `usize::MAX` extents,
  right-edge no-wrap, bottom-edge clip, off-origin draw, and an undersized
  frame returning `None`.

## Manual test required

Look-and-feel change, so it needs a human. `cargo run -p app --release`.
Expect a translucent dark band across the top with three lines of white text,
with falling sand still visible through it. Press `1`/`2`/`3`/`0` and watch the
`SELECTED:` line and its swatch change. Press `H` to hide and show — it must
not flicker while the key is held. Paint a bright sand pile under the band and
check the text is still legible.

## Noted, not fixed

- Stage two will need the HUD strip cleared each frame when the HUD is hidden,
  since nothing else redraws those pixels. Not an issue now: the overlay is
  redrawn from the board every frame.
- `H` collides with nothing today, but it is a plausible future element key.
  If element letters get used, the toggle should move to `F1` or `Tab`.
