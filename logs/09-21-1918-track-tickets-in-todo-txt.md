# feat: track tickets in todo.txt

## What changed

- `todo.txt` (new, committed) — the ticket list in todo.txt format. 11 open
  tickets and 8 closed, covering everything landed so far plus everything the
  manual test and the verifier runs turned up.
- `.claude/agents/sim-scoper.md` — the PM's standing "reconcile the roadmap"
  job now also requires mirroring it into `todo.txt`, with the field
  conventions spelled out.
- `CLAUDE.md` — the layout block names `logs/` and `todo.txt`, and the
  `sim-scoper` bullet says it owns both the roadmap and the ticket list.

## Why

Requested: the user has todo.txt tooling in nvim and wants the tickets there,
with the ticket type as the project and the format's other features used.

Conventions chosen, and the reasoning:

- **Project = the commit type** (`+feat`, `+fix`, `+review`), matching the
  `{feat/fix/review}:{description}` rule already in `CLAUDE.md`. This is the
  literal request, and it has a side benefit: a ticket already names the commit
  it will become, so filtering `+fix` shows the bug list.
- **Context = the layer** (`@sim`, `@app`, plus `@meta` for workflow work).
  The `libs/simulation` / `app` split is a hard rule in this project, so it is
  the most useful axis to filter on — and a ticket that would need both
  contexts is a ticket that should be split.
- **`id:` tags** (`D1`, `T1`, `W1`, ...) stay stable across `todo.txt`,
  `ROADMAP.md`, spec filenames and commit messages, so one ticket is traceable
  through all four.
- **Closed lines are kept**, not deleted, with `commit:<sha>` appended.
- **Abandoned tickets are completed with `RETIRED` and a reason** rather than
  deleted. D2 is the first: it was sink-rate tuning, and the manual test showed
  sand sinks through water fine, so there is no rate to tune. That decision is
  worth keeping visible.

`todo.txt` is committed, unlike `docs/` — `ROADMAP.md` holds the reasoning and
the file:line evidence and stays gitignored scratch, while `todo.txt` is the
terse durable index. The PM is now responsible for keeping them from
disagreeing, which is the failure mode to watch: the roadmap had already
drifted out of sync with `git log` once before the PM rewrite.

## Verification

No code changed; `cargo test --workspace` still 62 passing (20 app, 42
simulation), clippy and fmt clean. Format checked by eye against the todo.txt
spec: priorities are a leading `(X)` followed by a space, dates are
`YYYY-MM-DD`, completed lines start `x ` with completion date before creation
date, and no description contains a stray `+` or `@`.
