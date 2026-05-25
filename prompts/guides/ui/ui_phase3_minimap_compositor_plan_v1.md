# UI Phase 3 — GPU minimap compositor plan `v1`

| Field | Value |
|:---|:---|
| **Version** | `1.0.1` |
| **Date** | 2026-05-24 |
| **Promoted** | 2026-05-24 · **`UI-P2B-GATE` PASS** (witness bundle below) |
| **Owner** | `@planner` (architecture) |
| **Status** | **APPROVED** |
| **Prerequisite gate** | [`ui_phase2b_egui_gate_plan_v1.md`](ui_phase2b_egui_gate_plan_v1.md) · `debug_runs/ui_shell_migration_live.json` |
| **Handoff chain** | **`@sim-steward` `UI-P3-PREFLIGHT`** → **`S-M1` gate** → **`@coder` `UI-P3-001`** |
| **M1 gate** | [`minimap_m1_gate_v1.md`](../../../src/dev/minimap_m1_gate_v1.md) — **GO** 2026-05-24 |
| **Spine** | [`tools/orchestrator/knowledge/map_view_spine.json`](../../../tools/orchestrator/knowledge/map_view_spine.json) |
| **Boundary** | [`ui_boundary_guide_v1.md`](../ui_boundary_guide_v1.md) |
| **Design north star** | [`ux_gpu_minimap_design_v1.md`](../../../src/dev/ux_gpu_minimap_design_v1.md) |
| **Archive (landed M1 detail)** | [`ui_phase3_minimap_compositor_plan.md`](../../../src/dev/ui_phase3_minimap_compositor_plan.md) · [`ui_phase3_gpu_minimap_m1_planner_v1.md`](ui_phase3_gpu_minimap_m1_planner_v1.md) |

**No Rust in this deliverable.** Implementation follows steward preflight, then coder slice **`UI-P3-001`**.

---

## Summary

Phase 3 moves minimap **pixels** off the egui world-image bridge onto a **dedicated GPU compositor** (`MinimapCompositorPlugin`) while Phase 2B keeps **simulation product-shell egui** retired. Presentation chrome stays **Bevy** (`MinimapChromeRoot` / `MinimapGpuImageNode` in `simulation_shell_phase2.rs`). The **map-view spine** (`map_view_spine.json`) still resolves frames and caches egui bindings — but on the GPU path `resolve_minimap_egui_texture` **clears** the minimap cache and returns `None`.

**Single-writer invariant:** terrain + overlay heat are **composed once** into `MinimapRenderTargetRegistry.committed_image`. Fire/logistics/ecology **sim extract** flows through existing lanes (`FireVisualFrameSet`, `SharedOverlayFieldBuffers`, `RenderProjectionGraph`) — the compositor **samples published buffers only**.

---

## `UI-P2B-GATE` — promotion criterion (**PASS** 2026-05-24)

Gates below were verified against the witness bundle; plan promoted **DRAFT → APPROVED** on that basis. **`@sim-steward` `UI-P3-PREFLIGHT`** may proceed without re-opening gate unless witnesses regress.

| Gate ID | Source | Required | Verdict (2026-05-24) |
|:---:|:---|:---|:---:|
| **G-2B-01** | `ui_shell_migration_live.json` | `phase2b_closed: true` | ✅ |
| **G-2B-02** | same | `egui_pass_count_in_sim: 0` | ✅ |
| **G-2B-03** | same | `witness.minimap_chrome_aligned: true` | ✅ |
| **G-2B-04** | shell + compositor | GPU path active when env unset | ✅ *qualified* — shell `phase2.minimap_gpu_path: false` at proof frame; `gpu_minimap_compositor_env: true`, `backends.P3_minimap_texture: "bevy_ui_gpu"`, and `minimap_compositor_live.json` → `composite_ok`, `composite_path: GpuCompute`, `dual_minimap_present: false` |
| **G-2B-05** | same | `backends.P3_minimap_texture: "bevy_ui_gpu"` | ✅ |
| **G-2B-06** | compositor witness | `dual_minimap_present: false` | ✅ |
| **G-2B-07** | `infrastructure_view_isolation_live.json` | `minimap_shell_wrote_map_camera_desired: false` | ✅ |
| **G-2B-08** | `stage5_full_app_live.json` | FULL_APP operational gate | ✅ `operational_gate: "FULL_APP"` |

