# Coder fleet — multi-stage matrix with fallbacks `v1`

| Field | Value |
|:---|:---|
| **Queue ID** | **CODER-FLEET-MULTISTAGE-001** |
| **Version** | `1.0.0` |
| **Date** | 2026-05-26 |
| **Machine queue** | [`tools/orchestrator/queues/coder_active_queue.json`](../tools/orchestrator/queues/coder_active_queue.json) |
| **Orders** | [`wssr_coder_hybrid_orders_v1.md`](wssr_coder_hybrid_orders_v1.md) · construction [`plan_construction_param_exec_phases_v1.md`](plan_construction_param_exec_phases_v1.md) |
| **Rule** | **One primary per session** · pick **fallback** from same track when blocked · **never** two mutex files same cycle |

**Purpose:** Keep **construction** and **WSS/VFX terrain** both moving with explicit alternation and escape hatches.

> **Clarification (2026-05-27):** Parametric construction stages `B-C1..B-C6` are witness-closed (`construction_parametric_placement_001.green: true`).  
> Treat B-C rows below as historical execution map; use `coder_active_queue.json` + `stage_coder_workboard_v1.md` for current picks.

**Parallel:** [@planner](planner_parallel_workboard_v1.md) · [@designer](designer_parallel_workboard_v1.md) prep **next** slices while coders run — see § Sync. **Do not** re-sign closed WSS/PARAM baselines.

---

## Three parallel tracks (never collapse)

| Track | Color | Owner bias | Witness root | Must preserve |
|:---|:---|:---|:---|:---|
| **CONSTRUCTION** | build | **Coder B** | `construction_stage_live.json` | invariants · parametric witness |
| **WSS substrate** | terrain/sim spine | **Coder A** | `wss_substrate_live.json` | hybrid ECS · no sim-from-GPU |
| **VFX / extract** | fire·water·presentation | **Coder A** (extract) · **B** (UI) | `fire_ecology` · `stage5` · `minimap` | F7 exit · water W1/W2 closed |

```text
         ┌─ CONSTRUCTION (B) ─── PARAM-002→006 ─── always has UI fallback (M3/replay)
         │
CYCLE ───┼─ WSS (A + B hydro) ─ slab → atmos ∥ hydro → PR-2 shim
         │
         └─ VFX (A + B) ─── F2 extract · F7 debug · smoke bridge · M3/replay
```

---

## File mutex (hard — pick other track if hot)

| Path / domain | Track | Other track must wait |
|:---|:---|:---|
| `src/construction/*` | CONSTRUCTION | WSS — |
| `src/substrate/*` | WSS | CONSTRUCTION — |
| `src/construction/visual_authority.rs` | CONSTRUCTION | **R4-MV-GHOST** deferred |
| `src/render/fire_view_extract.rs` | VFX | WSS atmos (coordinate) |
| `src/gui/minimap/*` | PRODUCT (B) | — |
| `src/strategic/site/tile_occupation.rs` | CONSTRUCTION | — |

---

## Coder A — stages + fallbacks

### Track WSS (substrate spine)

| Stage | ID | Goal | Witness keys | If stuck / fail → |
|:---:|:---|:---|:---|:---|
| **A-W1** | **WSS-CHUNK-SLAB-001** | Finish CS-001..006: hydrate, paging, witness green | `slab_registry_present`, `hydrate_wired`, `runtime_writer` | **A-V1** F7-DEBUG-WIRE |
| **A-W2** | **WSS-ATMOS-CLIPMAP-001** | Clipmap types + L1 bridge; no delete 128² yet | `wss_atmos_clipmap_001.*` | **A-V2** FIRE-F2-EXTRACT |
| **A-W3** | **WSS-SLAB-PR-2** (future) | Dual-write shim weather/fire ↔ slab | `dual_write_drift_max` | **A-V3** smoke stub bridge |
| **A-W4** | **WSS-SMOKE-BRIDGE-001** | `fire_visual_emit_smoke_stub` → extract | `smoke_stub_removed` | **A-V2** |

**A-W1 partial landed:** `src/substrate/` exists — **complete** per [`plan_wss_chunk_slab_exec_001_v1.md`](plan_wss_chunk_slab_exec_001_v1.md), do not restart.

### Track VFX (extract — parallel escape)

