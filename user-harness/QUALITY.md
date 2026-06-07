# Quality

## Minimum bar
- Fix root causes instead of only masking symptoms.
- Keep changes coherent, scoped, and technically defensible.
- Add or update regression coverage when fixing a reproducible bug.
- Prefer targeted verification over claims based on inspection alone.

## Verification defaults
- Code change: run the smallest relevant test or check first.
- Shared logic change: add adjacent regression checks.
- UI change: verify rendering, interaction, and responsive behavior.
- Parser or data transform change: add concrete input-output tests.
- Workflow or harness change: validate the expected command path and failure mode.

## Review checklist
- Is the change scoped to the stated problem?
- Does it preserve behavior outside the target path?
- Is verification proportional to risk?
- Are public behavior changes explicit?
- Are assumptions and residual risks documented?

## Documentation
- Update docs when behavior, workflow, or canonical guidance changes.
- Remove stale guidance rather than letting conflicting instructions remain.
- Keep repo-specific facts in project docs, not in this global harness.