**Optional hardening (not blocking APPROVED):** `UI-P2B-002` counter reset on sim enter — see Phase 2B+ queue.

```powershell
# Gate refresh bundle
cargo test -p proc_A_dine01 --lib simulation_shell_phase2 stage5 minimap_compositor
$env:MINIMAP_GPU_COMPOSITOR = $null
cargo run -p proc_A_dine01 --release -- --test visual
```

---

## Target architecture

```text
┌─────────────────────────────────────────────────────────────────────────────┐
│ Sim / extract (existing — DO NOT duplicate for minimap)                      │
│  FireVisualFrameSet::BuildProfiles → FireVisualFramesByView (per-view cap)   │
│       │                                                                      │
│       ▼                                                                      │
│  ViewRepresentationSystemSet::SyncOverlayField → SharedOverlayFieldBuffers   │
│  LogisticsVisualSnapshot ────────────────────────┐                           │
│  RenderProjectionGraph (fire + logistics + ecology nodes) — policy-shaped   │
│       │ read-only row caps; compositor does NOT re-run graph                │
└───────┼──────────────────────────────────────────────────────────────────────┘
        │
        ▼
┌─────────────────────────────────────────────────────────────────────────────┐
│ MinimapCompositorPlugin (Update · WorldRender · on_visual_cadence_minimap)   │
│  TileWorldFallbackState (terrain) + SharedOverlayFieldBuffers (fire heat)    │
│  LogisticsVisualSnapshot (M2 logistics heat rows)                            │
│  MapViewInstances.minimap.overlays (MinimapOverlayMask toggles)              │
│       │                                                                      │
│       ▼  gpu_compute / minimap_composite.wgsl                                │
│  MinimapRenderTargetRegistry.committed_image  ◄── separate from preview RT   │
└───────┼──────────────────────────────────────────────────────────────────────┘
        │
        ├─► sync_resolved_map_view_frames → ResolvedMapViewFrames.minimap
        │
        ├─► sync_minimap_gpu_image_node_system → MinimapGpuImageNode (Bevy ImageNode)
        ├─► sync_minimap_chrome_root_system → MinimapChromeRoot (wire frame ≤2px)
        │
        └─► resolve_minimap_egui_texture → None when GPU env on (no dual path)

MinimapShellState — presentation only (zoom, rects, toggles, follow)
  · "Does not own terrain or fire extraction" (minimap_shell.rs module doc)
  · Consumer path enum: SharedCpuRaster | SharedRenderTargetImage
```

### Map-view spine (`map_view_spine.json`)

| Spine node | Module | Role in Phase 3 |
|:---|:---|:---|
| `backend` | `map_view/backend/` | `resolve_minimap_texture_source` → **minimap registry** handle |
| `texture_cache` | `map_view/texture_cache/` | egui bind cache — **cleared** on GPU path |
| `presentation_state` | `map_view/view_state.rs` | Independent `MapViewInstances.minimap` vs `world_preview` |
| `resolved_frames` | `map_view/projection/` | `projection_revision` without global viewport churn |
| `minimap_consumer` | `map_view/consumers/minimap.rs` | **Fallback only** — early-out when GPU compositor active |
| `world_preview_consumer` | `map_view/consumers/world_preview.rs` | Must not alias minimap RT |

**Isolation rule** (`view_state.rs`): minimap zoom/focus must not read `MapViewInstances::world_preview` state.

---

## Forbidden — duplicate extraction (hard)

These are **merge blockers** for any `UI-P3-*` PR. **`@sim-steward`** Shift B must assert **none** present before **`UI-P3-001`** lands.

| Forbidden pattern | Why | Correct pattern |
|:---|:---|:---|
| `MinimapOnlyExtract` / minimap-scoped ECS fire query | Second extract path | `SharedOverlayFieldBuffers` after `SyncOverlayField` |
| Compositor reading `FireVisualFrame` entities directly | Bypasses overlay publish | `upload_minimap_heat_textures(overlay, …)` |
| Compositor calling `RenderProjectionGraph::evaluate` | Policy belongs upstream | Read `LogisticsVisualSnapshot.active_overlay_rows` + overlay buffers |
| Shell / egui writing `SharedOverlayFieldBuffers` | Dual writer | Compositor read-only |
| `resolve_minimap_egui_texture` + GPU `ImageNode` both visible | `dual_minimap_present` | GPU on → cache clear + `None` ([`minimap.rs`](../../../src/gui/map_view/consumers/minimap.rs) L27–31) |
| Aliasing `MinimapRenderTargetRegistry` to `WorldPreviewRenderTargetRegistry` | Preview bleed | Separate registries; test `minimap_and_preview_handles_differ_when_both_allocated` |
| `apply_minimap_camera_intent` writing `WorldMain` | VT-5 pose leak | `ViewSurfaceId::Minimap` only |
| New `in_simulation_or_editor` HUD that draws minimap pixels | Phase 2B violation | Bevy chrome + compositor RT |
| Parallel LOD pass inside `minimap_compositor/` | Duplicate policy | `RepresentationResult` → cadence + caps upstream |

