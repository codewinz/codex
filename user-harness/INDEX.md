# User Harness Index

This directory contains the default Codex operating harness for this workspace.

Use this harness for:
- personal workflow defaults
- quality and verification expectations
- planning protocol
- multi-agent delegation guidance
- reusable references and templates

Do not use this harness for:
- repository architecture
- product rules
- database schema
- service integrations
- deployment runbooks
- repo-specific exceptions

Document map:
- [WORKFLOW.md](./WORKFLOW.md): default execution loop and handoff behavior
- [QUALITY.md](./QUALITY.md): quality gates and review baseline
- [PREFERENCES.md](./PREFERENCES.md): communication and implementation preferences
- [RELIABILITY.md](./RELIABILITY.md): bugfix and regression-prevention workflow
- [PLANS.md](./PLANS.md): plan requirements and templates
- [MULTI_AGENT.md](./MULTI_AGENT.md): role guidance for delegated work
- [references/](./references/): optional reusable references
- [templates/](./templates/): reusable writing templates

Precedence:
1. system and developer instructions
2. this global harness
3. project `AGENTS.md`
4. repo-specific canonical docs

When project docs conflict with this harness on project facts, architecture,
domain rules, or exceptions, follow the project docs.
