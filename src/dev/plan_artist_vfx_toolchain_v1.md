# PLAN-ARTIST-VFX-TOOLCHAIN-001 — EffectSpec schema critique & rollout `v1`

```text
⟦SYMLANG⟧⟐v1  ◈PLAN  ◈MCP-LANE
⟨ID⟩ PLAN-ARTIST-VFX-TOOLCHAIN-001  ·  VSS-T4-001
Parent: $ref:src/dev/plan_vfx_spectator_scale_program_v1.md § Track 4
Effects arch: $ref:src/dev/plan_effects_system_architecture_v1.md
Schema: $ref:tools/mcp/schemas/effect_spec_v1.schema.json
Example: $ref:tools/mcp/schemas/examples/effect_spec_spark_shower_v1.json
Trip: $ref:tools/orchestrator/queues/vss_trip_artifact_registry.json#TRIP-VSS-EFFECT-SPEC
Status: **ACTIVE** — schema P0 delivered 2026-08-28
Owner: @planner-mcp → @designer-mcp → @coder-mcp → @coder
```

---

## Summary

VSS-T4-001 extends the EffectSpec stub into a **promote-ready JSON contract** for artist-owned VFX consumables. Validators, promote tool, and **`EffectConsumable` ECS loader (T4-004)** are shipped for the reference batch.

---

## Order critique (what was questioned)

| Question | Verdict |
|:---|:---|
| New MCP server for effects? | **Rejected** — extend single `rust-engine-art` MCP (same exec-plan tier as geometry/tile) |
| Inline WGSL in spec? | **Rejected** — `shader_pack` paths only; no AI-generated shader bodies |
| Harness as ship proof? | **Rejected** — `preview.entry_path` allows `harness_parity` but schema text marks it diagnostic; VSS-T2 owns product-path parity |
| Free rotation for particles? | **Rejected** — `grid.rotation_quarters` 0..3 only |
| Require `preview_witness` at author time? | **Deferred** — optional until VSS-T4-003 preview tool runs; **required** implicitly when `development_tier: production` via `rules_check.passed` |
| Duplicate fire draw path? | **Blocked** — `lane: particle_instanced` must route through existing `fire_vfx` → `gpu_particle_draw` spine per TRIP-VSS-FIRE-AUTHORITY |

**Incomplete in brief:** world-scale tile unit constant (VSS-T1-001) is referenced in `grid.tile_units` but not numerically pinned — @designer-mcp must align reference effects once scale contract lands.

---

## Current state

| Item | Label | Notes |
|:---|:---|:---|
| `effect_spec_v1.schema.json` (extended) | **SHIPPED** | VSS-T4-001 deliverable |
| Reference batch `vss_t4_reference_effects_v1` | **SHIPPED** | VSS-T4-002 — spark, smoke_column, rain_streaks |
| Rain lane decision | **SHIPPED** | `particle_instanced` — ES-5 precip draw; `climate_precip` field_sample for density |
| Smoke lane decision | **SHIPPED** | `gpu_field` — smoke projection → weather_fire_field |
| `validate_effect_spec` / `validate-report effect_spec` | **SHIPPED** | VSS-T4-003 |
| `effect_promote` / `effect-pack` / `effect-promote` MCP+CLI | **SHIPPED** | VSS-T4-003 — staging hash pack + registry promote |
| `assets/effects/registry/` on disk | **SHIPPED** | spark_shower, smoke_column, rain_streaks |
| `preview_witness` capture + `capture_hash` | **STUB** | Envelope on witness; `honest_gate: pending` until G4 worker |
| `EffectConsumable` ECS + registry loader | **SHIPPED** | VSS-T4-004 @coder — registry + spawn_hook resolver |
| APS browse/preview/assign panel | **PLANNED** | VSS-T4-005 @designer |
| bpy pack step for `blender_glb` lane | **DEFER** | P1 reference set uses `wgsl_pack` for sparks; bpy path for smoke column meshes in P2 |
| Hanabi as effect lane | **DEFER** | L6 embellishment only — `lane: embellishment` reserved, not in reference batch |
| Separate asset-library MCP server | **DEFER** | Exec plan Phase 4 — premature split |

---

## Target architecture

