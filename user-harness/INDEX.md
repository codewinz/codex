# User Harness Index

This directory contains the default Codex operating harness for this workspace.

Use this harness for:
- personal workflow defaults
- quality and verification expectations
- planning protocol
- reusable references and templates

Do not use this harness for:
- repository architecture
- product rules
- database schema
- service integrations
- service or product deployment runbooks
- repo-specific exceptions

Document map:
- [WORKFLOW.md](./WORKFLOW.md): default execution loop and handoff behavior
- [QUALITY.md](./QUALITY.md): quality gates and review baseline
- [PREFERENCES.md](./PREFERENCES.md): communication and implementation preferences
- [RELIABILITY.md](./RELIABILITY.md): bugfix and regression-prevention workflow
- [PLANS.md](./PLANS.md): plan requirements and templates
- [references/](./references/): optional reusable references
- [templates/](./templates/): reusable writing templates
- [references/codex-cli-deployment.md](./references/codex-cli-deployment.md): local Codewinz Codex CLI deployment rule

Precedence:
1. system and developer instructions
2. explicit user instructions
3. project `AGENTS.md` files and repo-specific canonical docs within their scope
4. this global harness

Project-specific guidance overrides this global harness for repository facts,
architecture, domain rules, database schema, service integrations, workflows,
and exceptions.