**Module contract** ([`minimap_shell.rs`](../../../src/gui/minimap_shell.rs)):

> Does not own terrain or fire extraction; `TileWorldFallbackState` + `SharedOverlayFieldBuffers` remain authoritative.

**Consumer contract** ([`minimap.rs`](../../../src/gui/map_view/consumers/minimap.rs)):

> Minimap egui sampling through the shared map-view backend — **no** alternate ECS extraction.

---

## Authority map

| Layer | Writer | Reads | Must not |
|:---|:---|:---|:---|
| Fire sim extract | `FireVisualFrameSet` | ECS fire | — |
| Per-view fire cap | `view_fire_projection` / `FireVisualFramesByView` | Policy | Second minimap extract |
| Overlay publish | `SyncOverlayField` | Fire frames | Compositor RT |
| Logistics rows | `LogisticsVisualSnapshot` | Sim transport | Compositor policy mutation |
| Projection graph | `evaluate_render_projection_graph` | Snapshots + `RepresentationResult` | Minimap pixels |
| **Compositor pixels** | `run_minimap_compositor_pass` | Published buffers + fallback terrain | ECS, `ViewManager` |
| RT lifecycle | `MinimapRenderTargetRegistry` + resize queue | `ResolvedViewports.minimap_panel` | egui layout as extent authority |
| Frame metadata | `sync_resolved_map_view_frames` | Registry revision | Raster content |
| Bevy display | `sync_minimap_gpu_image_node_system` | `committed_image` | Composite shaders |
| Chrome geometry | `sync_minimap_chrome_root_system` | `MinimapShellState` rects | Pixels |
| Shell UX | `MinimapShellState` | — | Extract / overlay buffers |
| egui fallback | `resolve_minimap_egui_texture` | CPU raster / RT via cache | GPU path bind |

### `RenderProjectionGraph` relationship (read-only)

[`render_projection_graph.rs`](../../../src/render/extraction/render_projection_graph.rs) shapes **GPU row counts** for fire instances, logistics, ecology from `RepresentationResult` and committed sim stamps. The minimap compositor:

- **May read** `LogisticsVisualSnapshot::active_overlay_rows` (and overlay buffers) **after** graph-aligned publish.
- **Must not** invoke `ProjectionNodeTrait::evaluate` or duplicate `FireProjectionNode` / `LogisticsProjectionNode` work inside the compositor pass.

Witness alignment: `minimap_compositor_live.json` → `logistics_rows` should track projection when LOG-E01 scenario is seeded; `stage5` readiness string `order=fire+logistics+ecology` is the spine check, not a second compositor extract.

---

## Schedule + `run_if` strategy

### Compositor plugin (`MinimapCompositorPlugin`)

```text
Update · ViewRepresentationSystemSet:

  ViewportPipelineSet::Resolve
    └─ RenderTargets (chain)
         queue_minimap_render_target_resize
         apply_minimap_gpu_resize_request
         commit_minimap_render_target_bind_system

  FireVisualFrameSet::BuildProfiles
    └─ SyncOverlayField
         └─ WorldRender (chain)
              sync_minimap_presentation_source
              run_minimap_compositor_pass     [run_if: on_visual_cadence_minimap]

PostUpdate · Simulation (shell):
  sync_minimap_gpu_image_node_system    (after egui pass — bind ImageNode)
  sync_minimap_chrome_root_system

PostUpdate · MinimapCompositorPlugin:
  write_minimap_compositor_live_proof_system   [run_if: in_state(Simulation)]

EguiPrimaryContextPass:
  hud_root_tick → resolve_minimap_egui_texture
    early None when SharedRenderTargetImage + minimap_gpu_compositor_env_enabled()
```

