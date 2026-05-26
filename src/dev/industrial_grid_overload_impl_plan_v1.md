# Industrial grid overload — implementation plan `v1` (PLAN-IND-E03-001)

| Field | Value |
|:---|:---|
| **Queue ID** | **PLAN-IND-E03-001** |
| **Coder slice** | **IND-E03-CODER-A** |
| **Version** | `1.0.0` |
| **Date** | 2026-05-25 |
| **Owner** | `@planner` (plan) · **execute:** `@coder` |
| **Status** | **CLOSED** — implementation + lib witness **DONE** |
| **Board row** | **INDUSTRIAL-I3-02** — grid/substation stress |
| **Parent** | [`industrial_activation_pipeline.md`](industrial_activation_pipeline.md) § Priority 3 |
| **Phase todos** | [`industrial_activation_phase_todos.md`](industrial_activation_phase_todos.md) § I3 |
| **Witness** | [`debug_runs/industrial_activation_live.json`](../../debug_runs/industrial_activation_live.json) → `ind_e03` |

**No new gameplay in this doc.** Rollup for **IND-E03-CODER-A**: wire `GridOverloadEvent` into industrial activation proof + optional visual seed cluster.

---

## Executive summary

| Slice | Verdict |
|:---|:---|
| **IND-E03-CODER-A** — overload witness + seed cluster | **DONE** |
| **INDUSTRIAL-I3-02** board predicate | **DONE** |
| **IND-E03** post-Stage-6 exit | **DONE** — `ind_e03_green` |
| **S7P-IND-002** / Stage 7 play optional step 8 | **DONE** — mirrored in `stage7_play_live.json` |
| **Real smelter→grid load via activation** | **DEFERRED** — witness uses deterministic cluster |
| **Player-facing overload UX** | **DONE** — [`s7p_grid_overload_ux_note_v1.md`](s7p_grid_overload_ux_note_v1.md) (**S7P-DESIGN-002**) |

---

## Problem statement

Industrial activation must prove **Priority 3** from the pipeline: facilities with `ElectricalComponent` participate in grid rebuild; when aggregate load exceeds host capacity, simulation emits **`GridOverloadEvent`** and proof JSON records it.

**Not in scope for IND-E03:** placing `grid_substation.json` via construction (**INDUSTRIAL-I3-03/04**), brownout damage persistence UI, or petroleum/logistics coupling.

---

## Gate chain

```text
INDUSTRIAL-I1-* / IND-E01 concrete E2E          ☑ production_green
        │
        ▼
PowerRuntimePlugin + grid_topology              ☑ (existing)
        │
        ▼
ResourceFlowPlugin::collect_grid_overload       ☑ overload_events_total
        │
        ▼
IND-E03-CODER-A — witness cluster + seed        ☑
        │
        ▼
industrial_activation_live.json                 ☑ ind_e03_green
        │
        └─► stage7_play_live (optional)         ☑ s7p_grid_optional_green
```

---

## Architecture (authority map)

```text
rebuild_electrical_grid_topology
        → recalculate_grid_totals_from_members
        → emit_grid_overload_signals (GridOverloadEvent)
        → collect_grid_overload_witness_system (ResourceFlowSimWitness)
        → refresh_industrial_activation_witness_system (grid_overload_hook)
        → write_industrial_activation_live_proof_system (ind_e03 block)
```

| Module | Role |
|:---|:---|
| [`grid_topology.rs`](../entities/production/power/grid_topology.rs) | Membership radius, overload emit |
| [`resource_flow.rs`](../economy/resource_flow.rs) | `overload_events_total` counter |
| [`bridge.rs`](../economy/activation/bridge.rs) | `grid_overload_hook` from witness resource |
| [`concrete_chain_e2e.rs`](../economy/activation/concrete_chain_e2e.rs) | `spawn_ind_e03_grid_overload_cluster`, seed systems |
| [`live_proof.rs`](../economy/activation/live_proof.rs) | JSON `ind_e03` + lib tests |

**Schedule (production app):** `IndustrialActivationPlugin` chains `seed_ind_e03_grid_overload_witness_once` **after** `collect_grid_overload_witness_system`.

---

## IND-E03-CODER-A deliverables (**CLOSED**)

### E03-01 — Overload event path

| # | Task | Status |
|:---:|:---|:---:|
| 1 | `GridOverloadEvent` message registered (`PowerRuntimePlugin` + `ResourceFlowPlugin`) | **DONE** |
| 2 | `emit_grid_overload_signals` when `total_load > total_capacity` | **DONE** |
| 3 | `collect_grid_overload_witness_system` increments `overload_events_total` | **DONE** |

### E03-02 — Witness flags

| Field | Predicate |
|:---|:---|
| `grid_overload_hook` | `grid_topology.rs` exists **and** `overload_events_total > 0` |
| `grid_overload_sim_green` | same as hook in live JSON |
| `industrial_i3_02_green` | `grid_overload_hook` **and** `overload_events_total >= 1` |
| `ind_e03_green` | `production_green` **and** `grid_membership` **and** `grid_overload_hook` |

**Board:** `INDUSTRIAL-I3-02` → **Done** when `grid_overload_hook` true ([`industrial_activation_todos.rs`](industrial_activation_todos.rs)).

### E03-03 — Deterministic overload cluster

**Function:** `spawn_ind_e03_grid_overload_cluster(commands, origin)`

| Entity | Components | Load intent |
|:---|:---|:---|
| Host | `TransformerComponent`, `ElectricalGrid`, `ElectricalComponent` (capacity **2.0**) | Substation bus |
| 4× members | `Building` + `ElectricalComponent` (current_load **2.0**, capacity **0**) | Exceed host when summed |

