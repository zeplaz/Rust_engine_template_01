# PLAN-OPS-WITNESS-CADENCE-001 — Operator witness refresh cadence `v1`

| Field | Value |
|:---|:---|
| **Queue ID** | **PLAN-OPS-WITNESS-CADENCE-001** |
| **Prior** | [`operator_ops_witness_refresh_plan_v1.md`](operator_ops_witness_refresh_plan_v1.md) (OPS-F01/F03) · [`operator_visual_signoff_bundle_plan_v1.md`](operator_visual_signoff_bundle_plan_v1.md) |
| **Steward** | [`steward_witness_sync_gate_v1.md`](steward_witness_sync_gate_v1.md) |
| **Index** | [`debug_runs/README.md`](../../debug_runs/README.md) · `debug_runs/agent_debug_index.json` |
| **Version** | `1.0.0` |
| **Date** | 2026-05-27 |
| **Owner** | `@planner` |
| **Status** | **READY (planner finalized)** |

**Planner sign-off:** PASS (2026-05-27). Operator playbook — no Rust.

---

## Summary

When **`@operator`** (or release steward) should refresh which `debug_runs/*_live.json` files, versus when **lib tests** suffice. Witness JSON **wins** over markdown checkboxes.

---

## Cadence matrix

| Trigger | Who | Action | Required artifacts |
|:---|:---|:---|:---|
| **Daily dev** (no merge) | — | None mandatory | Lib regression only |
| **Post-merge to `master`** | @operator or CI | Lib bundle + spot witnesses | See § Post-merge bundle |
| **After WSS substrate coder slice** | @operator | Substrate + stage5 spot | `wss_substrate_live.json`, `stage5_full_app_live.json` |
| **After construction coder slice** | @operator | Construction witness | `construction_stage_live.json` |
| **After minimap/replay slice** | @operator | Minimap + parity | `minimap_compositor_live.json`, `replay_editor_parity_live.json` |
| **Pre-release / demo** | @operator | Full visual bundle | § Visual bundle |
| **Qualified VFX upgrade** | @operator optional | `--test visual` | Clears `visual_run_pending` |
| **Stage 6 ops tail** | @operator optional | Sim session | `stage6_virtualization_live.json` |

---

## Post-merge bundle (minimum)

Run after any merge touching `src/render/`, `src/substrate/`, `src/construction/`, `src/gui/`, or `src/systems/`:

```powershell
cargo test -p proc_A_dine01 --lib stage5
cargo test -p proc_A_dine01 --lib wss_substrate construction minimap_compositor replay_editor_parity
cargo test -p proc_A_dine01 --lib coder_a_wave3 coder_b_wave3
```

| File | Refresh method | Freshness check |
|:---|:---|:---|
| `wss_substrate_live.json` | sim writer or `cargo test -p proc_A_dine01 --lib wss_substrate` | `_agent_meta.written_at_epoch_secs` |
| `construction_stage_live.json` | `cargo test -p proc_A_dine01 --lib simulation_writes_construction_stage_live_json` | same |
| `minimap_compositor_live.json` | `cargo test -p proc_A_dine01 --lib minimap_compositor_live_witness_refresh` | same |
| `replay_editor_parity_live.json` | `cargo test -p proc_A_dine01 --lib replay_editor_parity` | `parity_green` |
| `stage5_full_app_live.json` | lib stage5 tests **or** `--test visual` | `readiness.passes` |
| `agent_debug_index.json` | auto on any proof write | index timestamp |

**Qualified rule:** Lib refresh is **sufficient** for fleet close unless product requests pixel/visual upgrade.

---

## WSS substrate cadence

| Event | Command | Keys to verify |
|:---|:---|:---|
| Slab / PR-2 / PR-3 merge | `cargo test -p proc_A_dine01 --lib wss_substrate` | `green`, `hydrate_wired`, `gate` |
| Hydro runtime merge | same + hydro lib tests | `wss_hydro_runtime_001.green` |
| Atmos clipmap merge | same | `wss_atmos_clipmap_001.green` |
| Dual-write drift alert | steward review | `dual_write_drift_max` < ε |

**Do not hand-edit** `wss_substrate_live.json`.

---

## Construction cadence

| Event | Command | Keys to verify |
|:---|:---|:---|
| Construction / parametric / R4 merge | `cargo test -p proc_A_dine01 --lib simulation_writes_construction_stage_live_json` | `operational_green`, lane blocks |
| Parametric only | `cargo test -p proc_A_dine01 --lib construction` | `construction_parametric_placement_001.green` |
| R4 corridor/MV | same witness test | `construction_r4_corridor_001`, `construction_r4_mv_ghost_001` |

---

## Stage 5 cadence

| Tier | Command | When |
|:---|:---|:---|
| **Lib (default)** | `cargo test -p proc_A_dine01 --lib stage5` | Every post-merge bundle |
| **Visual (optional)** | `cargo run -p proc_A_dine01 --release -- --test visual` | Pre-release; VFX qualified upgrade; steward epoch sync |
| **Strict VFX** | `$env:TACTICAL_VFX_PROOF=1` + visual run | Only when triaging VR-* blockers |

**Align epochs:** After visual run, `stage5_full_app_live.json` should be newest among sibling proofs (see steward sync gate).

---

## Visual bundle (optional single session)

From [`operator_visual_signoff_bundle_plan_v1.md`](operator_visual_signoff_bundle_plan_v1.md):

```powershell
cargo run -p proc_A_dine01 --release -- --test visual
```

PASS: exit 0, `readiness.passes`, `vfx_visual_signoff_001.green`, logistics rows > 0.

---

## Do not re-run (regression-only)

| Lane | Reason |
|:---|:---|
| F7-A/B/C exit gates | Closed — infra JSON only |
| Wave 3 dual-queue closure bundles | Historical |
| R4/M3/replay **planner** re-plan | Archived exec docs |
| Steward preflights (VM-09, S7P, UI-SHELL) | Unless witness row flips red |

---

## OPS tails (unchanged)

| ID | Cadence | Artifact |
|:---|:---|:---|
| **OPS-F01** | On perf investigation | `debug_runs/perf_attribution_60s.md` |
| **OPS-F03** | Monthly or pre-demo | `stage6_virtualization_live.json` via sim |
| **VFX-CAPTURE-INSIM-001** | Product request | `assets/vfx/reference/review_captures/` |

---

## Coder handoff

| Field | Value |
|:---|:---|
| **Unblocks** | @operator runbooks; steward epoch sync |
| **Acceptance** | Operator can follow matrix without reopening archived planner exec plans |
