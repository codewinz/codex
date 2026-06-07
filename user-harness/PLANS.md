# Plans

## When to plan
- Plan when work spans multiple subsystems, changes behavior, or contains unresolved tradeoffs.
- Skip formal planning only for narrow, low-risk changes where intent and implementation are obvious.

## Decision-complete standard
A useful plan leaves no important implementation decisions to the implementer.

Every substantial plan should cover:
- goal and success criteria
- important behavior or implementation changes
- public interfaces or contracts that change
- verification strategy
- assumptions and defaults chosen

## Reusable templates
- [Execution Plan](./templates/execution-plan.md)
- [Bugfix TDD](./templates/bugfix-tdd.md)
- [Review Checklist](./templates/review-checklist.md)

## Writing guidance
- Prefer behavior-level descriptions over file-by-file inventories.
- Mention files only when they prevent ambiguity.
- Keep plans compact unless the task genuinely needs more detail.
- Record scope boundaries when they prevent likely implementation mistakes.
