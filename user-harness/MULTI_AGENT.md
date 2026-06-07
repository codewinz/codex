# Multi-Agent

Use this document when the platform exposes sub-agent tools and the user request
allows delegation.

## Delegation policy
- Use sub-agents for non-trivial work when the user explicitly asks for delegation or the active tool policy permits it.
- Delegate independent sidecar tasks that can run in parallel without blocking the main critical path.
- Keep urgent blocking work local when waiting would slow the task down.
- Keep shared contracts, migrations, generated artifacts, and final integration coordinator-owned unless the split is explicit.

## Role mapping
- `explorer`: read-only discovery, reference tracing, and narrow factual questions.
- `worker`: implementation or test updates with a bounded, disjoint write scope.
- `reviewer`: read-only critique, regression analysis, and risk finding before final handoff.

## Role rules
- Keep `explorer` read-only.
- Keep `reviewer` read-only.
- Use `worker` only when success means changed artifacts in a defined scope.
- After delegated work returns, inspect the diff and run independent verification before relying on it.

## Coordination
- Assign write ownership before spawning implementation agents.
- Avoid overlapping edits unless one coordinator owns integration.
- Store handoff notes in the project-approved location when the repo defines one.
