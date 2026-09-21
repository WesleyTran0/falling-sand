# feat: make sim-scoper a project manager

## What changed

- `.claude/agents/sim-scoper.md` — rewritten from "writes the next ticket as a
  spec" into a PM role. Substantive additions:
  - An explicit bias toward shipping visible behavior: when a cleanup and a
    feature compete, pick the feature unless the cleanup actually unblocks it.
  - A two-phase product plan. **Phase 1** is a concrete quality bar for the
    three existing elements (sand piles at a believable slope and sinks through
    water; water finds its level without teleporting or twitching and is
    displaced rather than overwritten; stone never moves and does not leak
    material diagonally). **Phase 2** is new elements, and is gated on the
    Phase 1 bar being met or the user saying the three are good enough.
  - Phase 2 sequencing guidance by *machinery cost* rather than by appeal:
    density-only elements (oil) are cheap; state change (ice ⇄ water) forces a
    transformation pass that does not exist; neighbor-spread-with-lifetime
    (fire, smoke) also forces per-cell state the `Cell` enum cannot hold; and
    upward movement is called out as the largest architectural question on the
    horizon, because the bottom-up scan exists precisely so that falling works.
    The element list is left open on purpose.
  - A standing four-step job: reconcile `ROADMAP.md` against `git log` and
    `logs/`, decide one next ticket, spec it, and state what it is pushing the
    implementing agents toward next. The roadmap is named as the project's
    memory, and the agent is explicitly allowed to overrule its own stated
    order when new information warrants.
  - A `## Manual test` section is now required in every spec — naming the keys
    to press and what should be seen, because "check it feels right" is not
    actionable.
  - A "pitfalls this project has already hit" section: the T1 `FLAG_MOVED`
    misdiagnosis, the upward-movement double-move trap, the per-step flag wipe
    blocking persistent state, and coin-flip-dependent tests as latent flakes.
  - The stale hardcoded backlog list is gone. It had drifted — most of its
    items landed in `f652589` and `256a7bb` — and `ROADMAP.md` is the right
    home for it.
- `CLAUDE.md` — the `sim-scoper` bullet in `## Workflow` now describes the PM
  role and both phases, so the phase gate is visible in the always-loaded
  project instructions rather than only inside the agent definition.

The agent stays read-only (`tools: Read, Grep, Glob, Bash`) and its name is
unchanged, so `CLAUDE.md` and existing invocations still resolve.

## Why

Requested directly: the user wants the scoper "taking on more of a project
management role... pushing the other agents for more features," working toward
solid sand/water/stone before moving on to new cell types like ice, fire or
smoke.

Two additions go beyond the literal request and are worth flagging. The Phase 1
bar is written as observable behavior rather than "make them good," so the gate
can actually be evaluated instead of argued about. And Phase 2 is ordered by
machinery cost because the appealing elements (fire, smoke) are the expensive
ones — both need per-cell lifetime state and upward movement, which is the one
thing the current bottom-up scan is structurally hostile to.

## Verification

Prose and configuration only; no Rust changed, so `cargo test` is still 36
passing, clippy clean, fmt clean. The real check is behavioral and comes from
the next invocation: whether the agent reconciles the roadmap against `git log`
and returns one recommendation with a spec instead of a menu.
