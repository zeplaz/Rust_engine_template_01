# Fire sim Phase 7 — long-horizon sim depth `v1`

| Field | Value |
|:---|:---|
| **Track ID** | `FIRE-P7` |
| **Version** | `1.0.0` |
| **Status** | **PREFLIGHT GO** — [`../steward_fire7_preflight_gate_v1.md`](../steward_fire7_preflight_gate_v1.md) (2026-05-25) |
| **Exit milestone** | **F7-A-001** → F7-B/C implementation waves |
| **Preflight gate** | [`../steward_fire7_preflight_gate_v1.md`](../steward_fire7_preflight_gate_v1.md) |
| **F1 (done)** | [`../fire_ecology_f1_todos.md`](../fire_ecology_f1_todos.md) |
| **Triage** | [`../stage5_triage_backlog.md`](../stage5_triage_backlog.md) T3 |

**Not a Stage 5 gate.** Visual fire spine stays on `FireVisualFrameSet` + projection graph.

---

## North star

Per-view fire extract, streaming sleep/wake, and LOD tiers are **authoritative and bounded** — ecology F1 remains baseline; Phase 7 adds scale behavior.

---

## Phases (implementation waves)

| Wave | ID cluster | Goal |
|:---|:---|:---|
| **F7-A** | TRIAGE-FIRE-EXTRACT | Harden `FireVisualFramesByView` + visible set |
| **F7-B** | TRIAGE-FIRE-STREAM | Active/sleep chunk streaming, neighbor wake |
| **F7-C** | TRIAGE-FIRE-LOD-TIERS | Strategic → cinematic policy table |

---

## Witness bundle (target)

| File | Use |
|:---|:---|
| `debug_runs/fire_ecology_live.json` | F1 regression |
| `debug_runs/stage5_full_app_live.json` | fire projection rows |
| `debug_runs/infrastructure_view_isolation_live.json` | per-view fire isolation |
| New (future) | `debug_runs/fire_streaming_live.json` |

---

## @planner instructions

### FIRE7-PLAN-001 (before F7-A code)

Deliver **one page** in [`fire_sim_phase7_architecture_v1.md`](../fire_sim_phase7_architecture_v1.md) — **DONE** (2026-05-25):

| Section | Content |
|:---|:---|
| Authority map | Who writes `VisibleFireChunkSet`, `FireChunkLodState` |
| Schedule | Relation to `publish_stage6_virtualization_frame` (no cycles) |
| LOD table | Band → instance cap → overlay bin |
| Forbidden | Second global fire extract for minimap |

**Blocks:** **F7-A** until planner doc exists. **Blocks real F7-B/C** until **F7-A-001** closes (witness-only JSON does **not** count) — see architecture § Gate chain.

---

## @designer instructions

### FIRE7-DESIGN-001 — Overlay readability by LOD band

| Band | Player should see |
|:---|:---|
| Strategic | Heat blobs only |
| Operational | Cluster caps |
| Tactical | Instances + sparks (VFX track) |
| Cinematic | Full local detail |

**Deliverable:** table in `src/dev/fire_lod_player_read_v1.md` — no Rust.

### FIRE7-DESIGN-002 — Debug overlay tooling (optional)

F3 section names for fire streaming stats (align BQ-134 telemetry style).

---

## @sim-steward instructions

### FIRE7-PREFLIGHT-001

**Before any F7-A code:**

1. Confirm `FireVisualFrameSet` is sole extract writer
2. Confirm compositor does **not** query fire ECS ([`ui_phase3_minimap_compositor_plan_v1.md`](../../prompts/guides/ui/ui_phase3_minimap_compositor_plan_v1.md) forbidden list)
3. Shift B YAML → route F7-A to `@coder` with ≤3 file budget

---

## @coder instructions (do not start until FIRE7-PLAN-001)

### F7-A-001 — Per-view visible set hardening

```
Track: FIRE-P7 — F7-A-001
Read: src/dev/stages/fire_sim_phase7_plan_v1.md
      src/render/fire_view_extract.rs
Prereq: FIRE7-PREFLIGHT GO + fire_sim_phase7_architecture_v1.md
First: one invariant test on FireVisualFramesByView isolation
Do NOT: add MinimapOnly fire extract
Verify: cargo test -p proc_A_dine01 --lib fire_view_extract stage5
```

**Defer F7-B/C** until **F7-A-001** **CLOSED** (code + lib tests + `fire7_f7_a_001_green`). **No** `fire_streaming_live.json` stub PRs.

---

## Acceptance — Fire Phase 7 preflight (planning exit)

| # | Criterion |
|:---:|:---|
| F0 | F1 ecology witness still green |
| F1 | Planner architecture doc exists |
| F2 | Designer LOD read table exists — [`fire_lod_player_read_v1.md`](../fire_lod_player_read_v1.md) **SIGNED** |
| F3 | Steward preflight GO recorded |
| F4 | No implementation until F1–F3 done |

---

## Changelog

| Version | Date | Notes |
|:---|:---|:---|
| v1.0.0 | 2026-05-24 | Planning-only track |
