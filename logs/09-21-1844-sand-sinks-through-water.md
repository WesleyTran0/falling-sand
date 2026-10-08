# feat: sand sinks through water via density displacement

## What changed

- `libs/simulation/src/rules.rs` — was a zero-byte file rustc never compiled.
  Now holds `pub(crate) fn density(cell: Cell) -> u8`: `Empty 0`, `Water 1`,
  `Sand 2`, `Stone u8::MAX`. Stone's value is documented as a sentinel rather
  than a measurement, so no element added later can displace it. The gaps
  between values are deliberate — a liquid between air and water (oil) gets a
  number without renumbering. The doc comment says explicitly that this is
  *material* density, not the scatter probability `brush.rs:11-14` also calls
  density.
- `libs/simulation/src/lib.rs` — `mod rules;`, no `pub use`. Nothing crosses the
  crate boundary.
- `libs/simulation/src/board.rs`:
  - `can_displace(mover, nx, ny)` — new predicate: in bounds, destination
    non-`Empty`, destination unflagged, and `density(mover)` **strictly**
    greater than the destination's.
  - `swap_cells(from_idx, to_idx)` — new sibling of `move_cell`, exchanges the
    two `cell` fields and sets `FLAG_MOVED` on **both** slots.
  - `try_fall` — reads the mover once, keeps its existing candidate list and
    coin-flipped diagonal order, and per candidate tries
    `can_move_into → move_cell`, else `can_displace → swap_cells`.
  - `can_move_into`, `move_cell`, `try_flow_sideways`, `step` and `update_cell`
    are untouched — the diff has no `-` line in any of them.
  - `sand_does_not_yet_sink_through_water` → `sand_sinks_through_water`,
    inverted in place with its setup preserved byte-for-byte, so the diff shows
    the inversion rather than a delete plus an unrelated add.
  - Six new tests. 36 → 42.

## Why

Ticket D1 of `docs/specs/ROADMAP.md`, specced in
`docs/specs/2026-09-21-density-displacement-design.md`. Sand and water shared a
board and ignored each other, which was the largest missing behavior in the toy.
The cause was one predicate: `can_move_into` accepted only `Cell::Empty`, so
sand resting on water was inert.

Three design decisions worth recording:

**`swap_cells` flags both slots.** A swap sends the displaced lighter cell
*upward*, into the slot the bottom-up scan is currently processing. The hazard
is not that row's scan revisiting it — it cannot — but the row *above*: a denser
cell at `(x, y-1)` would find the just-displaced water unflagged and swap again,
so water would climb several rows per step. The roadmap originally recorded the
wrong mechanism for this trap; it was corrected during scoping, before it could
produce a test that could not fail.

**`move_cell` still does not flag its source, and `Empty` destinations never
route through `swap_cells`.** `sand_column_falls_without_gaps` depends on the
row above being able to fall into the slot just vacated. The two predicates are
disjoint by construction (`== Empty` vs `!= Empty`), which is what makes the
`if`/`else if` order safe.

**`try_flow_sideways` is deliberately untouched.** Water's strictly-lighter set
is `{Empty}`, which `can_move_into` already handles, so sideways displacement is
a provable no-op with the current three elements. Implementing it now would add
unreachable, untestable code. It becomes real in Phase 2, with a liquid lighter
than water.

## Verification

- `cargo test --workspace` — 42 passed (36 → 42: 6 new, 1 rewritten in place).
- `cargo clippy --all-targets` — zero warnings. `cargo fmt --check` — clean.
- `app/` is byte-for-byte untouched; `libs/simulation/Cargo.toml` still lists
  `rand` as its only dependency; `rules::density` is `pub(crate)`.
- Determinism: green with `rng.random_bool(0.5)` forced to both `true` and
  `false` at both call sites, and green across seeds 0-39 substituted into every
  `seed_from_u64`. Note for future runs: `brush.rs` uses hex seed literals, so a
  digits-only substitution regex silently skips them.
- Mutation checks, each caught: `swap_cells` flagging only the destination →
  `displaced_water_does_not_move_twice_in_one_step` (and only that test, so it
  is not a tautology); `>` relaxed to `>=` → `same_density_cells_do_not_swap`;
  the comparison reversed → 9 tests; `density(Stone)` lowered below sand → 8.
- Review fuzzed the core invariant over 120 random boards x 40 steps: no water
  cell rose more than one row per step and no sand cell ever rose, across 4800
  steps. The same fuzz reports 155 violations when `swap_cells` is mutated to
  flag only the destination.
- Settling is stable: in a basin run to steady state and then watched for 100
  further steps across 4 seeds, the sand layer is identical on every step.

### Two mutations that survive, and why no test could catch them

Dropping the non-`Empty` guard from `can_displace`, and checking `can_displace`
before `can_move_into`, each leave all 42 tests green. This is not a coverage
hole. The two predicates are disjoint by construction, so each mutation alone is
a semantic no-op — review confirmed the outputs are *bit-identical* to base over
~1.1M sampled cell states, not merely test-equivalent. Applied **together** they
break `sand_column_falls_without_gaps`, which is what makes the guard worth
keeping: it is the single term that makes the disjointness true and stops a
future reorder from routing empties through `swap_cells`. Documented in place
rather than papered over with a test asserting a distinction that does not exist.

### Known issue, pre-existing, not introduced here

Sand leaks through a one-cell-thick diagonal stone wall. `try_fall`'s diagonal
candidates check only the destination cell, never the two orthogonal cells
between source and destination, so any diagonal seam is porous. Minimal repro:
stone at `(1,1)` and `(2,2)`, sand at `(1,0)` — the grain reaches `(1,2)` in one
step on exactly the 50% of seeds where the coin tries down-left first. Confirmed
present at `1f06fa0` as well as here, so D1 did not cause it. It needs its own
ticket against the Phase 1 "stone holds material" bar.

## Manual test still required

**This changes how the sim feels, so `cargo test` passing is not the verdict.**
Not yet judged by a human. Scenario and the open question (sink rate, which is
ticket D2) are in the spec's `## Manual test` section and in the handoff.