| System | `run_if` / gate | Notes |
|:---|:---|:---|
| `run_minimap_compositor_pass` | `on_visual_cadence_minimap` + env + shell visible | Multirate via `VisualCadence.minimap_hz` |
| `resolve_minimap_egui_texture` | implicit GPU gate inside function | Clears `MapViewInstanceId::Minimap` cache |
| `sync_minimap_gpu_image_node_system` | `gpu_active` = env ∧ `SharedRenderTargetImage` ∧ RT bound | Sets `witness.minimap_gpu_path` |
| `minimap_egui_texture_dock_active` (editor) | `product_egui_shell_active` | Sim uses Bevy chrome only ([`ui_phase2b_egui_gate_plan_v1.md`](ui_phase2b_egui_gate_plan_v1.md)) |

### Env gate

| `MINIMAP_GPU_COMPOSITOR` | Behavior |
|:---|:---|
| unset | GPU compositor **on** (default) |
| `0` / `false` | CPU bridge + egui texture fallback |
| `1` / `true` | GPU on |

---

## Overlay inputs (M1 + M2)

| Input | Producer | Compositor use |
|:---|:---|:---|
| `TileWorldFallbackState` | Fallback raster | Terrain storage sync |
| `SharedOverlayFieldBuffers` | `SyncOverlayField` | Fire heat texture upload |
| `LogisticsVisualSnapshot` | LOG-E01 transport lane | Logistics heat (M2); `logistics_rows` witness |
| `MinimapOverlayMask` | `MapViewInstances.minimap.overlays` | `fire_heat`, `logistics_heat` uniforms |
| `ResolvedViewports.minimap_panel` | Viewport resolve | RT extent, `extent_match_px` |

**M3 defer (out of `UI-P3-001`):** construction phase channel, ecology macro band, fog-of-war — [`ux_gpu_minimap_design_v1.md`](../../../src/dev/ux_gpu_minimap_design_v1.md) §4.

---

## Witness + diagnostics

| Artifact | Profile / fields | Owner |
|:---|:---|:---|
| `debug_runs/minimap_compositor_live.json` | `MINIMAP_COMPOSITOR_M1` · `composite_ok`, `dual_minimap_present`, `logistics_rows`, `composite_path` | compositor live proof |
| `debug_runs/ui_shell_migration_live.json` | `minimap_chrome_aligned`, `minimap_gpu_path`, `P3_minimap_texture` | shell migration |
| `debug_runs/stage5_full_app_live.json` | minimap RT + compositor revision block | FULL_APP |
| `debug_runs/infrastructure_view_isolation_live.json` | minimap must not write world camera desired | VT-5 |

**`dual_minimap_present` definition:** `true` if egui world minimap texture and Bevy `MinimapGpuImageNode` are both visible in the same frame. **Target:** `false` always in sim with GPU default.

**`egui_pass_count_in_sim`:** Phase 2B metric — compositor must not increment it; product shell root must not run in sim.

---

## Handoff — `@sim-steward` **`UI-P3-PREFLIGHT`**

**Goal:** Authority + spine regression before **`@coder` `UI-P3-001`** touches compositor or shell files.

### Shift A — Observe (readonly)

1. Read witnesses: `ui_shell_migration_live.json`, `minimap_compositor_live.json`, `infrastructure_view_isolation_live.json`, `stage5_full_app_live.json`.
2. Confirm **`UI-P2B-GATE`** table (§ above) — if any red, route **`@coder`** Phase 2B+ first; **hold** Phase 3 promotion.
3. Map writers: compositor → `MinimapRenderTargetRegistry`; shell → `MinimapShellState` geometry only; map-view → frame resolution only.
4. Scan for forbidden patterns (§ Forbidden) via grep: `MinimapOnly`, duplicate `extract_minimap`, compositor `Query<` fire components.

### Shift B — Decide

Emit debug-intelligence YAML:

```yaml
issue:
  id: UI-P3-PREFLIGHT
  severity: HIGH
root_cause: [list if gate fail or dual path detected]
affected:
  - src/render/minimap_compositor/
  - src/gui/map_view/consumers/minimap.rs
  - src/gui/hud/simulation_shell_phase2.rs
gates:
  UI-P2B-GATE: pass | fail
  dual_minimap_present: false | true
  duplicate_extraction: none | FOUND
route:
  pass: "@coder UI-P3-001"
  fail_2b: "@coder UI-P2B-* per ui_phase2b_egui_gate_plan_v1.md"
  fail_vt: "@coder viewport_cleanup_agent + render_pipeline_agent"
```

