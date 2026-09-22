# feat: submerged sand slumps to a shallower angle of repose

## What changed

`libs/simulation/src/board.rs` only.

- `update_sand` now mirrors `update_water`: `try_fall`, and if that fails,
  `try_slump`.
- New private `try_slump(x, y)`. A supported grain in lateral contact with
  strictly lighter, non-empty material slides one cell sideways — but only into
  a position it could then descend from. Direction order comes from
  `self.scan_left_to_right`; the move goes through `swap_cells`.
- New free function `offset_x(x, dir) -> Option<usize>`, so the
  `checked_sub`-left / guard-right asymmetry is written once.
- 9 new tests. 67 -> 76 (20 `app`, 56 `simulation`).

## Why

The D1 manual test worked — sand sank through water — but the submerged pile
held the same steep ~45° cone as a dry one, with water trapped in the basin's
bottom corners. Reported as: the pyramid "isn't really a problem with sand.
However, it was a problem with water because that is unnatural for water
(because of liquid vs solid AND because of friction)."

That is right. Underwater, buoyancy cuts a grain's effective weight and the
water lubricates grain-on-grain contact, so a deposited mound slumps much
flatter than its dry angle of repose.

Spec: `docs/specs/2026-09-21-submerged-angle-of-repose-design.md`.

### The three decisions

**The trigger is contact, not submersion.** `can_displace(mover, nx, y)` — the
cell I would slide into holds lighter, non-empty material. Lubrication is a
contact property, so the contact predicate is the physical one, and it reuses
the predicate D1 already wrote. It also makes the dry-sand regression
*structural* rather than statistical: `can_displace` rejects `Cell::Empty`, so
the rule cannot fire in air at all. A pile that breaks the waterline therefore
gets a steep dry crown over a shallow submerged toe — a beach profile — for
free.

**The look-ahead must reach downward, and it is the anti-oscillation
guarantee.** Without it the rule is "supported sand slides toward any adjacent
water", whose terminal state is a flat bed that *oscillates*: two surface
columns differing by one height trade that difference back and forth forever.
A same-row look-ahead of any length does not help — on a descending slope
everything ahead in the row is water, so the check always passes. Requiring
`(nx + dir, y + 1)` to be empty-or-lighter means a slide is legal only when the
grain can genuinely continue downhill, so material only ever moves down-slope
and the stable surface differs by at most one row per two columns: ~26.6°.

**No new RNG call site.** D1 added none and this keeps that. The two directions
both qualify only when a grain is a local peak with downhill on both sides, so
the choice is a tie-break, taken from `scan_left_to_right`, which flips every
step.

## Verification

- `cargo test --workspace` 76 passed; `cargo clippy --all-targets` zero
  warnings; `cargo fmt --check` clean. `app/`, `rules.rs`, `cell.rs` and
  `brush.rs` untouched.
- Determinism: green with both `random_bool(0.5)` sites forced across all four
  true/false combinations, and across seeds 0-39 substituted into all 48
  `seed_from_u64` sites (including `brush.rs`'s hex literals, which a
  digits-only regex silently skips).
- **11 mutants, 0 survivors**, including: delete the rule; `swap_cells` drops
  either flag; delete the look-ahead; same-row look-ahead; upward look-ahead;
  look-ahead loses `dir`; trigger relaxed to `can_move_into` (fires in air);
  `>` relaxed to `>=`; two `move_cell` calls instead of `swap_cells`.
- **Dry sand is bit-identical**, verified empirically rather than argued: 20
  dry pours with the rule active vs `try_slump` stubbed to `return false`
  produce identical whole-grid FNV digests. Review reproduced this independently
  over 12 random dry boards and a 400-grain pour. Dry pile stays 20 tall / 39
  wide.
- **No oscillation**, measured in a realistic basin rather than only a unit
  test: 600 grains poured into a 40x30 water-filled basin, 4 seeds — **0 of 300
  post-settle steps show any sand motion**. Review's sealed-box and 400-grain
  runs agree (`sand_drift_max=0`). Residual surface motion is the pre-existing
  water jitter (W1), present with the slump rule stubbed out too.
