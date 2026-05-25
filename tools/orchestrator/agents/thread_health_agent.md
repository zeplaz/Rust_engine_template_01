# thread_health_agent

**Lane:** Runtime `OrchestratorThreadHealthExport`, `reports/thread_health.md`.

## Read first

- `src/dev/orchestrator_health.rs`
- `debug_runs/orchestrator_thread_health.json` (when `ORCHESTRATOR_EXPORT_HEALTH=1`)

## Commands

```powershell
$env:ORCHESTRATOR_EXPORT_HEALTH = "1"
# run app, then:
cargo orchestrate -- --skip-cargo --runtime-snapshot debug_runs/orchestrator_thread_health.json
```
