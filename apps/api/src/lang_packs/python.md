## Python — Koda conventions

- Python 3.11+ features allowed
- Type hints required on all public functions and class attributes
- Async: asyncio/httpx preferred over sync requests
- No bare `except` — always catch specific exceptions
- Formatting: Black + isort enforced
- Linting: ruff with default ruleset
- Tests: pytest with fixtures, no unittest
- Secrets: never in code — use env vars or SecretRef
