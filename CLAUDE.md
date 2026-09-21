# falling-sand

A falling-sand particle simulation: a 2D grid of cells (sand, water, stone) that
fall, pile, and flow, painted in with the mouse. The goal is a responsive, fun
sandbox toy with element behavior that *feels* physical, not a physically
accurate simulation.

This project is made to learn to create an agentic workflow for myself, as well as
learning to create a scalable / scaled project.

## Layout

Cargo workspace, edition 2024.

```
libs/simulation/   # the simulation library — all logic lives here, no I/O
  src/cell.rs      # Cell enum (element kinds) + CellSlot (cell + per-step flags)
  src/board.rs     # Board: the grid, get/set, step(), per-element movement rules
  src/brush.rs     # Brush: per-element scatter shape + density for painting
  src/render.rs    # Board::render — writes the grid into an RGBA8 buffer
  src/rules.rs     # empty placeholder, not yet wired into lib.rs
app/               # the binary: minifb window, input handling, nearest-neighbor upscale
  src/main.rs
docs/              # design specs (gitignored, local only)
```

`simulation` must stay free of windowing/rendering deps — it only produces an
RGBA byte buffer. `app` owns the window, keyboard/mouse, board size, and scale.

## Key concepts

- **Grid**: flat `Vec<CellSlot>`, indexed `y * width + x`. `x` is left→right,
  `y` is top→bottom (so "down" is `y + 1`).
- **Step order**: `Board::step` scans bottom row upward, alternating left→right
  and right→left each step to avoid directional bias.
- **`FLAG_MOVED`**: set on a cell after it moves so it can't move twice in one
  step; all flags are cleared at the end of `step`.
- **Adding an element**: add a `Cell` variant, then handle it in
  `Board::update_cell`, `brush_params`, `cell_color`, and the key bindings in
  `main.rs`.

## Commands

```bash
cargo run -p app --release   # run the app (release — it's slow in debug)
cargo test                   # unit tests (mostly in board.rs and brush.rs)
cargo fmt
cargo clippy --all-targets
```

## Conventions

- Doc comments (`///`) on public items and on non-obvious private helpers;
  existing code documents coordinate/bounds behavior explicitly — match that.
- Bounds are handled by returning `Option`/`bool` rather than panicking.
- Tests live in `#[cfg(test)] mod tests` at the bottom of the file they cover.
- RNG is passed in as `&mut impl Rng` and seeded in `main`, so simulation
  behavior is reproducible — don't reach for thread-local randomness.
- Commits will be structured as such: {feat/fix/review}:{description}
- Each commit will have an associated markdown file explaining what CLAUDE did.
  This file will live at the project root under a folder called logs/ and each file
  will match its commit with the following structure: {MM-DD-HHMM}-{description}
  (a literal `/` can't appear in a filename, so the date is dash-separated).

## Workflow

Four agents in `.claude/agents/`, kept separate so the agent that writes code
is never the one that reviews it:

- `sim-scoper` — the project manager. Owns `docs/specs/ROADMAP.md`, reconciles
  it against `git log` and `logs/`, decides what gets built next, and writes
  that ticket as a spec in `docs/specs/`. Biased toward shipping visible
  behavior over cleanups. Read-only.

  It works in two phases. **Phase 1 (current):** make sand, water and stone
  genuinely good — sand piles at a believable slope and sinks through water,
  water finds its level without teleporting or twitching, stone holds material
  without leaking. **Phase 2 (gated):** new elements — ice, fire, smoke, oil
  and friends, sequenced cheapest-machinery-first. Phase 2 opens when the
  Phase 1 bar is met *or* when I say the three existing elements are good
  enough; the element list is deliberately still open.
- `sim-dev` — implements a spec, with tests.
- `sim-reviewer` — reviews the diff against the sim's invariants
  (`FLAG_MOVED`, scan order, coordinate direction, determinism, layer purity).
  Read-only. Run this before committing.
- `sim-verifier` — steps a board headlessly and dumps it as ASCII to check
  what the material actually did, since "feels physical" isn't unit-testable.

`/log-commit` handles the commit message + `logs/` entry — it's mechanical, so
it's a command, not an agent.

## Working agreement

- Stop for manual testing after every large change. A "large change" is a
  feature that alters how the sim looks or feels, or a series of tickets landed
  together. Don't chain into the next piece of work: report what changed, say
  exactly what to try in the app (which keys, what to expect), and wait. Passing
  `cargo test` is not the same as the feature feeling right, and only a human
  playing with it can judge that.
- Small, self-contained changes (a typo, a doc comment, a test-only commit) don't
  need a stop.
