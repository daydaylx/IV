# GitHub Copilot Instructions

Follow the canonical repository rules in [`AGENTS.md`](../AGENTS.md).

Before suggesting or applying changes:

1. Read [`AGENTS.md`](../AGENTS.md).
2. Read [`docs/PROJECT_STATE.md`](../docs/PROJECT_STATE.md).
3. Use [`docs/INDEX.md`](../docs/INDEX.md) to load only the task-relevant documentation.
4. Check the applicable items in [`docs/DEFINITION_OF_DONE.md`](../docs/DEFINITION_OF_DONE.md) before completion.

Keep IV a lightweight Linux terminal emulator. Do not introduce IDE features, autonomous command execution, telemetry, cloud accounts, multiple AI providers in the MVP, or direct VTE coupling outside the terminal backend.

Treat planned architecture as unimplemented until code and tests prove otherwise. Prefer small Rust changes, explicit error handling, keyboard-accessible GTK4 UI, non-blocking background work, and tests for state transitions. Never store secrets in source files, TOML, SQLite, fixtures, logs, examples, screenshots, prompts, or responses.
