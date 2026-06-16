# PLAN-M3-DEPTH-EXEC-001 — Minimap M3 product depth execute plan `v1`

| Field | Value |
|:---|:---|
| **Queue ID** | **PLAN-M3-DEPTH-EXEC-001** |
| **Prior** | `PLAN-M3-PRODUCT-DEPTH-001` — [`m3_minimap_product_depth_plan_v1.md`](m3_minimap_product_depth_plan_v1.md) |
| **Version** | `1.0.0` |
| **Date** | 2026-05-26 |
| **Coder lane** | **M3-UNITS-DEPTH-001** (coder B secondary; product depth) |
| **Status** | **READY (planner finalized)** — active next-phase coder B plan |

**Planner sign-off:** PASS (2026-05-27). Queue alignment: archived `PLAN-M3-DEPTH-EXEC-001`.

---

## Coder handoff (acceptance)

| Field | Value |
|:---|:---|
| **Witness** | `debug_runs/minimap_compositor_live.json` |
| **Unblocks** | `M3-UNITS-DEPTH-001` |
| **Acceptance** | `ui_p3_m3_units_001_green=true` AND `unit_marker_rows>0` (real reader, not seed coords); `ui_p3_m3_replay_001_green=true` when `replay_scrub_enabled` and ring `len>=2` |
| **Verify** | `cargo test -p proc_A_dine01 --lib ui_p3_m3_units minimap_compositor` |

---

## Scope

Upgrade minimap M3 “units + replay scrub” from seed/stub behavior into product-grade behavior:
- unit aggregation markers read from strategic/logistics snapshot (not seed coords)
- replay scrub driven by `CommittedSimReplayRing` growth in **Simulation**

This plan only targets the **M3** channels used by:
- `ui_p3_m3_units_001_green`
- `ui_p3_m3_replay_001_green`

---

## Authority map (single writer per resource)

| Resource | Single writer | Evidence / where it’s set | Must NOT be second-written by |
|:---|:---|:---|:---|
| `MinimapOperationalSnapshot.unit_markers` | operational snapshot publisher (seed → replaced by product reader) | `src/render/visual_domain_snapshots.rs` populates `unit_markers` | minimap compositor; any other ECS extract |
| `CommittedSimReplayRing.stamps` | sim cadence stamp recorder | `src/systems/sim_frame_delta.rs` `record_committed_sim_replay_stamp()` | any UI/system that “pushes” ring stamps |
| `MinimapCompositorState.unit_marker_rows` | minimap compositor pass | `src/render/minimap_compositor/composite.rs` `paint_unit_markers()` | direct writes outside minimap compositor |
| `MinimapCompositorState.replay_scrub_enabled` | minimap compositor pass | `src/render/minimap_compositor/composite.rs` `paint_replay_scrub()` | direct UI toggles or manual witness hacks |
| `debug_runs/minimap_compositor_live.json` fields | live proof writer | `src/render/minimap_compositor/live_proof.rs` rollup fields | manual JSON editing |

---

## Task list (U-P1, U-P2, R-P1, R-P2)

Each task below is designed to fit in ≤3 files per PR.

### U-P1 — strategic/logistics snapshot → real unit marker reader (NOT seed coords)
1. Replace or bypass the current seed population:
   - seed function exists: `seed_minimap_m3_units_replay_witness` (currently fills `unit_markers` with fixed coords)
2. Implement real aggregation that outputs `MinimapOperationalSnapshot.unit_markers` as chunk coords.
3. Respect the compositor hard cap:
   - `paint_unit_markers` uses `M3_UNIT_MARKER_CAP` and stores into `unit_marker_rows`

Files (≤3):
- `src/render/visual_domain_snapshots.rs`
- (optional) strategic aggregation reader module used by the real unit snapshot
- `src/render/minimap_compositor/composite.rs` (only if the input contract changes; avoid if possible)

### U-P2 — aggregation rules, zoom LOD, and witness fields in `minimap_compositor_live.json`
1. Ensure M3 marker density stays within the compositor cap (no marker spam):
   - target: `unit_marker_rows > 0` and never exceed the compositor cap logic
2. Ensure witness rollups remain consistent:
   - `ui_p3_m3_units_001_green` must equal `units_heat_enabled && unit_marker_rows > 0`
3. Ensure replay scrub witness fields follow `CommittedSimReplayRing` growth:
   - `ui_p3_m3_replay_001_green` equals `replay_scrub_enabled` from compositor state

Files (≤3):
- `src/render/minimap_compositor/live_proof.rs`
- `src/render/minimap_compositor/composite.rs`
- `src/render/visual_domain_snapshots.rs`

### R-P1 — `CommittedSimReplayRing` live commits on Simulation tick
1. Verify `SimFrameDeltaPlugin` is registered and that `CommittedSimReplayRing` grows during Simulation.
2. The ring recorder is gated by render-extraction fence constraints:
   - commit only when `fence.fire.tick != 0` and `fence.fire.sim_time_micros != 0`

Files (≤3):
- `src/systems/sim_frame_delta.rs`
- `src/render/extraction/fire_visual_extract.rs` (only if fence emission needs adjustment)

### R-P2 — minimap scrub wiring when ring_len >= N
1. Enforce the scrub “active” threshold:
   - `paint_replay_scrub()` returns false when `ring.stamps.len() < 2`
   - therefore `N = 2` for this product depth lane
2. Ensure the `MinimapCompositorHeatSources.replay` input is present in Simulation.

Files (≤3):
- `src/render/minimap_compositor/composite.rs`
- `src/systems/sim_frame_delta.rs` (only if threshold contracts change)

---

## Witness JSON schema + green predicates

**File:** `debug_runs/minimap_compositor_live.json`

Required witness fields for this exec plan:
- `/ui_p3_m3_units_001_green: bool`
- `/unit_marker_rows: number` (or integer-like)
- `/replay_scrub_enabled: bool`
- `/ui_p3_m3_replay_001_green: bool`

Green predicates (authoritative rollups in code):
```text
ui_p3_m3_units_001_green :=
  units_heat_enabled == true
  AND unit_marker_rows > 0

ui_p3_m3_replay_001_green :=
  replay_scrub_enabled == true

replay_scrub_enabled ==
  MinimapOverlayMask.replay_scrub == true
  AND CommittedSimReplayRing.stamps.len() >= 2
```

---

## Verification (required test commands)

```powershell
cargo test -p proc_A_dine01 --lib ui_p3_m3_units
cargo test -p proc_A_dine01 --lib minimap_compositor
```

Additionally (product depth regression):
```powershell
cargo test -p proc_A_dine01 --lib replay_editor_parity
```

---

## Anti-patterns / do-not-reopen list (M3 depth)

Do NOT:
- reopen FoW/EW gating for UI-P3-M4-001 (leave M4 acceptance intact)
- reopen `ui_p3_m3_green` / M2 ecology-construction semantics
- add a separate minimap extraction path for unit markers (use the existing `MinimapOperationalSnapshot → paint_unit_markers` pipeline)
- hand-edit `minimap_compositor_live.json` witness fields

