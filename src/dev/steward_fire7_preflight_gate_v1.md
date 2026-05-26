# FIRE7-PREFLIGHT-001 gate `v1`

| Field | Value |
|:---|:---|
| **Lane ID** | `FIRE7-PREFLIGHT-001` |
| **Date** | 2026-05-25 |
| **Owner** | `@sim-steward` |
| **Track** | **FIRE-P7** — [`stages/fire_sim_phase7_plan_v1.md`](stages/fire_sim_phase7_plan_v1.md) |
| **Architecture** | [`fire_sim_phase7_architecture_v1.md`](fire_sim_phase7_architecture_v1.md) (**FIRE7-PLAN-001 SIGNED**) |
| **Minimap boundary** | [`prompts/guides/ui/ui_phase3_minimap_compositor_plan_v1.md`](../prompts/guides/ui/ui_phase3_minimap_compositor_plan_v1.md) |

## Verdict: **GO (qualified)**

Sole fire extract spine and minimap buffer-only path **confirmed**. **F7-A-001** may proceed to `@coder` with ≤3-file budget.

**Post-preflight closure (2026-05-26):** **FIRE7-F7-A-EXIT-001**, **F7-B**, **F7-C** landed — `fire7_f7_a_exit_001.green` + `fire_streaming_live.json` green. **Do not** re-open without product scope change.

**Historical (resolved at preflight):**

- **FIRE7-DESIGN-001** — **SIGNED** [`fire_lod_player_read_v1.md`](fire_lod_player_read_v1.md).
- Sole-extract + minimap buffer path confirmed before F7-A exit wave.

---

## Shift A — Architecture + sole-extract check

### 1 — `FireVisualFrameSet` sole extract writer

| Check | Evidence | Result |
|:---|:---|:---:|
| Single ECS `ChunkSurfaceFire` scan | Only `extract_fire_simulation_snapshot` in `fire_visual_extract.rs` | ✅ |
| Sole `ResMut<FireVisualFramesByView>` writer | Only `build_fire_visual_frames_by_view` in `fire_view_extract.rs` | ✅ |
| Registered producer count | `fire_visual_producer_count() == 1` (`representation_spine_audit.rs`) | ✅ |
| Schedule spine | `FireVisualFrameSet::BuildProfiles` → … → `ProjectGpu` → `EmitParticles`; no parallel global fire extract | ✅ |
| Projection consumes by-view | `fire_frame_for_projection_graph` reads `FireVisualFramesByView` only | ✅ |

### 2 — Minimap compositor does **not** query fire ECS

| Check | Evidence | Result |
|:---|:---|:---:|
| No `ChunkSurfaceFire` / `ActiveFireChunkSet` / `VisibleFireChunkSet` under `minimap_compositor/` | ripgrep clean | ✅ |
| M1 plan invariant | Compositor samples `SharedOverlayFieldBuffers` / committed RT — sim extract via existing lanes | ✅ |
| Live isolation witness | `infrastructure_view_isolation_live.json` → `vm08_overlay_masks_aligned: true`, `infrastructure_view_isolation_green: true` | ✅ |

### 3 — Witness baselines (regression)

| File | Role | Observed |
|:---|:---|:---|
| `debug_runs/fire_ecology_live.json` | F1 ecology | ✅ `f1_green: true` |
| `debug_runs/infrastructure_view_isolation_live.json` | VM-08/10/11 fire isolation | ✅ green |
| `debug_runs/stage5_full_app_live.json` | fire projection / instanced dispatch | (refresh on F7-A witness land) |

---

## Shift B — Route F7-A

```yaml
shift: B
issue:
  id: FIRE7-PREFLIGHT-001
  severity: NONE
route:
  pass: GO — unblocks F7-A-001
  delegate:
    track: FIRE-P7 — F7-A-001
    agent: "@coder"
    budget: "≤3 files per PR"
    read:
      - src/dev/fire_sim_phase7_architecture_v1.md
      - src/render/fire_view_extract.rs
      - src/render/extraction/fire_visual_extract.rs
    first: "invariant test on FireVisualFramesByView isolation (extend per_view_fire_extract_bounded)"
    verify: "cargo test -p proc_A_dine01 --lib fire_view_extract fire_visual_extract stage5"
    do_not:
      - second global FireVisualFrame extract
      - MinimapOnly fire extract
      - minimap ECS fire query
block: none
```

---

## Lib test bundle

Run from repo root (`C:\dev\github\Rust_engine_template_01`). Use alt target dir if Windows **LNK1104** locks the default test exe:

```powershell
$env:CARGO_TARGET_DIR = "target\test-alt-steward"
cargo test -p proc_A_dine01 --lib fire_view_extract
cargo test -p proc_A_dine01 --lib fire_visual_extract
cargo test -p proc_A_dine01 --lib stage5 -- --test-threads=1
```

| Filter | Result (2026-05-26) |
|:---|:---:|
| `steward_w3_gate_001_lib_bundle` | ✅ 1/1 |
| `steward_s7b_preflight_001_lib_bundle` | ✅ 1/1 |
| `fire_view_extract` | ✅ 8/8 |
| `stage5` | ✅ 29/29 |
| `single_fire_visual_producer_registered` | ✅ (in `stage5` bundle) |

---

## Changelog

| Version | Date | Notes |
|:---|:---|:---|
| v1.0.0 | 2026-05-25 | **GO (qualified)** — sole extract + minimap buffer path; routes **F7-A-001** |