### Shift C — Act (bounded)

| Condition | Action |
|:---|:---|
| All gates green + no forbidden patterns | Sign **`UI-P3-PREFLIGHT: GO`** in `HANDOFF.md`; notify `@coder` |
| `dual_minimap_present` or VT bleed | Bounded fix ≤3 files **or** handoff to `@coder` with evidence paths |
| `UI-P2B-GATE` fail | **No** Phase 3 pixel work — route 2B lane |

### Preflight commands

```powershell
cargo test -p proc_A_dine01 --lib minimap_compositor simulation_shell_phase2 stage5
cargo test -p proc_A_dine01 --lib map_view -- minimap
Remove-Item Env:MINIMAP_GPU_COMPOSITOR -ErrorAction SilentlyContinue
cargo run -p proc_A_dine01 --release -- --test visual
```

**Playbook:** [`tools/orchestrator/agents/ui_layout_agent.md`](../../../tools/orchestrator/agents/ui_layout_agent.md) · [`tools/orchestrator/agents/stage5_readiness_agent.md`](../../../tools/orchestrator/agents/stage5_readiness_agent.md)

---

## Handoff — `@coder` **`UI-P3-001`**

**Prerequisite:** **`UI-P3-PREFLIGHT: GO`** (plan **APPROVED** 2026-05-24; gate table above).

**Scope:** Operationalize GPU minimap as the **simulation default**; close witness gaps; **no** new extract paths. If M1/M2 already green, `UI-P3-001` is **verify + harden + document** — not greenfield.

### File list (touch ≤3 files per commit)

| Step | Files | Intent |
|:---:|:---|:---|
| **P3-001.1** | `src/render/minimap_compositor/pass.rs`, `src/gui/minimap_shell.rs` | Default `presentation_source`, env + shell sync |
| **P3-001.2** | `src/gui/map_view/consumers/minimap.rs`, `src/gui/hud/hud_root_tick.rs` | Enforce egui early-out; no dual bind |
| **P3-001.3** | `src/gui/hud/simulation_shell_phase2.rs` | Chrome/GPU node alignment; witness fields |
| **P3-001.4** | `src/render/minimap_compositor/live_proof.rs` | Proof payload completeness |
| **P3-001.5** | `src/gui/map_view/projection/mod.rs` | Revision hash stability (VT flicker guard) |

**Read-only authority (do not duplicate logic):**

- [`minimap_shell.rs`](../../../src/gui/minimap_shell.rs) — presentation state
- [`minimap.rs`](../../../src/gui/map_view/consumers/minimap.rs) — egui consumer gate
- [`simulation_shell_phase2.rs`](../../../src/gui/hud/simulation_shell_phase2.rs) — `MinimapChromeRoot`, `MinimapGpuImageNode`
- [`render_projection_graph.rs`](../../../src/render/extraction/render_projection_graph.rs) — policy-shaped row caps only

### Acceptance — **`UI-P3-001`**

| # | Criterion | Verify |
|:---:|:---|:---|
| A1 | `UI-P2B-GATE` all rows green | witness JSON |
| A2 | `minimap_compositor_live.json` → `composite_ok: true`, `dual_minimap_present: false` | witness |
| A3 | `composite_path: "GpuCompute"` with GPU env default | witness |
| A4 | `logistics_rows > 0` when LOG scenario seeded (M2) | witness / visual |
| A5 | `extent_match_px` ≤ 1.0 at stable layout | witness |
| A6 | `cargo test -p proc_A_dine01 --lib minimap_compositor stage5` green | CI |
| A7 | No new `MinimapOnly*` extract or compositor ECS fire `Query` | grep / review |
| A8 | `infrastructure_view_isolation_live.json` minimap camera isolation | witness |
| A9 | Sim session: minimap visible via Bevy chrome, F3 diagnostics still works | manual |
| A10 | Editor: egui minimap dock still available when `product_egui_shell_active` | editor smoke |

### Copy-paste — **`UI-P3-001`**

