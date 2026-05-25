# Build orchestrator

Architectural intent-preservation pipeline for the Rust engine template. **Not** a “delete warnings” bot.

## Pipeline

```text
Build Run (cargo check / clippy / test)
    ↓
Compiler Diagnostics Collector  (--message-format=json)
    ↓
Warning Classifier              (WarningState + lifecycle)
    ↓
Subsystem Tracer
    ↓
Ownership Resolver
    ↓
Architectural State Analyzer    (BROKEN / IN_PROGRESS / STAGING / LEGACY)
    ↓
Runbook + Report Generator
    ↓
Continuation Queue
    ↓
Agent Coordination Reports
```

## Run

```powershell
# From repo root (runs check + clippy + test, writes reports)
cargo run --manifest-path tools/orchestrator/Cargo.toml

# Faster: diagnostics from check only
cargo run --manifest-path tools/orchestrator/Cargo.toml -- --skip-clippy --skip-test

# Re-classify from existing tree without cargo
cargo run --manifest-path tools/orchestrator/Cargo.toml -- --skip-cargo
```

Alias (after `.cargo/config.toml` is present):

```text
cargo orchestrate
```

### Plan slice (development → implementation)

Picks the next implementation slices from `debug_runs/*_live.json` + [`src/dev/stage5_triage_backlog.md`](../../src/dev/stage5_triage_backlog.md) + open markdown todos.

```powershell
.\tools\orchestrator\scripts\plan_slice.ps1 -SkipCargo
# or:
cargo orchestrate --plan-slice --skip-cargo
```

Outputs:

| Path | Purpose |
|------|---------|
| `reports/plan_slice.md` | Human-readable plan + witness digest |
| `queues/continuation_queue.json` | Machine queue for agents |
| [`src/dev/development_plan_index.md`](../../src/dev/development_plan_index.md) | Master index (manual) |

Start a slice:

```powershell
.\tools\orchestrator\scripts\invoke_slice.ps1
.\tools\orchestrator\scripts\invoke_slice.ps1 -SliceId SLICE-TRIAGE-VM-06
```

### Main-thread orchestrator (Shift A→B in code)

Runs **debug-intelligence** + **cleanup-completion-intelligence** + **bevy-simulation-grade** authority scan; writes `debug_runs/main_thread_orchestrator_live.json`.

```powershell
.\tools\orchestrator\scripts\main_thread_shift.ps1
# or:
cargo orchestrate --main-thread-shift --skip-cargo
```

Agent: [`.cursor/agents/main-thread-orchestrator.md`](../.cursor/agents/main-thread-orchestrator.md)

Post-build hook (optional):

```powershell
.\tools\orchestrator\hooks\post_build.ps1
```

Set `RUST_ENGINE_ORCHESTRATE=1` before `cargo check` to chain automatically.

## Outputs

| Path | Purpose |
|------|---------|
| `reports/build_report.md` | Last cargo phase summary |
| `reports/warning_registry.md` | Classified issues |
| `reports/ownership_map.md` | Owner routing |
| `reports/migration_tasks.md` | Migration continuation tasks |
| `reports/deprecation_tracker.md` | Deprecated API surface |
| `reports/system_completion.md` | Lifecycle counts |
| `runbooks/viewport_pipeline.md` | Viewport authority runbook |
| `queues/agent_queue.md` | Active migrations + agent list |
| `queues/continuation_queue.json` | Machine-readable tasks |
| `state/last_run.json` | Latest snapshot |
| `history/run_*.json` | Historical snapshots |

## Source annotations

Tag in-progress work so cleanup agents do not destroy active development:

```rust
/// @orchestrator-status IN_PROGRESS
/// @orchestrator-owner viewport_migration_agent
/// @orchestrator-do-not-cleanup
pub fn semantic_viewport_from_map_fill(...) { ... }
```

## Knowledge base

Edit `knowledge/*.json` to extend subsystem graphs, migration pairs, and active migration rules.

## Classifier notes (O-23)

| Spec name | `WarningState` |
|-----------|----------------|
| ArchitecturalLeak | `VisibilityViolation` |
| Unused import (viewport debug) | `TransitionalArchitecture` + `do_not_touch` |
| `unused_mut` / `unused_variables` | `DiagnosticNoise` |

## Reports policy

See [`REPORTS_POLICY.md`](REPORTS_POLICY.md).

## Runtime thread health (R-03)

```powershell
$env:ORCHESTRATOR_EXPORT_HEALTH = "1"
# run game, then:
cargo orchestrate -- --skip-cargo --runtime-snapshot debug_runs/orchestrator_thread_health.json
```
