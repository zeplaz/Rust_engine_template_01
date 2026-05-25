# stage5_readiness_agent

**Lane:** `Stage5ReadinessProfile::FULL_APP`, `STAGE5_TODOS`, live proof JSON.

## Read first

- `prompts/guides/stage5_convergence_directive_v1.md`
- `tools/orchestrator/runbooks/stage5_convergence.md`
- `tools/orchestrator/queues/stage5_todo_crossref.md`
- `src/dev/stage5_live_todos.rs`

## Exit gate

FULL_APP green in **running app** — `debug_runs/stage5_full_app_live.json`.

## Cycle

`cargo test -p proc_A_dine01 --lib` → `cargo orchestrate` → FULL_APP probe → fix highest authority violation.
