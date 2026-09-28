# fix: correct stale rules.rs docs and adopt a PR workflow

## What changed

`CLAUDE.md` only. No code.

- **Layout block** — `src/rules.rs` was still described as "empty placeholder,
  not yet wired into lib.rs". It has held the material density table and been
  in the module list since `e3d79c9`. Now reads "per-element material data
  (density); consumed by board.rs".
- **"Adding an element"** — the touchpoint list omitted `rules::density`, so
  following it would have left a new element out of the density table. Added,
  and recorded that every one of those sites is an exhaustive `match` with no
  wildcard arm, so the compiler names each one you still owe. That is the
  safety net which makes a table split across five files tolerable, and it is
  worth saying out loud because the "fix" a hurried reader reaches for is a
  `_ =>` arm, which would silently give new elements wrong physics.
- **New `## Branching and PRs` section** — work now lands through pull
  requests rather than commits straight to `main`, one branch per ticket, with
  stacked branches when a ticket depends on one still in review. Records that
  `gh` is not installed, so the flow is push-branch-and-hand-over-a-compare-link.

## Why

The doc drift was found by a direct question: why is density a rule rather than
a property of a cell? Checking the answer surfaced that `CLAUDE.md` had been
wrong about `rules.rs` since D1 landed.

Worth recording the answer, since it will be asked again. The original H4
rationale — "`cell.rs` stays a pure data definition" — is weak: it cited
`brush_params` living next to its consumer as precedent, but density's only
consumer is `board.rs`, so that precedent argues for putting it *there*. The
module name is also plainly wrong; the actual rules (`try_fall`,
`can_displace`, `try_slump`) all live in `board.rs`.

The better justification is in the table itself. Two of its four entries are
not material facts: `Empty => 0` is the absence of material, with zero chosen
so the ordering stays total and `can_displace` needs no special case for air;
and `Stone => u8::MAX` is documented as a sentinel meaning "nothing may ever
displace this", which is a decision about the toy rather than a property of
rock. `Cell::Empty.density()` would assert that every cell *has* a density.
`rules::density(Cell::Empty)` reads honestly as a rule's treatment of air.

The likely end state, recorded here so it is not re-litigated: when a second
property arrives in Phase 2 (flammability for fire, a melting point for ice,
"rises" for smoke), collapse these into one `Material` struct and a single
`fn material(cell: Cell) -> Material`, so adding an element is one edit and the
whole material matrix can be read in one place. With exactly one property today
that refactor is premature; the decision point is property number two.

The PR workflow was requested directly: "I want you to start creating PRs
rather than merging, so that I can read the code in segments."

## Verification

Documentation only. `cargo test --workspace` still 76 passing (20 `app`, 56
`simulation`), clippy and fmt clean.
