# feat: add movement-rule tests for Board

## What changed

`libs/simulation/src/board.rs` — 386 added lines, all inside the existing
`#[cfg(test)] mod tests`. Production code (`board.rs:1-186`) is byte-identical to
`f652589`; verified by diffing the extracted prefixes, not just by reading the diff.

The `// TODO: add tests for update_sand and update_water and all helpers within
them` line is replaced by the two imports the tests need (`rand::SeedableRng`,
`rand::rngs::SmallRng`), matching the pattern already in `brush.rs`.

18 new tests, taking the suite from 18 to 36:

- **Sand / `try_fall`** (6): falls one row per step, not more; does not move on the
  bottom row; slides to the only open diagonal; does not move when fully blocked;
  a column falls without opening gaps.
- **Water / `try_flow_sideways`** (5): prefers falling over flowing; flows when it
  cannot fall; does not move when blocked both sides; stops before an obstacle
  rather than jumping it; flows at most `FLOW_DIST = 5` cells.
- **Step invariants** (7): `move_cell` flags the destination and `can_move_into`
  rejects it; water does not flow twice in one step; cell counts are conserved
  over 50 steps; `step` clears every flag bit; the scan direction alternates;
  stone never moves.

Three tests deliberately pin behavior that later tickets must change, each with a
doc comment naming the ticket and stating the assertion is current-but-unwanted:
`sand_does_not_yet_sink_through_water` (D1), `water_flows_at_most_flow_dist_cells_per_step`
(W2), `step_clears_all_flags` (W1).

## Why

Ticket T1 of `docs/specs/ROADMAP.md`, specced in
`docs/specs/2026-09-21-movement-rule-tests-design.md`.

Every line of movement logic was untested: a change that stopped sand falling
would have passed `cargo test`. That matters now because the next ticket (D1,
density and displacement) rewrites `can_move_into`, which every element's motion
funnels through.

Two tests exist because review found a coverage hole the spec had misdiagnosed.
The spec claimed `sand_falls_one_row_per_step_not_more` pins `FLAG_MOVED`. It does
not, and no falling test can: `step` scans bottom-up, so a cell that moves down
lands in an already-scanned row and is never revisited. Mutation testing confirmed
it — deleting the `FLAG_MOVED` guard in `update_cell`, or the
`flags |= FLAG_MOVED` in `move_cell`, left all 34 tests green. Sideways flow is
the only motion that can observe the flag, so `move_cell_flags_the_destination`
asserts it directly and `water_does_not_flow_twice_in_one_step` asserts it
behaviorally. This is the exact invariant D1's `swap_cells` will depend on, since
a swap moves the displaced cell *up* into the row being scanned.

`step_clears_all_flags` now sets a spare bit by hand before stepping. Without
that it could not tell `flags = 0` from `flags &= !FLAG_MOVED` (W1's change),
because `FLAG_MOVED` is the only bit production code ever sets — so W1 would have
landed with the test untouched and its behavior change invisible.

## Verification

- `cargo test --workspace` — 36 passed, 0 failed.
- `cargo clippy --all-targets` — zero warnings.
- `cargo fmt --check` — clean.
- **Determinism**: the suite is green with `rng.random_bool(0.5)` forced to a
  constant at both call sites (`board.rs:118`, `board.rs:138`), and green for all
  40 seeds 0-39 substituted into every `seed_from_u64`. No assertion depends on
  the RNG byte stream, so the suite survives `rand` changing its output.
- **Mutation testing** against a scratch copy. Caught: top-down scan; alternation
  removed; end-of-step flag clear deleted; `move_cell` duplicating material (8
  tests); `move_cell` not flagging the destination (2); `move_cell` flagging the
  source (3); `update_cell`'s `FLAG_MOVED` guard removed (1); `flags = 0` narrowed
  to `flags &= !FLAG_MOVED` (1); `can_move_into` accepting non-`Empty` (2);
  `FLOW_DIST` 4 or 6; diagonals before straight down (5); `y+1` → `y-1` (10); sand
  flowing sideways (4); stone falling (3); water never flowing (3).

Known residual gap, accepted: replacing `if self.scan_left_to_right` with `if
false` — always scanning right-to-left while still toggling the field — leaves the
suite green. Every scenario is built so exactly one destination is legal (that is
what makes them coin-proof), so scan order cannot matter by construction, and
statistical tests are out of scope per the spec. The suite pins that the field
toggles, not that anything reads it. Catching "piles lean one way" needs
`sim-verifier`, not `cargo test`.

No manual-test stop: test-only commit, no behavior change.
