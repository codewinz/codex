# Sentry TDD Reference

- Start from the closest real artifact: event metadata, stack trace, report text, screenshot, or log.
- Convert the production input into a minimal failing automated test when possible.
- Apply the narrowest generalizable fix.
- Re-run the targeted test and nearby regression coverage.
- Record any production-only uncertainty that cannot be reproduced locally.
