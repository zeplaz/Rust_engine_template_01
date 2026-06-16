# Fire sim Phase 7 — architecture `v1` (FIRE7-PLAN-001)

| Field | Value |
|:---|:---|
| **Queue ID** | **FIRE7-PLAN-001** |
| **Track** | **FIRE-P7** — [`stages/fire_sim_phase7_plan_v1.md`](stages/fire_sim_phase7_plan_v1.md) |
| **Version** | `1.1.0` |
| **Date** | 2026-05-25 |
| **Owner** | `@planner` |
| **Status** | **SIGNED** — **FIRE7-PREFLIGHT GO** [`steward_fire7_preflight_gate_v1.md`](steward_fire7_preflight_gate_v1.md) |
| **F1 baseline** | [`fire_ecology_f1_todos.md`](fire_ecology_f1_todos.md) |
| **VFX closure** | [`fire_spark_track_closure_plan_v1.md`](fire_spark_track_closure_plan_v1.md) |

**No Rust in this deliverable.**

---

## North star

Per-view fire visibility, streaming sleep/wake, and LOD tiers are **bounded and authoritative** — ecology F1 stays the sim truth; Phase 7 scales **presentation** without a second global extract.

**Not a Stage 5 gate.** FULL_APP may stay green while F7 waves land incrementally.

---

## Authority map

| Domain | Sole writer | Readers | Notes |
|:---|:---|:---|:---|
| Sim fire state | `FireSimulationSnapshot` / `FireChunkRuntime` | sim systems | gameplay truth |
| Active set | `ActiveFireChunkSet` | visibility + streaming | sim tick |
| Per-view visible chunks | `VisibleFireChunkSet` | `fire_view_extract` | **one writer** per frame after view resolve |
| Render frames | `FireVisualFramesByView` | `fire_visual_extract`, projection graph, GPU particles | **no ECS fire reads** in extract |
| Tactical rollup | `tactical_fire_visual()` | legacy single-frame consumers | `WorldMain` → `SimulationMap` fallback |
| Stage 6 residency | `PerViewResidencyConsumerWindow` | intersect visible ∩ residency | before extract publish |
| Minimap / overlay | `SharedOverlayFieldBuffers` + compositor | M1 fire heat | **no** second fire ECS extract for minimap |

**Forbidden:** Second global `FireVisualFrame` extract; minimap querying fire ECS directly; `MapCameraDesired` as fire cull authority (use `ViewManager` + `ViewId`).

---

## Schedule (no cycles)

```text
CoreSystemSet::Sim
  → fire sim tick (ActiveFireChunkSet)
  → (F7-B) streaming sleep/wake mutates residency hints
  → ViewAuthority::SyncViewManager
  → fill VisibleFireChunkSet per ViewId (fire_view_extract)
  → intersect_visible_chunks_with_residency_window (Stage 6)
  → build FireVisualFramesByView
  → publish_stage6_virtualization_frame   (after frames stable)
  → run_render_projection_graph           (fire nodes consume by_view)
  → fire_visual_extract / GPU dispatch    (existing spine)
```

**Ordering rule:** `publish_stage6_virtualization_frame` runs **after** `FireVisualFramesByView` is populated for the frame, **before** render graph nodes that assume frame counts.

**Do not** insert fire extract before `ViewportAuthority` resolve or parallel to `fill_logistics_snapshot` writer.

---

## LOD table (F7-C target)

| Band | `WorldLodBand` / view | Player read | Instance cap (policy) | Overlay channel |
|:---|:---|:---|:---:|:---|
| **Strategic** | far / minimap | Heat blobs only | low (e.g. 32) | compositor fire heat |
| **Operational** | mid | Cluster caps | medium (e.g. 128) | heat + sparse sparks |
| **Tactical** | `WorldMain` near | Instances + sparks | high (policy cap) | full `FireVisualFrame` |
| **Cinematic** | dev / replay | Full local detail | budgeted by `FireChunkLodState` | VFX track |

**Clamp rule:** `ViewRenderPolicy` + `WorldLodBand` clamp `FireLodBand` per view ([`fire_view_extract.rs`](../render/fire_view_extract.rs) module intent).

---

## Gate chain — what **FIRE7-PLAN-001** blocks

**FIRE7-PLAN-001** is **planning only** (this doc). It **does not** close F7-B or F7-C.

| Milestone | Unblocks | Does **not** substitute for |
|:---|:---|:---|
| **FIRE7-PLAN-001 SIGNED** | **F7-A** coder work (architecture + forbidden list) | F7-B/C implementation |
| **F7-A-001 CLOSED** | **Real** F7-B streaming + **real** F7-C LOD wiring | Witness JSON alone |
| **F7-B CLOSED** | F7-C band caps tied to live streaming state | `fire_streaming_live.json` stub |
| **F7-C CLOSED** | Phase 7 product depth exit | Stage 5 FULL_APP (orthogonal) |

