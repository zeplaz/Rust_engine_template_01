# Plan slice report

Generated: epoch 1779508467 (orchestrator `--plan-slice`)

## Health

- Logistics witness not green — see logistics_throughput_live.json open_todos
- Default infra track: 5.5-A — start TRIAGE-VM-06 (view runtime sole writer)

## Witness digest

| Proof | Present | Green | Age (h) | Note |
|-------|---------|-------|---------|------|
| `debug_runs/stage5_full_app_live.json` | true | Some(true) | "1.4" | FULL_APP passes=true violations=0 |
| `debug_runs/infrastructure_view_isolation_live.json` | true | None | "0.0" | per-view isolation |
| `debug_runs/construction_stage_live.json` | true | None | "0.0" | construction boards |
| `debug_runs/industrial_activation_live.json` | true | Some(true) | "0.0" | industrial activation |
| `debug_runs/logistics_throughput_live.json` | true | Some(false) | "0.0" | open_todos=20 |
| `debug_runs/fire_ecology_live.json` | true | Some(true) | "0.0" | mean_heat=0.000 |
| `debug_runs/replay_editor_parity_live.json` | false | None | "—" | missing — run proof command for this lane |
| `debug_runs/main_thread_orchestrator_live.json` | true | None | "—" | orchestrator_shift |

## Recommended slices (continuation queue)

### SLICE-TRIAGE-VM-06 (P2)

- **Title:** Sole writer per `ViewId`; audit all pose paths
- **Track:** 5.5-A · **Lane:** VM · **Agent:** @coder
- **Source:** TRIAGE-VM-06
- **Witness:** `debug_runs/infrastructure_view_isolation_live.json`
- **Playbook:** `tools/orchestrator/agents/viewport_cleanup_agent.md`
- **Commands:**
  ```powershell
  cargo test -p proc_A_dine01 --lib
  ```
  ```powershell
  cargo orchestrate --plan-slice
  ```

### SLICE-TRIAGE-FIRE-STREAM (P3)

- **Title:** Active/sleep chunk streaming, neighbor wake, budgets
- **Track:** 5.5-E · **Lane:** Fire · **Agent:** @sim-steward
- **Source:** TRIAGE-FIRE-STREAM
- **Witness:** `debug_runs/infrastructure_view_isolation_live.json`
- **Playbook:** `tools/orchestrator/agents/render_pipeline_agent.md`
- **Commands:**
  ```powershell
  cargo test -p proc_A_dine01 --lib
  ```
  ```powershell
  cargo orchestrate --plan-slice
  ```

### SLICE-MD-F2-01 (P4)

- **Title:** Per-tile / hot-cell extract contract for GPU (`TRIAGE-FIRE-EXTRACT`)
- **Track:** 5.5-E · **Lane:** Fire · **Agent:** @coder
- **Source:** markdown:src/dev/fire_ecology_f1_todos.md
- **Witness:** `debug_runs/fire_ecology_live.json`
- **Playbook:** `tools/orchestrator/agents/render_pipeline_agent.md`
- **Commands:**
  ```powershell
  cargo test -p proc_A_dine01 fire:: --lib
  ```
  ```powershell
  cargo run -p proc_A_dine01 --release -- --test visual
  ```

### SLICE-MD-F2-02 (P4)

- **Title:** Smoke beyond stub (`chunk_smoke_field` + render)
- **Track:** 5.5-E · **Lane:** Fire · **Agent:** @coder
- **Source:** markdown:src/dev/fire_ecology_f1_todos.md
- **Witness:** `debug_runs/fire_ecology_live.json`
- **Playbook:** `tools/orchestrator/agents/render_pipeline_agent.md`
- **Commands:**
  ```powershell
  cargo test -p proc_A_dine01 fire:: --lib
  ```
  ```powershell
  cargo run -p proc_A_dine01 --release -- --test visual
  ```

### SLICE-MD-F2-03 (P4)

- **Title:** Fuel-linked spread (ember + neighbor fuel depletion)
- **Track:** 5.5-E · **Lane:** Fire · **Agent:** @coder
- **Source:** markdown:src/dev/fire_ecology_f1_todos.md
- **Witness:** `debug_runs/fire_ecology_live.json`
- **Playbook:** `tools/orchestrator/agents/render_pipeline_agent.md`
- **Commands:**
  ```powershell
  cargo test -p proc_A_dine01 fire:: --lib
  ```
  ```powershell
  cargo run -p proc_A_dine01 --release -- --test visual
  ```


Open triage rows parsed: **27** · Open markdown todos: **7**