```text
EffectSpec JSON
  → validate_report (effect_spec_v1)     [SHIPPED VSS-T4-003]
  → pack (wgsl_pack | blender_glb)       [SHIPPED — wgsl hash copy]
  → assets/staging/<effect_id>/
  → preview_witness (G4 sandbox)         [STUB — honest_gate pending G4]
  → promote → assets/effects/registry/   [SHIPPED VSS-T4-003]
  → EffectConsumable (Bevy)              [SHIPPED VSS-T4-004]
  → runtime emit hook (scenario/APS)       [PLANNED VSS-T4-005]
  → spawn_hook resolver at runtime       [PLANNED — same spine as fire_vfx emit]
```

**Schema ownership:** `@planner-mcp` owns `effect_spec_v1`; `@designer-mcp` owns reference spec *content*; `@coder-mcp` owns validator + packer implementation; `@coder` owns runtime consumer.

**Adapter boundary:** Spec never names ECS types — `spawn_hook` + `sim_coupling` are the only sim touchpoints. Render paths reference existing shader files under `assets/shaders/fire/` until artist packs land in `assets/effects/`.

---

## Implementation phases (rollout)

### P0 — Schema (this slice) ✅

- **Goal:** Machine-readable contract for Track 4 queue.
- **Owner:** @planner-mcp
- **Acceptance:** Example validates manually; trip registry honored.
- **Witness:** schema on disk + this plan slice.

### P1 — Reference effect content (VSS-T4-002)

- **Goal:** spark, smoke, rain specs in `tools/mcp/schemas/examples/` or `assets/effects/specs/`.
- **Owner:** @designer-mcp
- **Gates:** G0 schema-green · G1 staging artifacts exist
- **Rule enforcement:** `rules_check` emitted per spec before any pack call.

### P2 — Validate + promote MCP (VSS-T4-003) ✅

- **Goal:** `validate_report effect_spec <path>` · `effect_promote` · `write_witness` → `debug_runs/artist_vfx_pipeline_live.json`
- **Owner:** @coder-mcp
- **Acceptance:** `pytest` + CLI/MCP parity on reference batch — **met** (`green: true`; G4 `honest_gate` pending)
- **Rollback:** disable promote if `honest_gate != honest`.

### P3 — Bevy consumer (VSS-T4-004) ✅

- **Goal:** `EffectConsumable` registry; spawn_hook resolver; lane routing via fire_vfx spine — **extend, don't fork**.
- **Owner:** @coder
- **Acceptance:** **met** — `EffectConsumableRegistryPlugin`, 3 reference effects loaded, 8 lib tests green

### P4 — APS panel (VSS-T4-005)

- **Goal:** Browse batch · preview ladder · assign to scenario markers.
- **Owner:** @designer
- **Gate:** G4 designer sign-off on preview captures.

---

## Gate alignment (G0–G5)

| Gate | EffectSpec field / tool |
|:---|:---|
| G0 Schema | `effect_spec_v1.schema.json` + validate_report |
| G1 Tool run | pack → `assets/staging/<effect_id>/` |
| G2 Validate | `rules_check.passed` + GLB/WGSL hash check |
| G3 Staging review | `preview_witness.honest_gate: honest` |
| G4 Designer | APS preview frames · `development_tier: production` |
| G5 Registry | `promotion.registry_dir` + `library_register` pattern TBD |

---

## Edge cases

| Case | Handling |
|:---|:---|
| Blender absent on CI | `wgsl_pack` lane skips bpy; `blender_glb` jobs fail closed with structured report |
| Mid-batch validation failure | `batch_id` groups rollback — do not partial-promote within `vss_t4_reference_effects_v1` |
| Scale contract drift (T1) | Re-validate `grid.tile_units` + `preview.camera_zoom_px_per_tile` when `world_scale_contract_live.json` updates |
| `field_sample` on stale overlay | `spawn_hook_config.field_sample.field_id` must match `SharedOverlayFieldBuffers` authority names |

---

## Open questions

1. **⌁?** Should `promotion.sidecar` be RON-only (engine default) or JSON mirror for APS? — defer to @coder-mcp with `building_definition_v1` precedent.
2. **⌁?** `capture_hash` algorithm — frame PNG hash vs GPU buffer digest? @coder-mcp must pin in validator, not schema.
3. **⌁✓** Rain reference effect lane — **DECIDED VSS-T4-002:** `particle_instanced` (not `gpu_field` for streak draw). Smoke: `gpu_field`. See `lane_decisions` in `artist_vfx_pipeline_live.json`.

---

## Routing

| Next owner | Slice |
|:---|:---|
| @designer | VSS-T4-005 — APS browse/preview/assign (**pick next**) |
