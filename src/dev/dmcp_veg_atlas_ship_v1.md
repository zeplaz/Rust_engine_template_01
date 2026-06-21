# Landscape veg atlas art-ship criteria `v1` — DMCP-VEG-ATLAS-SHIP-001

| Field | Value |
|:---|:---|
| **Gate** | **DMCP-VEG-ATLAS-SHIP-001** |
| **Program** | T3 VEG-SHIP · APS-E4 |
| **Owner** | `@designer-mcp` |
| **Date** | 2026-06-02 |
| **Machine spec** | [`veg_atlas_ship_001.json`](../assets/staging/specs/veg_atlas_ship_001.json) |
| **Authority** | [`design_landscape_lg5_expand_bake_v1.md`](design_landscape_lg5_expand_bake_v1.md) · [`design_landscape_lg5_keyframe_qc_v1.md`](design_landscape_lg5_keyframe_qc_v1.md) · [`landscape_expanded_g0_rules.yaml`](../debug_runs/art_pipeline/landscape_expanded_g0_rules.yaml) |
| **Verdict** | **PASS WITH NOTES** — criteria locked · **`ship: false` stays** until G4 manual green |

```yaml
order_critique:
  request_summary: "Lock G4/G5 art-ship sign-off criteria for landscape_lg5_expanded_v1"
  rules_audit:
    no_ai_generated_images: pass
    deterministic_output: pass
    honest_ship_false: pass
    g4_manual_complete: open
  proceed: yes_with_notes
  note: "Criteria doc ≠ ship flip — operator G4 still required"
```

---

## 0. Honest state (2026-06-02)

| Field | Value | Meaning |
|:---|:---|:---|
| `batch.ship` | `false` | Teach tier — correct |
| `development_tier` | `pilot` | Not production until G4 |
| `proceed_tile_ship` (G0) | `no` | Do not register production row |
| Teach keyframes | 16/16 @ 64px | Phase A complete |
| G4 gap | `topology_corridor_regrowth_grass` | Manual still deferred |

**Forbidden:** any witness or registry row with `ship: true` before §3 G4 sign-off.

---

## 1. Gate ladder (G0 → G5)

| Gate | Owner | Pass when | Artifact |
|:---|:---|:---|:---|
| **G0** | @designer-mcp | Rules audit · batch aligned · `proceed_production_bake: yes` | `landscape_expanded_g0_rules.yaml` |
| **G3** | @coder-mcp | 16 teach PNGs · batch witness green | `tile_landscape_expanded_live.json` |
| **G4** | @designer-mcp + operator | Manual iso-rig stills pass §2 rubric | `landscape_expanded_g4_signoff.yaml` |
| **G5** | @coder-mcp + @designer-mcp | Atlas pack + register · index production row | `tile_tile_landscape_expanded_v1_live.json` |

**This gate (DMCP-VEG-ATLAS-SHIP-001)** defines G4/G5 criteria and blocks `ship:true` until G4 `proceed_ship: yes`.

---

## 2. G4 minimum review set (operator + designer-mcp)

Per [`design_landscape_keyframe_burn_reqs_v1.md`](design_landscape_keyframe_burn_reqs_v1.md) §3:

| Variant key | Rubric |
|:---|:---|
| `topology_patch_burn_04` | Burn reads orange @ 64px iso — distinct from scar |
| `topology_patch_scar` | Ash wash — not confused with burn or clean patch |
| `topology_corridor_regrowth_grass` | Corridor spine legible · regrowth green distinct from scar |

Plus cross-check [`design_veg_burn_visual_language_v1.md`](design_veg_burn_visual_language_v1.md):

| Criterion | Fail if |
|:---|:---|
| Burn vs scar | Same hue family at thumbnail |
| Corridor spine | Indistinguishable from patch fill |
| Regrowth | Reads as clean mature canopy when state is regrowth |

**Export path:** `assets/staging/tiles/keyframe_stills/tile_landscape_expanded_v1/` via `Tile_iso_rig_v1.blend` — **not** procedural overlay for G4 pass.

---

## 3. G4 sign-off artifact

File: `debug_runs/art_pipeline/landscape_expanded_g4_signoff.yaml`

| Field | Required |
|:---|:---|
| `proceed_ship` | `yes` \| `no` |
| `reviewer` | `designer-mcp` + `operator` |
| `minimum_set` | All three §2 keys reviewed |
| `verdict` | `PASS` \| `PASS_WITH_NOTES` \| `FAIL` |

**Ship flip authorized only when** `proceed_ship: yes` **and** all minimum_set keys have `manual_still: true`.

Current template: **`proceed_ship: no`** — corridor regrowth manual still open.

---

## 4. Registry flip (G5 — after G4 yes)

When §3 passes:

| Field | Before | After |
|:---|:---|:---|
| `tile_batch_landscape_expanded_v1.json` → `ship` | `false` | `true` |
| `development_tier` | `pilot` | `production` |
| `_landscape_atlas_index.ron` | teach row | `atlas_id: landscape_lg5_expanded_v1` production |
| `bake_source` | `keyframe_pack` | `blender_keyframe_light_rig` |

Rollup witnesses (all green before engine consumer):

| ID | Witness |
|:---|:---|
| A1 | `tile_landscape_expanded_v1_live.json` |
| A2 | `tile_tile_landscape_expanded_v1_live.json` |
| A3 | `landscape_expanded_g4_signoff.yaml` → `proceed_ship: yes` |
| A4 | `_landscape_atlas_index.ron` production stamp |
| A5 | `vegetation_system_honest_status_v1.md` reconcile |

---

## 5. Unblocks

| Row | Owner | After this gate |
|:---|:---|:---|
| **VEG-F01-ATLAS-SHIP-001** | @coder_a | Engine LG-5 consumer wiring (may stay teach until G4 yes) |
| **VEG-F02-BURN-ATLAS-001** | @coder-mcp | Burn atlas bake after catalog burn rows |
| **APS-EVO-E4** | @coder-mcp | Maintain `ship:false` until G4 sign-off file flips |

---

## Sign-off

| Role | Verdict | Date |
|:---|:---|:---|
| `@designer-mcp` | **PASS WITH NOTES** | 2026-06-02 |

```text
DMCP-VEG-ATLAS-SHIP-001 Q✓ — G4/G5 criteria locked · ship:false honest until operator G4
```