```
Lane: UI-P3-001 — GPU minimap compositor operationalization
Read: prompts/guides/ui/ui_phase3_minimap_compositor_plan_v1.md (APPROVED v1.0.1)
      ui_phase2b_egui_gate_plan_v1.md
Preflight: @sim-steward UI-P3-PREFLIGHT must be GO
First: confirm resolve_minimap_egui_texture returns None on GPU path; dual_minimap_present false
Do NOT: add MinimapOnlyExtract, compositor fire ECS queries, or shell writes to SharedOverlayFieldBuffers
Verify: cargo test -p proc_A_dine01 --lib minimap_compositor stage5
        refresh debug_runs/minimap_compositor_live.json + ui_shell_migration_live.json
```

---

## Rollback path

| Trigger | Action |
|:---|:---|
| `dual_minimap_present: true` | Revert P3-001.2; verify egui cache clear |
| VT-5 camera bleed | Revert `apply_minimap_camera_intent` changes; check isolation witness |
| GPU composite regression | `MINIMAP_GPU_COMPOSITOR=0` for session; CPU + egui fallback |
| FULL_APP red | Fix spine only per Stage 5 directive — do not add parallel minimap extract |

**Partial hotfix:** env opt-out to CPU path preserves sim playability without reverting Bevy chrome.

**Post-rollback verify:**

```powershell
cargo test -p proc_A_dine01 --lib stage5 minimap_compositor
```

---

## Phase 3 forward queue (post-`UI-P3-001`)

| ID | Owner | Goal |
|:---|:---|:---|
| **UI-P3-M2-001** | `@coder` | Logistics heat — **done** → **D-MINIMAP-M2** [`minimap_d_m2_signoff_v1.md`](../../../src/dev/minimap_d_m2_signoff_v1.md) |
| **UI-P3-M3-001** | `@coder` | Construction / ecology — **done** (M2 channels; `ui_p3_m3_green`) |
| **UI-P3-M2-TRAY-OPT** | `@coder` | Overlay tray → mask — **optional** |
| **UI-P3-M4-001** | `@designer` + `@coder` | FoW + multirate polish per design doc |
| **UI-P3-DEFAULT-001** | `@coder` | Remove env gate; GPU always on when RT valid (product decision) |

---

## Edge cases

| Case | Expected |
|:---|:---|
| Sim enter before first egui layout | `bootstrap_simulation_layout_rect` seeds chrome |
| RT resize mid-frame | Deferred commit via `MinimapRenderTargetBindBarrier` |
| `MINIMAP_GPU_COMPOSITOR=0` | `CpuBridge`; egui may bind CPU raster |
| WorldGen + Editor | Product egui shell off; minimap egui dock not sim path |
| Minimap minimized / hidden | Compositor skip; GPU node hidden |
| Preview + minimap both allocated | Distinct handles (unit test) |
| Strategic zoom | Overlay rows may be 0; compositor still valid if `composite_ok` |

---

## Open questions

| ID | Question | Default |
|:---|:---|:---|
| Q1 | Promote plan to APPROVED while M1 code already landed? | **Done** 2026-05-24 — governance doc now APPROVED |
| Q2 | Make GPU path unconditional (drop env)? | Defer `UI-P3-DEFAULT-001` |
| Q3 | Rename witness `egui_pass_count_in_sim` vs compositor counter? | Phase 2B+ only |
| Q4 | CI gate `phase2b_closed` + `dual_minimap_present` in orchestrator? | Optional harness extension |

---

## Document index

| Doc | Role |
|:---|:---|
| [`ui_phase2b_egui_gate_plan_v1.md`](ui_phase2b_egui_gate_plan_v1.md) | Prerequisite egui retirement |
| [`ui_phase3_coder_queue_v1.md`](ui_phase3_coder_queue_v1.md) | M2/M3 task queue |
| [`ui_phase3_minimap_compositor_plan.md`](../../../src/dev/ui_phase3_minimap_compositor_plan.md) | Landed M1 file-level archive |
| [`map_view_spine.json`](../../../tools/orchestrator/knowledge/map_view_spine.json) | Orchestrator knowledge graph |

---

## Changelog

| Version | Date | Notes |
|:---|:---|:---|
| v1.0.1 | 2026-05-24 | **APPROVED** — `UI-P2B-GATE` PASS (witness bundle); handoffs unlocked |
| v1.0.0 | 2026-05-24 | Initial DRAFT — UI-P2B-GATE, no-duplicate-extract, UI-P3-PREFLIGHT / UI-P3-001 handoffs |
