# Workflow

## Default execution loop
- Ground in the real codebase, config, and runtime surface before making assumptions.
- Prefer execution over prolonged discussion once intent is clear.
- Keep changes scoped to the requested outcome.
- Verify the changed behavior with the smallest meaningful checks first.
- Expand verification when the change touches shared or risky surfaces.

## Exploration
- Read existing code, docs, tests, scripts, and generated artifacts before proposing structure.
- Prefer targeted searches and file inspection over asking discoverable questions.
- Surface assumptions when a decision cannot be derived from the repo or system.

## Implementation
- Preserve existing patterns unless there is a clear reason to replace them.
- Avoid broad refactors when a narrow fix satisfies the request.
- Keep user-facing behavior changes explicit in the final report.
- On Windows, split broad patches into small file-by-file or hunk-by-hunk edits.

## Verification
- Run targeted tests for the changed behavior.
- Run broader regression checks when shared infrastructure or reusable components change.
- If verification cannot be completed, state exactly what was not run and why.

## Handoff
- Summaries should focus on outcome, verification, and remaining risk.
- Report concrete commands and results, not general confidence.
- In a git repository, create a focused commit after completing repo-tracked changes unless the user explicitly says not to.
- Do not wait for a separate commit request after implementation work is complete; commit before the final handoff.
- Keep pre-existing or unrelated dirty worktree changes unstaged and out of that commit.