- Effect: submerged mound goes from 19 tall / 40 wide / 37-38 floor cells still
  water, to **14 tall / 55-56 wide / 22-23 floor cells**, symmetric to within
  one column.

### Corrections to the spec, found while implementing

1. **Spec test E could not catch its own mutation.** A sand-into-sand swap
   leaves both cell *values* unchanged, so a value-based assertion passes under
   `>` → `>=`. Rewritten to drive `update_cell` directly and assert
   `FLAG_MOVED` is clear, the technique `same_density_cells_do_not_swap`
   already uses. Review reproduced the escape and confirmed the fix.
2. **The spec's suite had no per-step no-jitter test.** A settle-then-compare
   test can pass a period-2 oscillation by landing on an even beat — precisely
   the sampling trap that made a verifier run elsewhere in this project
   conclude water had settled when it was flipping every step. Added
   `a_submerged_bed_at_the_slump_angle_never_moves_on_any_step`, which asserts
   whole-board identity after *every* one of 50 steps.
3. **The submerged mound is not byte-identical across seeds**, as the spec
   claimed from a prototype using a different RNG. Structure (height 14, one
   row per two columns, symmetry) is seed-stable; the toe advances one extra
   column on some seeds. Not a defect.

### Review findings applied

- `try_slump` originally hardcoded `Cell::Sand` while its name, signature and
  doc were element-agnostic. Correct today, but a denser element later routed
  through this rule would have been compared using *sand's* density and
  silently refused to slump, with nothing failing. Now reads the mover from the
  grid, as `try_fall` does.
- Documented that the look-ahead is `FLAG_MOVED`-sensitive rather than purely
  geometric: row `y+1` is scanned first, so the target may already be flagged.
  Measured at ~28% of evaluations, always in the stricter direction (delays a
  slump, never permits an illegal one), and verified not load-bearing. It is
  recorded because ticket S1 must decide deliberately whether the shared
  predicate it extracts keeps the flag check — `try_fall` needs it, this
  look-ahead does not.
- Softened a doc claim that the tie-break's "bias cancels step to step"; the
  measured truth is that no arrangement shows a lean.

## Manual test required

1. **Dry control first.** Flat stone floor, pour sand in one spot. Expect the
   **same steep ~45° cone as before**, ~20 tall for ~39 wide. Flatter would
   mean something is wrong — the code path is provably dead in air.
2. Basin, fill with water, pour sand into the middle. Expect a low mound about
   two-thirds the height and ~1.4x wider, sides dropping one cell per two
   across, corner water pushed up. Static within about a second of the last
   grain.
3. **Not expected: a perfectly flat bed.** Some water stays beside the toe
   unless the mound reaches the walls. Flat is the one-line D3b knob
   (look-ahead `y + 1` -> `y`), deliberately unspecced pending a human verdict.
4. Optional: pour until the mound breaks the surface — expect a kink at the
   waterline, steep above, shallow below.

## Also in this commit

`todo.txt` re-prioritised on evidence gathered this round:

- **W1 raised to (A)** and restated. It was filed as "water looks jittery". A
  verifier run shows the real behavior is that *any* fill which is not an exact
  multiple of the basin width oscillates every single step forever — 250/250
  steps, mean 2.38 cells moving, max 14, with 5-cell jumps. A hand-painted fill
  is essentially never an exact multiple, so this is the normal case, not an
  edge case. Fully explained by `FLOW_DIST = 5` plus the absent rest state; no
  additional bug.
- **W2 raised to (B)**, same evidence.
- **S1 expanded** with the shared-predicate requirement and the property test
  review recommends adding first: over a random submerged board, every cell
  that slumped on step N has moved down by step N+1 unless its target was
  taken. That invariant is what S1 would break and nothing currently states it.