| Stage | ID | Goal | Witness | If stuck → |
|:---:|:---|:---|:---|:---|
| **A-V1** | **F7-DEBUG-WIRE-001** | F3 overlay labels | `fire_streaming_live.json` | **A-V4** S7B-M4-LIVE |
| **A-V2** | **FIRE-F2-EXTRACT-001** | `fire_instance_buffer_rows > 0` | [`plan_fire_f2_extract_exec_001_v1.md`](plan_fire_f2_extract_exec_001_v1.md) · stage5 | **A-V1** |
| **A-V3** | **WSS-SMOKE-BRIDGE-001** | `smoke_stub_removed` | [`plan_wss_smoke_bridge_exec_001_v1.md`](plan_wss_smoke_bridge_exec_001_v1.md) | **A-W2** |
| **A-V4** | **S7B-M4-LIVE-001** | Live play enqueue | `stage7_behavioral_live.json` | infra S4 optional |

**Regression every A session:** `cargo test -p proc_A_dine01 --lib stage5 fire_streaming wss_substrate`

---

## Coder B — stages + fallbacks

### Track CONSTRUCTION (always open — never empty)

| Stage | ID | Goal | Witness flags | If stuck → |
|:---:|:---|:---|:---|:---|
| **B-C1** | **CONSTRUCTION-PARAM-CODER-002** | P2-A: Enter commit; no Shift queue (buildings) | `shift_queue_building_removed`, `enter_commits_single_ghost` | **B-P1** |
| **B-C2** | **CONSTRUCTION-PARAM-CODER-003** | P1-B: `TileOccupationBook` | `overlap_blocks_commit` | **B-C1** retry smaller |
| **B-C3** | **CONSTRUCTION-PARAM-CODER-005** | P2-B: partial-alpha ghosts | overlap visual | **B-P1** |
| **B-C4** | **CONSTRUCTION-PARAM-CODER-004** | P3-A: staging panel | `staging_toggle_wired`, `build_approved_drains_staged` | **B-C5** economy first |
| **B-C5** | **CONSTRUCTION-PARAM-CODER-006** | P4-A: economy scale | `economy_scales_at_activation` | **B-P2** |
| **B-C6** | **PARAM rollup** | `construction_parametric_placement_001.green` | all booleans | — |

**Done:** CODER-000 scaffold · CODER-001 `weighted_footprint.rs` ☑

**Deferred:** **R4-MV-GHOST-001** until **B-C3** lands (same `visual_authority.rs`).

### Track WSS-HYDRO (after slab hydrate)

| Stage | ID | Goal | Witness | If stuck → |
|:---:|:---|:---|:---|:---|
| **B-H1** | **WSS-HYDRO-RUNTIME-001** | HY-001..004 hydrate + tick stub | `wss_hydro_runtime_001` | **B-P1** |
| **B-H2** | **WSS-HYDRO-BOUNDARY-001** | Construction → `HydrologyDirtyReason` only | coupling witness | **B-C2** |

**Blocked until:** `wss_substrate_live.json` → `hydrate_wired: true`

### Track PRODUCT (disjoint — always available fallback)

| Stage | ID | Goal | Witness | When to use |
|:---:|:---|:---|:---|:---|
| **B-P1** | **M3-UNITS-DEPTH-001** | Real unit marker reader | `minimap_compositor_live.json` | Construction blocked on design/merge |
| **B-P2** | **REPLAY-RING-LIVE-001** | Live replay ring | `replay_editor_parity_live.json` | After M3 or instead |
| **B-P3** | **UI-P3-M2-TRAY-OPT** | Tray → `MinimapOverlayMask` | minimap witness | Low risk filler |

```powershell
cargo test -p proc_A_dine01 --lib construction minimap_compositor
```

---

## 12-cycle weave (recommended alternation)

Pick **one cell per session** (primary). If primary blocked >30 min, switch to **Fallback** same cycle.

