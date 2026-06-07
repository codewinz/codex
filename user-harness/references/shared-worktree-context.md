# Shared Worktree Context Reference

Some repositories coordinate parallel agents through two layers:

- local shared context under `<git-common-dir>/agent-context/`
- sanitized tracked exports under a repo-approved documentation directory

Use this pattern when a repo already provides commands, hooks, or docs for it.
Do not invent a parallel context store if the repo has an established harness.

Recommended behavior:
- Load shared context before changing files in handoff-heavy work.
- Update only the current worktree or assigned agent log by default.
- Export sanitized context before committing when the repo requires it.
- Never store secrets, tokens, account numbers, credentials, or local deploy config contents.

If the repo has no shared-context harness, treat this as a design reference only.
