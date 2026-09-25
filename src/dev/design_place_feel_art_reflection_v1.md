# Place-feel art reflection `v1` — kit phase must not lie at the click

| Field | Value |
|:---|:---|
| **ID** | **DES-PLACE-FEEL-ART-001** |
| **Date** | 2026-09-25 |
| **Owner** | `@designer-mcp` |
| **Does not edit** | `src/gui/**`, `src/construction/**`, `src/economy/**` |
| **Witness** | [`debug_runs/place_feel_art_pipeline_reflection.json`](../../debug_runs/place_feel_art_pipeline_reflection.json) |
| **Lint** | CLI `place-feel-phase-note` · rule `TIER-PLACE-001` when `place_feel_claim` is set |
| **Rebake** | No |

```yaml
order_critique:
  request_summary: "Reflect Stronghold/SimCity place-feel (tight click, icon, ghost, then the thing that stays) into art-pipeline phase honesty."
  concerns:
    - "development_tier (smoke/lod0/production) is being read as if it were the site phase, the ghost FSM, or deployable stock."
    - "Coverage and kit-fill can stay green while the click still lands an lod0 stand-in."
    - "Picker letters, unit sprites, and 4 m modules share no click-scale contract."
  rules_audit:
    no_ai_generated_images: n/a
    deterministic_output: pass
    batch_processing: pass
    grid_alignment: pass
  blocked: false
  proceed: yes_with_documented_tradeoffs
  foresight_flags:
    - "Do not add a fourth development_tier for Planned, ghost, or in-transit."
    - "Do not rebake the world to 'fix' a label."
    - "HUD/ghost drawing stays @designer. This doc only stops the art lane from certifying the lie."
```

---

## Three sharpest lies

### 1. Same-pack lod0 beats a real building, then the map says Built

`bq_smoke_tier_audit.py` already states the mechanic: kit-fill treats any GLB resolve, including lod0 fallback, as covered, and a style-pack pool prefers **same-pack lod0** over **cross-pack production**.

That is a place-feel lie. The player clicked a catalog row that reads as a finished building. The mesh that lands can still be a pilot stand-in. `phase_visual` then paints `SiteConstructionPhase::Operational` green and the label **Built**. `BUILD-READ-VISUAL-001` still records `mesh_tier_used: lod0` as a passing extract when the iso stamp is absent (`parametric_commit.rs`).

Green coverage ≠ the thing that stayed under the cursor.

**Art-lane rule:** `place_feel_claim: operational_envelope` is legal only at `development_tier: production`. lod0 may claim `preview_ghost_stand_in` only. It must not claim the placed envelope.

### 2. Preview, Planned, and the kit are three clocks

The ghost FSM is `BuildPlacementMode`: `Place` means **Preview** (follows the cursor), `Adjust` means the locked ghost. Neither value is a `development_tier`. After the second click the ghost unlocks and a **Planned** site tile (blue, `phase_visual`) appears on a different clock. The module that eventually instances is a third clock (`smoke` hidden, `lod0` still stylepack-visible, `production` ship).

A Stronghold/SimCity click keeps one silhouette: translucent copy, then the same volume sitting on the grid. This repo draws the click as a footprint tint and the “building” as whichever tier the pack resolved. Nothing in the job JSON says those two pictures must be the same mesh.

**Art-lane rule:** the honest production ghost is `preview_ghost_same_mesh` (same GLB as `operational_envelope`, drawn translucent by `@designer`). An lod0 ghost is `preview_ghost_stand_in` and must stay labeled stand-in (APS rubric: “LOD0 stand-in — production pending”). Do not swap tiers to mean Preview vs Built.

### 3. A 32px cell is not a 4 m module, and a truck is not a building click

`assets/configs/ui/icon_atlas_phase4.icon_atlas.ron` is one sheet: `cell_size (32, 32)`, atlas `256×128`. Building-rail categories (`RD`, `RL`, `UT`, `IN`, `CV`) and movers (`TRUCK`, `URAL`, `BUS`) occupy the same cell budget. Modules are `GRID_UNIT_M = 4` with `bottom_center` pivot.

The tight click wants a building silhouette that matches the ghost footprint. The unit wants a small marker that can move. The atlas does not say which cells are place-targets and which are movers, and a geometry job has no icon role. Using a module thumbnail as the click icon collapses those sizes.

**Art-lane rule:** `place_feel_claim: picker_icon` is **always illegal** on a geometry job or AssetSpec. The icon stays an atlas cell owned by `@designer`. Do not downscale a GLB and call it the Stronghold button.

---

## Phase vocabulary (do not alias)

| Clock | Values | What the player sees | What kit phase is allowed to do |
|:---|:---|:---|:---|
| Kit phase | `smoke`, `lod0`, `production` | Mesh finish | The only enum on the job |
| Site phase | Planned, Survey, Clearing, Foundation, Building, Provisioning, Built, Damaged, Offline | Colored footprint tiles | Nothing. Do not encode Built by promoting, or Planned by leaving lod0 |
| Ghost FSM | Preview (`Place`), Adjust | Footprint under the cursor, then locked | Production = same mesh; lod0 = labeled stand-in only |
| Deployable stock | in transit, staged, ready | Caption on the ghost; commit blocked | Not a tier. Do not show lod0 to mean “not on site” |
| Icon atlas | 32px cells | Rail category letters and unit sprites | Not a module output |

Deployable defense is the trap case. Dragon’s teeth can be a **production** prefab and still be unplaceable because stock is in transit. Swapping that module to lod0 or smoke to “look unready” teaches the wrong click: unfinished art instead of blocked stock. The caption already belongs to `@designer` (`design_mil_deployable_defense_v1.md`). The art lane must not invent a kit phase for it.

Parametric `scale_factor` is the other trap. Ghost footprint tiles can resize. A module bay cannot, or grid alignment breaks. The click shape is integer 4 m bays. Do not uniformly scale the GLB to match Shift+scroll.

---

## Lint (local, silent unless a claim is asserted)

`development_tier` description on `geometry_job_v1` and `asset_spec_v1` now says it is kit phase only.

Optional `place_feel_claim`:

| Kit phase | Legal claim |
|:---|:---|
| `smoke` | `harness_only` |
| `lod0` | `preview_ghost_stand_in` |
| `production` | `preview_ghost_same_mesh`, `operational_envelope` |
| any | `picker_icon` **rejected** |

Jobs and specs that omit `place_feel_claim` do not fail. Existing promoted GLBs are untouched.

```text
python -m rust_engine_mcp.cli place-feel-phase-note --tier lod0
python -m rust_engine_mcp.cli place-feel-phase-note --tier production --claim operational_envelope
```

Illegal pairs fail `validate-report` job/spec as `TIER-PLACE-001`. This is a note and a gate on **claims**, not a rebake.

---

## Handoff

| Owner | Next | This lane does not do it |
|:---|:---|:---|
| `@designer` | Ghost draw uses production mesh when the claim is `preview_ghost_same_mesh`; label lod0 stand-ins; split place-target vs mover on the 32px sheet | Art specs, bpy |
| `@coder` | Stop treating lod0 extract as Built (`BUILD-READ-VISUAL-001`) | New meshes |
| `@coder-mcp` | Style-pack resolve must not prefer same-pack lod0 over production for an operational claim | HUD |
| `@designer-mcp` | Keep `TIER-PLACE-001` as the claim gate; G4 still requires `art_quality: keyframe_manual` before ship | Site paint, economy logistics |

Open: no icon-role field on the RON atlas yet (that file is `@designer` consumption). No second `development_tier` value for staging.
