# review: remove cargo template code, add Brush Default, run cargo fmt

## What changed

- `libs/simulation/src/lib.rs` — deleted the `cargo new` template `pub fn add(left: u64, right: u64)`
  and its `#[cfg(test)] mod tests` / `it_works` test. Nothing in the workspace
  referenced either. The file is now just the module list and the three `pub use`
  re-exports.
- `libs/simulation/src/brush.rs` — added `impl Default for Brush` delegating to
  `Brush::new()`, and a doc comment on `Brush::new` noting the brush is stateless
  and all tuning is looked up per element in `brush_params`. The rest of the diff
  in this file is `cargo fmt` exploding hand-collapsed multi-argument
  `scatter_paint` calls and `Shape::Rectangle` literals in the test module.
- `libs/simulation/src/render.rs` — `cargo fmt` only: import order (`board` before
  `cell`), the extra space in the `Cell::Sand` match arm, and the wrapped
  `assert_eq!` in `render`.

## Why

Epic 1 (H1, H2, H3) of `docs/specs/ROADMAP.md`. All three are no-behavior-change
items, so they land as one commit ahead of the first real ticket (T1, movement-rule
tests, specced in `docs/specs/2026-09-21-movement-rule-tests-design.md`).

The `Default` impl clears the only warning `cargo clippy --all-targets` produced
(`clippy::new_without_default`), so the next ticket starts from a clean clippy run
and any new warning it introduces is unambiguously its own.

## Verification

- `cargo fmt --check` — clean.
- `cargo clippy --all-targets` — zero warnings (was 1).
- `cargo test --workspace` — 18 passed, 0 failed. Was 19; the removed count is
  exactly the deleted `it_works` template test.

No manual check needed. The simulation's behavior is untouched: `board.rs` has no
diff at all, and the only non-formatting additions are a `Default` impl and a doc
comment.
