# fix: untrack leftover docs/ spec

## What changed

Removed `docs/superpowers/specs/2026-07-24-per-element-brush-density-design.md`
from git tracking (87 lines). The file was already absent from disk; only the
index still carried it.

## Why

Commit `2617e03` ("removed docs from git repo") added `/docs` to `.gitignore`,
but `.gitignore` does not apply to already-tracked files, so this one survived.
It has shown as `D docs/superpowers/...` in `git status` ever since, which meant
any `git commit -a` would have swept an unrelated 87-line doc deletion into
whatever commit was being made. It was deliberately kept out of `f652589` and
`256a7bb` for that reason.

`docs/` itself stays. It is not unused: `docs/specs/ROADMAP.md` and
`docs/specs/2026-09-21-movement-rule-tests-design.md` are the live scoping
artifacts, and CLAUDE.md's workflow points `sim-scoper` at `docs/specs/`. The
directory is gitignored on purpose — specs are scratch, not history, and only
`logs/` is committed. The `superpowers/` subtree was the dead part: a leftover
path from a previous workflow that nothing references now.

## Verification

`git status --porcelain` is empty on a clean tree. No source or build change, so
`cargo test` (36 passing) is unaffected.
