# P0-BUILD-FOOTPRINT-001 — construction ghost projection exec `v1`

| Field | Value |
|:---|:---|
| **⟨ID⟩** | **P0-BUILD-FOOTPRINT-001** |
| **Program** | ⟨POST-DRAIN-PHASE-3-001⟩ · **G-PLAY-01** 🧩 blocker |
| **Owner** | `@coder` |
| **Playbook** | `$ref:tools/orchestrator/agents/viewport_cleanup_agent.md` |
| **Board** | **VR-10** · `$ref:src/dev/visual_run_blockers.md` § VR-10 |
| **Parent review** | ⟨REVIEW-ORDER-MAP-VFX-UI-DEBUG⟩ · operator witness 2026-06-02 |
| **Date** | 2026-06-02 |
| **φ** | 🟢 closed — hotfix A + witness `map_pick_closure_001.green` 2026-06-12 |

---

## Lattice

```text
⟨P0-BUILD-FOOTPRINT-001⟩  🔴⏳⊗☊
Lattice  Ct:🟨🟨  Cx:🟨🟨  Au:🟩🟩🟩🟩  Rk:⚠☋ VM-09 footprint slice
Flow     ⊚pick(ConstructionMapProjection) ☍ ⊚paint(TileDebugInstance+MainWorldCamera)
Review◈  🧠? TRIAGE-GPU-TILE-001 ★ caused regression · ⊗! skip egui when gpu active
Result   🟢 cursor Δ<8px · footprint under crosshair · ⟨COMMIT:WIT⟩ optional
```

---

## SYMPTOM

- Ghost footprint **not under mouse** when placing buildings
- LMB drag → **blob at bottom of window** (mis-projected GPU quads)
- Pick tile may be correct; **paint path lies**

---

## AUTH split (root cause)

```text
PICK  (correct)
  $sym:build_pick_ghost_tile_system@src/construction/build_interaction.rs
  → $sym:ConstructionMapProjection@src/construction/map_egui_projection.rs
  → $sym:sim_map_cursor_world_xy@src/gui/map_camera.rs
  → SimulationMapViewport hole + ViewProjectionAuthority

PAINT (wrong in Simulation)
  $sym:push_footprint_tile_instances@src/construction/footprint_tile_instances.rs
  → $sym:sync_tile_debug_draw_globals@src/gui/gpu_tile_debug.rs
  → MainWorldCamera full-window view_proj (NO hole scissor)

REGRESSION
  TRIAGE-GPU-TILE-001 ★ → enable_tile_gpu_instanced_authoritative
  visual_authority.rs skips egui footprints when FootprintTileWitness.gpu_path_active
```

**TRIAGE-VM-09-v2 ★** = pose authority only — **does not** close this slice.

---

## DEBUG (landed — BLANG:REF only)

| Asset | Path |
|:---|:---|
| Overlay | `$ref:src/construction/placement_debug.rs` |
| Env | `CONSTRUCTION_PLACEMENT_DEBUG=1` |
| Auto | `--test vfx` · `--test visual` · `--test renderdebug` |
| Crosshair | white=cursor · green=ghost center (egui-correct) |

---

## FIX order

### A — HOTFIX ⚡P0 (1 PR)

- `$ref:src/construction/visual_authority.rs` — **stop** skipping egui `footprint_tiles` when `gpu_path_active`
- Optional: omit footprint rows from `TileDebugInstanceMap` until B

**Exit:** green crosshair overlaps white; footprint tiles under cursor.

### B — PROPER (follow PR)

- `$ref:src/gui/gpu_tile_debug.rs` — hole-aware `view_proj` (match `map_camera.rs` sim hole)
- Re-enable egui skip only when witness `footprint_gpu_hole_correct: true`

**Exit:** GPU quads + egui crosshair same screen position.

### C — DEFER ⟨P0-BUILD-SCALE-002⟩

- `ghost.scale_factor` never wired to input (always 1.0) — separate slice

---

## DO NOT TOUCH

- Construction commit funnel · `$ref:src/dev/construction_invariants.md`
- Stage 5 fire extract spine
- `tools/mcp/` (MCP agents **idle** on this slice)

---

## Verify

```text
BLANG:CARGO  → proc_A_dine01 green
cargo run -p proc_A_dine01 --release -- --test vfx
Construction toolbox → building → "Construction placement (debug)"
Exit: cursor Δ < 8px · no bottom-window blob
```

```text
⟨COMMIT:WIT⟩ debug_runs/construction_stage_live.json
  optional key: footprint_projection_ok: true
```

---

## Agent routing

| Agent | Action |
|:---|:---|
| **@coder** | ⚡P0 pick A then B |
| **@designer** | idle — placement is viewport not HUD chrome |
| **@orchestrator-mcp** | **do not pick** — ΔWF→@coder |
| **@coder-mcp** | **do not pick** |
| **@planner** | queue hygiene only — row already in phase3 |
| **@sim-steward** | triage if dual-writer drift after fix |
| **Operator** | G-PLAY-01 building steps blocked until φ→🟢 |

---

## Related ⟨REVIEW-ORDER⟩ (parallel — not this slice)

| ⟨ID⟩ | Agent | Note |
|:---|:---|:---|
| P0-MINIMAP-WIDGET-001 | @designer | widget drag ≠ texture pan |
| P0-FIRE-TILE-VFX-001 | @coder | chunk tint ≠ tile fire |
| P0-VFX-ZOOM-LOCK-001 | @coder | debug-only tactical lock |
| P0-TERRAIN-BLOB-001 | @coder | raster seam instrument |
