## What this project is

`falling_sand` is a falling-sand cellular-automaton simulation written in Rust.
Elements (sand, water, stone) are painted onto a grid and updated each frame by
per-element physics rules, then rendered to a window. It's an interactive
desktop app: left-click paints the currently selected element, number keys pick
the element.

This is a learning vehicle. The owner is deliberately using it to build up
toward a **larger-scale Rust system**, so favor choices that scale and teach
good habits over quick hacks:
- Clean module boundaries and a real workspace/crate split (already in place).
- The simulation logic lives in a library crate with no windowing/rendering
  dependency, so it stays testable and reusable — keep it that way.
- Thorough unit tests and doc comments are expected, not optional.
- When there's a tradeoff between "clever" and "clear/idiomatic," pick idiomatic.

## Project structure

Cargo workspace (`Cargo.toml` at root, `resolver = "2"`):

```
falling_sand/
├── Cargo.toml            # workspace root; members = libs/simulation, app
├── app/                  # binary crate: windowing, input, scaling, main loop
│   └── src/main.rs
├── libs/
│   └── simulation/       # library crate: the actual simulation (no UI deps)
│       └── src/
│           ├── lib.rs    # module wiring + public re-exports (Board, Brush, Cell)
│           ├── board.rs  # Board: grid storage + step()/physics dispatch
│           ├── cell.rs   # Cell enum + CellSlot (cell + per-step flags)
│           ├── brush.rs  # Brush: per-element scatter painting
│           ├── render.rs # Board::render — writes RGBA8 into a buffer
│           └── rules.rs  # (currently empty; reserved for physics rules)
└── docs/                 # design specs (git-ignored — see Tools/workflow)
```

Architecture notes:
- **`app` depends on `simulation`, never the reverse.** `simulation` must not
  pull in `minifb` or any UI/rendering-window dependency. `render.rs` only
  fills a plain `&mut [u8]` RGBA buffer; the app owns the window and scaling.
- **`Board`** stores a flat `Vec<CellSlot>` indexed `y * width + x`. `(x, y)` is
  `(column, row)`, origin top-left, `y` increases downward.
- **Per-step flags**: `CellSlot.flags` carries `FLAG_MOVED` so a cell that
  already moved this step isn't processed twice. Flags are cleared at the end of
  `Board::step`.
- **Scan direction alternates** each step (`scan_left_to_right`) to avoid
  directional bias in the automaton.
- **RNG is injected** (`&mut impl Rng`) into `step`, `paint`, and the physics
  helpers — never created inside the simulation. This keeps runs deterministic
  and testable (tests seed a `SmallRng`).

## Code style

Follow the conventions already established in `libs/simulation`:

- **Rust 2024 edition.** `rustfmt` defaults — run `cargo fmt` before finishing.
- **Doc comments (`///`) on public items** and on non-trivial private helpers,
  written as full sentences explaining intent (what/why), as in `board.rs` and
  `brush.rs`. Module-level behavior notes are welcome.
- **Small, focused helpers.** Physics is decomposed into `try_fall`,
  `try_flow_sideways`, `can_move_into`, `move_cell`, etc. Keep new rules in the
  same shape — a per-element `update_*` method dispatched from `update_cell`.
- **Return `bool` / `Option` to signal outcomes** (e.g. `try_fall` returns
  whether the cell moved; `Board::get`/`set` return `Option`/`bool` for bounds).
- **Bounds are always checked** at the storage boundary (`Board::get`/`set`,
  `can_move_into`). Callers rely on this rather than pre-checking — see the
  brush's negative-coordinate skip + `set`'s bounds check. (Historic bug: a
  bounds check once used `&&` instead of `||`; be careful with these.)
- **Tuning tables live in one function** (e.g. `brush_params(cell)`) and are
  pinned by a test, so accidental edits fail loudly.
- **Prefer `usize` for grid coords**, `i32` for signed offsets/deltas, and
  convert explicitly at the boundary (skip negatives before casting).
- Constants for magic numbers (`BOARD_WIDTH`, `FLOW_DIST`, `FLAG_MOVED`, …).

## Tools & workflow

- **Toolchain**: Rust 1.92 / Cargo, edition 2024.
- **Dependencies**: `minifb` (windowing/framebuffer, app only), `rand` (both
  crates). Shared deps should go in `[workspace.dependencies]` when added.
- **Run the app**: `cargo run -p app --release` (release matters for framerate).
- **Test**: `cargo test` (workspace) or `cargo test -p simulation`. Tests are
  colocated in `#[cfg(test)] mod tests` at the bottom of each file and use a
  seeded `SmallRng` for determinism. Add tests alongside any new rule/helper.
- **Format / lint**: `cargo fmt` and `cargo clippy` before wrapping up.
- **Design docs**: nontrivial features get a markdown spec in `docs/` first
  (see `docs/superpowers/specs/…-design.md` for the format: Problem, Goal,
  parameters, Algorithm, API changes, Testing, Out of scope). Note `docs/` is
  **git-ignored** — specs are local design aids, not tracked artifacts.
- **Commits**: short imperative subject lines, prefixed (`fix:`, `config:`). 
  Keep commits scoped to one change.

## Controls (current app)

- `1` sand · `2` water · `3` stone · `0` eraser (empty)
- Left-click: paint selected element · `Esc`: quit

## Known TODOs / rough edges

- `rules.rs` is empty — intended home for extracted physics rules as they grow.
- `lib.rs` still has the `cargo new` placeholder `add()` fn + `it_works` test;
  remove when convenient.
- Water flow lacks momentum (`update_water` TODO) — sideways motion is purely
  RNG-driven, so it looks less realistic than it could.
- `board.rs` has a `TODO` to add tests for `update_sand`/`update_water` and the
  movement helpers (the step/physics path is currently untested).