Mirrors integration test in [`bridge.rs`](../economy/activation/bridge.rs) `tests::overload_cluster_exceeds_transformer_capacity` (pattern name paraphrased).

### E03-04 — Visual / play seed (optional operator)

| Env / launch | Effect |
|:---|:---|
| `RUST_ENGINE_IND_E03_SEED=1` | One-shot cluster after `production_green` |
| `RUST_ENGINE_STAGE7_PLAY_SEED=1` | Same (Stage 7 play bundle) |
| `--test visual` + `full_capture_active()` | Same via launch args |

**Systems:**

- `reset_ind_e03_grid_overload_seed_on_enter_simulation` — `OnEnter(Simulation)`
- `seed_ind_e03_grid_overload_witness_once` — spawns cluster, waits for `overload_events_total > 0`, sets `seed.seeded`

**Prereq:** Portland / concrete chain `production_green` before cluster spawn (avoids racing empty sim).

---

## Witness bundle (fleet truth 2026-05-25)

| File | Keys |
|:---|:---|
| `industrial_activation_live.json` | `grid_overload.*` (**IND-E03-WITNESS-001**), `ind_e03` (mirror), `industrial_i3_02_green` |
| `stage7_play_live.json` | `grid_overload_hook`, `overload_events_total`, `s7p_grid_optional_green` |

Example (live proof):

```json
"grid_overload": {
  "grid_overload_hook": true,
  "grid_overload_sim_green": true,
  "grid_membership": true,
  "production_green": true,
  "ind_e03_green": true,
  "green": true,
  "ind_e03_witness_001_green": true,
  "overload_events_total": 30
},
"ind_e03": { "...": "same object as grid_overload" }
```

---

## Regression

```powershell
cargo test -p proc_A_dine01 --lib industrial_activation
```

| Test | Covers |
|:---|:---|
| `simulation_writes_industrial_activation_live_json` | `industrial_i3_02_green` + board **INDUSTRIAL-I3-02** |
| `simulation_writes_industrial_activation_live_json_i3_02_grid_overload` | **IND-E03** full predicate |
| `bridge` overload integration | `GridOverloadEvent` count > 0 |

**Operator (visual depth):**

```powershell
$env:RUST_ENGINE_IND_E03_SEED = "1"
cargo run -p proc_A_dine01 --release -- --test visual
```

---

## Copy-paste — IND-E03-CODER-A (archive — done)

```
Lane: IND-E03-CODER-A — grid overload witness depth
Read: src/dev/industrial_grid_overload_impl_plan_v1.md
      src/dev/industrial_activation_pipeline.md § Priority 3
Prereq: ResourceFlowPlugin + PowerRuntimePlugin in proof harness
First: collect_grid_overload_witness_system after emit_grid_overload_signals
Then: spawn_ind_e03_grid_overload_cluster + ind_e03 JSON block in live_proof.rs
Max files: 4 — concrete_chain_e2e.rs, resource_flow.rs, bridge.rs, live_proof.rs
Do NOT: stage5 FULL_APP spine; minimap extract; fake overload without GridOverloadEvent
Verify: cargo test -p proc_A_dine01 --lib industrial_activation
Witness: ind_e03_green: true, overload_events_total >= 1
```

---

## Copy-paste — follow-on (not IND-E03)

### INDUSTRIAL-I3-03/04 — transformer catalog placement

```
Lane: INDUSTRIAL-I3-03/04 — utilities JSON → activation spawn
Read: industrial_activation_pipeline.md § I3
Files: assets/configs/buildings/grid_*.json, bridge activate path
Do NOT: duplicate IND-E03 witness cluster as “production complete”
```

### S7P-DESIGN-002 — overload UX (designer only)

```
Lane: S7P-DESIGN-002 — smelter overload player feedback
Read: prompts/designer_questions/production_economy/power_damage_ui_persistence_v1.md
Deliver: one paragraph — toast vs tray vs diagnostics
No Rust
```

---

## Forbidden

| Pattern | Reason |
|:---|:---|
| Set `grid_overload_hook` without `overload_events_total` | False green |
| Skip `emit_grid_overload_signals` ordering | Events never collected |
| Mutate `RepresentationResult` / Stage 5 spine | Industrial lane only |
| Collapse Portland chain into single mega-factory | **GOV** witness |
| Re-open IND-E03 for **UI-P3** / minimap work | Disjoint lanes |

---

## Open tails (honest)

| ID | Goal | Owner |
|:---|:---|:---|
| **INDUSTRIAL-I3-03** | `grid_distribution_transformer.json` placeable | coder |
| **INDUSTRIAL-I3-04** | Activation spawns transformer from catalog | coder |
| **INDUSTRIAL-I3-05** | Capacity bottleneck gameplay (not decorative) | coder + design |
| ~~**S7P-DESIGN-002**~~ | Overload toast/tray copy | **DONE** — design note on disk |
| **IND-E03+** | Smelter `ElectricalComponent` from real activation loads | coder (future) |

---

## Sign-off

| Role | Date | Status |
|:---|:---|:---|
| Planner | 2026-05-25 | **SIGNED** — PLAN-IND-E03-001 |
| Coder IND-E03-CODER-A | 2026-05-25 | **CLOSED** |
| Steward S7P | 2026-05-25 | **INDUSTRIAL-I3-02** in [`steward_s7p_gate_v1.md`](steward_s7p_gate_v1.md) |

---

## Changelog

| Version | Date | Notes |
|:---|:---|:---|
| v1.0.0 | 2026-05-25 | IND-E03-CODER-A impl rollup; witness + seed documented |
