# `debug_runs/` — live proof JSON for agents

Each `*_live.json` file is rewritten during simulation or `--test visual`. Newer files include **`_agent_meta`** (schema `debug_run_envelope_v1`):

| Field | Purpose |
|-------|---------|
| `written_at_epoch_secs` | Freshness check |
| `logging_env` | Which trace env vars were set when written |
| `agent_commands` | Suggested `cargo` / `RUST_LOG` commands |
| `related_proofs` | Cross-links to sibling witness files |
| `orchestrator` | Paths to `tools/orchestrator/reports/*` |

**Indexes:**

| File | Scope |
|------|--------|
| [`agent_debug_index.json`](agent_debug_index.json) | Sim spine + art anchor proofs (Rust refresh on proof write) |
| [`unified_witness_index.json`](unified_witness_index.json) | Full sim + art rollup (OPS Track D) |
| [`agent_ops/ops_report_latest.json`](agent_ops/ops_report_latest.json) | DSM + Q/C/E + ΔWF + `program_summary` |

**Programs:** stage5 · fire_vfx · construction · infrastructure · economy · wave · stage7 · ui · art A/B/C — see [`../tools/orchestrator/queues/OPS_LANE_REGISTRY.json`](../tools/orchestrator/queues/OPS_LANE_REGISTRY.json)

**Contract:** $ref:docs/archive/2026-06-src-dev/plans/witness_exec_shape_v1.md — tensor AUTH from scan, not hardcoded ○ nodes.

Refresh: `powershell -File tools/orchestrator/scripts/ops_intelligence_scan.ps1` · Contract: [`../tools/orchestrator/queues/OPS_WITNESS_SPINE.md`](../tools/orchestrator/queues/OPS_WITNESS_SPINE.md)

## Primary files

| File | Lane |
|------|------|
| `stage5_full_app_live.json` | FULL_APP readiness + viewport + fire playback |
| `infrastructure_view_isolation_live.json` | VM-A view isolation |
| `construction_stage_live.json` | Construction todo boards + history |
| `industrial_activation_live.json` | Economy activation |
| `logistics_throughput_live.json` | Logistics throughput |
| `orchestrator_thread_health.json` | `ORCHESTRATOR_EXPORT_HEALTH=1` only |

## Stage 5 closure

- **Checklist (gate):** [`docs/archive/2026-06-src-dev/plans/stage5_close_checklist.md`](../docs/archive/2026-06-src-dev/plans/stage5_close_checklist.md)
- **Deferred / sticky:** [`src/dev/stage5_triage_backlog.md`](../src/dev/stage5_triage_backlog.md)
- Live JSON includes `stage5_closure` when written by visual harness

## Regenerate

```powershell
cargo test -p proc_A_dine01 --lib stage5
cargo run -p proc_A_dine01 --release -- --test visual
```

`--test visual` uses **tactical** map zoom by default so `fire_spark_rows` and `water_particle_river_streaks` are not strategic-culled. Optional strict gate: `$env:TACTICAL_VFX_PROOF=1`.
