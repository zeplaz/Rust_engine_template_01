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

## Bevy 0.19 migration (`mig_bevy_019/`)

**Read first:** [`src/dev/plan_bevy_019_migration_v1.md`](../src/dev/plan_bevy_019_migration_v1.md) § **AGENT ROUTING** · [`HANDOFF.md`](../tools/orchestrator/queues/HANDOFF.md) § PLAN-BEVY-019-MIG-v1

**Authoritative witnesses:**

| File | Meaning |
|------|---------|
| [`mig_v1_gate.json`](mig_bevy_019/mig_v1_gate.json) | **P0–V1 complete** — `gate_pass: true` = Bevy 0.19 on master (not "Phase 0 blocked") |
| [`mig_a_rollup.json`](mig_bevy_019/mig_a_rollup.json) | **MIG-A slice status** — authoritative per-slice shipped/audit/defer |
| `compat_matrix_g1.json` | Ecosystem go/no-go (bevy_egui 0.41, etc.) |
| `baseline_stage5_pre019.json` / `post019.json` | Perf/readiness anchor |
| `feature_flag_audit_g3.json` | Cargo feature audit |
| `mig_a_a8_settings_coexistence_audit.json` | MIG-A8 — Settings vs shell_persistence (audit) |
| `mig_a_a9_bsn_scene_handoff.json` | MIG-A9 — **handoff complete** → city grammar § BSN ASSEMBLY CHARTER |
| `mig_a_a11_depth_prepass_audit.json` | MIG-A11 — custom Core2d pass inventory |
| `mig_a_a17_mesh_collection_audit.json` | MIG-A17 — RN-* batch metrics scaffold |
| `mig_a_frame_perf.json` | MIG-A18 — opt-in (`MIG_A18=1` or `PERF=1`) after frame 120 |

**Pick now (stable):** RTT/VFX operator visual — [`visual_run_blockers.md`](../src/dev/visual_run_blockers.md) · `--test vfx` · `--test visual`

**Defer (with reason):** [`plan_deferral_registry_v1.md`](../src/dev/plan_deferral_registry_v1.md) — morph (**DR-MIG-A15**), tilemap (**DR-MIG-TILEMAP**), BSN expansion (**DR-CITY-C6-BSN** — product, not migration). **MIG-A9 = closed_handoff.** Incremental MIG-A deep is **pick-now**.

## Regenerate

```powershell
cargo check -p proc_A_dine01
cargo test -p proc_A_dine01 --lib stage5
cargo run -p proc_A_dine01 --release -- --test vfx          # operator sandbox (manual VFX)
cargo run -p proc_A_dine01 --release -- --test visual --stay-open   # proof harness, no auto-exit
```
