# GitHub Copilot Instructions

Follow the canonical repository rules in [`AGENTS.md`](../AGENTS.md).

Required context before suggesting changes:

- [`README.md`](../README.md)
- [`docs/ARCHITECTURE.md`](../docs/ARCHITECTURE.md)
- [`docs/DEFINITION_OF_DONE.md`](../docs/DEFINITION_OF_DONE.md)

Keep IV a lightweight Linux terminal emulator. Do not introduce IDE features, autonomous command execution, telemetry, cloud accounts, multiple AI providers in the MVP, or direct VTE coupling outside the terminal backend.

Prefer small Rust changes, explicit error handling, keyboard-accessible GTK4 UI, asynchronous network work, and tests for state transitions. Never store secrets in source files, TOML, SQLite, fixtures, logs, or examples.
