# Reliability

## Bugfix default
1. Reproduce from the closest real input available.
2. Add or update a failing test when the issue is reproducible in automation.
3. Apply the narrowest root-cause fix.
4. Re-run the targeted test, then adjacent regression checks.
5. Document any residual risk or unverified edge cases.

## Incident handling
- Prefer concrete evidence over speculation.
- Preserve reproduction artifacts when they clarify the failure.
- Avoid mixing broad cleanup into incident fixes unless required for correctness.
- When logs and visible behavior disagree, treat visible user-facing behavior as the acceptance check.

## Regression prevention
- When a bug reveals a missing invariant, encode it in a test or validator.
- When docs caused the mistake, update or remove stale guidance.
- When a workflow depends on local setup, verify the setup command and final state.

## Tooling reliability
- Verify tool availability and version from the active shell before depending on it.
- If a tool behaves differently across shells, rerun in the shell specified by the user.
- Distinguish host invocation failures from repo or product failures.
