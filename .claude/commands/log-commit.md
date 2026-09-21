---
description: Commit the current changes following the project convention and write the matching logs/ entry.
---

Commit the staged/unstaged work following this project's convention. Arguments, if any: $ARGUMENTS

Steps:

1. Run `git status` and `git diff` (plus `git diff --staged`) to see what's
   actually changing. Run `git log --oneline -5` to match the existing style.
2. Pick the type: `feat` for new behavior, `fix` for a bug, `review` for
   review-driven cleanup. Message form is `{type}:{description}` — lowercase,
   imperative, no trailing period.
3. Get the timestamp with `date +%m-%d-%H%M`.
4. Write `logs/{MM-DD-HHMM}-{description}.md`, where `{description}` is the
   commit description in kebab-case. Contents:

   ```markdown
   # {type}: {description}

   ## What changed
   Files touched and what each change does.

   ## Why
   The reason — link the spec in docs/specs/ if one drove this.

   ## Verification
   cargo test / clippy results, plus anything a human still needs to check by hand.
   ```

   Describe only what actually happened. No speculation, no filler sections.
5. Stage the source changes and the log file together, then commit so the log
   lands in the same commit as the code it describes.

End the commit message with:

```
Co-Authored-By: Claude Opus 5 (1M context) <noreply@anthropic.com>
```

Do not push unless asked.