| Cycle | Coder A primary | Coder A fallback | Coder B primary | Coder B fallback |
|:---:|:---|:---|:---|:---|
| 1 | A-W1 finish slab | A-V1 F7-debug | B-C1 PARAM-002 | B-P1 M3 |
| 2 | A-W2 atmos clipmap | A-V2 F2 extract | B-C2 PARAM-003 occupation | B-P1 |
| 3 | A-V2 F2 extract | A-W2 | B-C3 PARAM-005 visual | B-H1 *if hydrate* |
| 4 | A-W2 (cont) | A-V3 smoke | B-C1 polish / tests | B-P2 replay |
| 5 | A-W3 dual-write plan impl | A-V2 | B-C4 staging panel | B-P3 tray |
| 6 | A-V3 smoke bridge | A-W3 | B-C4 (cont) | B-H1 hydro |
| 7 | A-W4 smoke complete | A-V1 | B-C5 economy | B-C6 witness rollup |
| 8 | A-V2 hardening | A-W3 | B-C6 green | B-P2 |
| 9 | A-W2 render clipmap slice | A-V2 | B-H2 hydro coupling | B-P1 |
| 10 | Integration pass | steward regression | B-C6 manual playtest | operator visual |
| 11 | A-V4 S7B M4 | A-W2 | **R4-MV-GHOST** *if C3 done* | B-P1 |
| 12 | Buffer / tech debt | CW-50 hygiene | Construction bugfix | witness refresh |

**Weave rule:** After **2 consecutive** sessions on same track for one coder, **switch track** (e.g. A: WSS→VFX→WSS).

---

## Checkpoint gates (witness-driven)

| Gate | JSON path | Unblocks |
|:---|:---|:---|
| Slab spine | `wss_substrate_live.json` → `wss_chunk_slab_001` green | A-W2, B-H1 |
| Hydrate | `hydrate_wired: true` | B-H1, B-H2 |
| Param input | `construction_parametric_placement_001` → `enter_commits_single_ghost` | B-C3 staging |
| Param visual | `overlap_blocks_commit` + partial alpha | B-C4, then R4-MV |
| F2 extract | `stage5` → `fire_instance_buffer_rows > 0` | A-W3 smoke, planner F2-02 |
| Atmos bridge | `wss_atmos_clipmap_001.green` | WSS-SIM weather v2 planner |

---

## Done — regression only

F7-STREAM-DEEP · F7-A/B/C exit · R4-CORRIDOR · wave 3 bundles · water W1/W2 closure

---

## Planner / designer sync (parallel + checkpoints)

### Always-on parallel (while coders work)

| Role | Board | Focus |
|:---|:---|:---|
| **@planner** | [`planner_parallel_workboard_v1.md`](planner_parallel_workboard_v1.md) | F2 exec · PARAM P3/P4 · weather v2 · WSS PR-2 draft · Hanabi charter |
| **@designer** | [`designer_parallel_workboard_v1.md`](designer_parallel_workboard_v1.md) | Contamination · smoke A/B · param staging/HUD · hydro read · wave 6 PASS |

### Witness checkpoints (sign/amend plans)

| Checkpoint | Planner | Designer |
|:---|:---|:---|
| `wss_chunk_slab_001.green` | Sign **PLAN-WSS-SLAB-PR-2-EXEC** | **DESIGN-WSS-DIAGNOSTICS-PASS-002** |
| `wss_atmos_clipmap_001` started | Sign **WEATHER-SIM-PLAN-001** v2 | **DESIGN-CONTAMINATION-001** done |
| `fire_instance_buffer_rows > 0` | **PLAN-FIRE-F2-02** smoke | **DESIGN-F2-EXTRACT-READ-001** |
| `construction_parametric_placement_001.green` | R4 product open policy | **DESIGN-R4-MV-POST-PARAM-001** |
| Hanabi spike report | **PLAN-HANABI-ADOPTION-001** | **DESIGN-HANABI-BOUNDS-001** |
| `dual_write_drift` > ε | PR-3 exec | **DESIGN-DUAL-WRITE-UX-001** |

**Do not** re-open: WSS-PLAN-001..004 sign-off · PARAM product spec · elemental charter baselines.

---

## Session prompt template (both coders)

```
Session: Cycle N — primary [ID]
Fallback if blocked: [ID]
Hybrid Assessment: (required for WSS/construction slices)
Mutex check: I am NOT touching [forbidden paths]
Exit: witness field [x] = true OR explicit defer note in HANDOFF
Regression: cargo test -p proc_A_dine01 --lib [lane tests]
```

---

## Changelog

| Version | Date | Notes |
|:---|:---|:---|
| v1.0.0 | 2026-05-26 | Multi-stage matrix; construction + WSS + VFX woven; planner/designer deferred to checkpoints |
