# feat: add CLAUDE.md and agent workflow

## What changed

- `CLAUDE.md` (new) — project guide: layout, the simulation/app layering rule,
  the grid and step-order invariants, commands, conventions, the agent
  workflow, and the working agreement to stop for manual testing after large
  changes.
- `.claude/agents/sim-scoper.md` (new) — read-only agent that scopes the next
  ticket into a spec under `docs/specs/`.
- `.claude/agents/sim-dev.md` (new) — implements a spec with tests; carries the
  layering rule and the five element touchpoints.
- `.claude/agents/sim-reviewer.md` (new) — read-only diff review against the
  sim's invariants (`FLAG_MOVED`, scan order, coordinate direction,
  determinism, layer purity, mass conservation).
- `.claude/agents/sim-verifier.md` (new) — steps a board headlessly and dumps
  ASCII to check observable behavior.
- `.claude/commands/log-commit.md` (new) — the commit + `logs/` entry convention.
- `logs/.gitkeep` (new) — so the log directory is tracked while empty.

## Why

The project exists partly to build an agentic workflow for it, so the workflow
is itself a deliverable. The agents are split so the one that writes code is
never the one that reviews it. The review agent gets the most detail because
this codebase's bugs are silent — a mis-flagged `FLAG_MOVED` destination or a
broken scan-direction alternation produces sand that looks subtly wrong rather
than a crash or a failing test.

`/log-commit` is a command rather than an agent because the commit + log format
is mechanical and does not need a model call to reason about it.

Deviation from the convention as originally written: the log filename format
was `{MM/DD HH:MM}`. A literal `/` cannot appear in a filename — it would have
created a `logs/09-21/` subdirectory — so the date is dash-separated
(`MM-DD-HHMM`).

## Verification

Documentation and configuration only; no Rust source touched. `cargo test`
was not re-run because nothing compiled changed.

Still to check by hand: whether the agent descriptions match how they should
actually be dispatched, and whether the scoped ticket sizes come out right in
practice.