```text
FIRE7-PLAN-001 (planner)     ☑ SIGNED — doc on disk
FIRE7-PREFLIGHT-001          ☑ GO — sole extract + minimap boundary
        │
        ▼
F7-A-001 (coder)             ☐ OPEN — per-view invariants + tests
        │   Machine queue: FIRE7-F7-A-EXIT-001 (v2 FIRE7-F7-A-001 = witness bundle only)
        │   MUST land before any “real” F7-B/C
        ▼
F7-B TRIAGE-FIRE-STREAM      ☐ BLOCKED — sleep/wake + neighbor wake systems
F7-C TRIAGE-FIRE-LOD-TIERS   ☐ BLOCKED — band → cap enforcement in extract path
```

### Anti-pattern — “witness block only” (forbidden for F7-B/C)

| Forbidden PR | Why |
|:---|:---|
| Add `fire_streaming_live.json` with all greens and **no** streaming systems | **Not** F7-B |
| Set `f7_b_green: true` in infra JSON without sleep/wake mutating residency | **Not** F7-B |
| Wire LOD caps in witness only, no `FireChunkLodState` / extract clamp | **Not** F7-C |
| Second global fire extract “to make witness green” | Violates **FIRE7-PLAN-001** |

**Allowed before F7-A closes:** planner/designer docs, steward preflight, **F7-A** code + lib tests.

---

## Implementation waves (coder routing)

| Wave | ID | **Real** deliverable (not witness-only) | Exit witness |
|:---|:---|:---|:---|
| **F7-A** | TRIAGE-FIRE-EXTRACT | Invariant tests + bounded `build_fire_visual_frames_by_view`; `Stage5FireViewChunkWitness::f7_a_per_view_extract_bounded` | `infrastructure_view_isolation_live.json` → `fire7_f7_a_001_green` (or `vm08` + new field) |
| **F7-B** | TRIAGE-FIRE-STREAM | Sim systems: active/sleep, neighbor wake, budget — **feeds** `PerViewResidencyConsumerWindow` | `fire_streaming_live.json` + stage6 residency fields |
| **F7-C** | TRIAGE-FIRE-LOD-TIERS | `FireChunkLodState` + view band clamps instance caps in extract | `stage5_full_app_live.json` fire rows + per-band caps exercised in lib test |

**Prereq:** **FIRE7-PREFLIGHT-001** before **F7-A**. **F7-A CLOSED** before **any** F7-B or F7-C product code.

### F7-A exit (unblocks real F7-B/C)

| # | Criterion | Evidence |
|:---:|:---|:---|
| A1 | Sole `FireVisualFramesByView` writer | code + `fire_visual_producer_count() == 1` |
| A2 | Per-view isolation test green | `per_view_fire_extract_bounded` (or successor) |
| A3 | Minimap does not read fire ECS | compositor + preflight ripgrep |
| A4 | Witness field explicit | infra JSON `fire7_f7_a_001_green: true` |
| A5 | Stage 5 not regressed | `cargo test -p proc_A_dine01 --lib stage5 fire_view_extract fire_visual_extract` |

### F7-B exit (product — not stub)

| # | Criterion |
|:---:|:---|
| B1 | Sleep/wake transitions change `ActiveFireChunkSet` or residency hints on tick |
| B2 | Neighbor wake observable in lib test (fixed seed) |
| B3 | `fire_streaming_live.json` written from **runtime** proof system, not hand-authored greens |
| B4 | No new global extract |

### F7-C exit (product — not stub)

| # | Criterion |
|:---:|:---|
| C1 | LOD table § above enforced in `fire_view_extract` / `FireChunkLodState` |
| C2 | Strategic vs tactical instance counts differ under test fixture |
| C3 | Minimap stays heat-only (compositor), tactical stays on `WorldMain` frame |

---

## Witness bundle

| File | Role |
|:---|:---|
| `debug_runs/fire_ecology_live.json` | F1 regression — maintain |
| `debug_runs/stage5_full_app_live.json` | fire projection / instanced dispatch |
| `debug_runs/infrastructure_view_isolation_live.json` | per-view fire isolation |
| `debug_runs/stage6_virtualization_live.json` | residency window |
| `debug_runs/fire_streaming_live.json` | **future** — F7-B |

---

## Copy-paste — F7-A (@coder)

```
Track: FIRE-P7 — F7-A-001
Read: docs/archive/2026-06-src-dev/plans/fire_sim_phase7_architecture_v1.md
      src/render/fire_view_extract.rs
      src/render/extraction/fire_visual_extract.rs
Prereq: FIRE7-PREFLIGHT GO
Budget: ≤3 files per PR
Verify: cargo test -p proc_A_dine01 --lib fire_view_extract fire_visual_extract stage5
Do NOT: second global extract; minimap ECS fire query
```

---

## Changelog

| Version | Date | Notes |
|:---|:---|:---|
| v1.1.0 | 2026-05-25 | Gate chain — **F7-B/C blocked** until F7-A; anti witness-only |
| v1.0.0 | 2026-05-25 | **FIRE7-PLAN-001** signed |
