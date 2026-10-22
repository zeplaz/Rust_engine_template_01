# Dual view — Tactical icons + Pixel world `v1`

| Field | Value |
|:---|:---|
| **⟨ID⟩** | **PLAN-DUAL-VIEW-PIXEL-WORLD-001** |
| **Date** | 2026-10-17 |
| **Owner** | `@planner` (charter) · `@coder` (mode/toggle/scaffold) · `@designer-mcp` / `@coder-mcp` (APS home) |
| **Status** | **MACHINE_GREEN_v6** — ship building atlases + massing zoom collapse + veg/cover PixelWorld |
| **Witness** | [`debug_runs/dual_view_pixel_world_live.json`](../debug_runs/dual_view_pixel_world_live.json) |
| **Art home** | [`design_dual_view_art_home_v1.md`](design_dual_view_art_home_v1.md) |
| **Related** | [`design_ebbtide_build_style_rubric_v1.md`](design_ebbtide_build_style_rubric_v1.md) · **DR-VOXEL-OCCUPATION** |

## Product lock

| Mode | Presentation | Art |
|:---|:---|:---|
| **TacticalFlat2D** (default) | Icons + symbols (occupation, phase, strategic icons, abstract ghosts) | Not APS ship target |
| **PixelWorld3D** | 3D-pixel world — landscape/env first | **Primary APS + Blender ship target** |

Same sim map RTT hole ([`sim_map_rtt`](../gui/tactical/sim_map_rtt.rs)). Toggle on sim product shell. One `WorldPresentationMode` writer. One `RepresentationResult` writer.

## Authority

```text
HUD toggle ─▶ WorldPresentationMode (sole ResMut)
                ├─ TacticalFlat2D → icon/symbol consumers
                └─ PixelWorld3D   → PIXEL_WORLD_VIEW_SCAFFOLD → landscape/env → RepresentationResult band
```

Forbidden mode writers: `ViewManager`, `ViewProjectionAuthority`, `RepresentationResult`, `ResolvedViewports`.

## Conflict matrix

| Peer | Relation |
|:---|:---|
| WorldPreview GPU | BLOCK — editor only; never alias for play |
| Second gameplay RTT | REJECT |
| Strategic icons | PARALLEL OK under Tactical |
| VoxelMassing presentation | COORDINATE — later into PixelWorld, not tactical APS dump |
| DR-VOXEL-OCCUPATION | SERIAL — sim voxels deferred |
| RTT / VFX lane | COORDINATE file ownership |

## Scaffold

```text
PIXEL_WORLD_VIEW_SCAFFOLD {
  owner: "gui/world_presentation_mode",
  intended_replacement: "RepresentationResult → RenderProjectionGraph pixel-world band",
  exit_condition: "landscape/env FULL_APP witness green under PixelWorld3D + APS ship packs",
  removal_trigger: "parallel extract OR APS targeting tactical as primary",
}
```

**Deferrals:** [`plan_deferral_registry_v1.md`](plan_deferral_registry_v1.md) — **DR-VOXEL-OCCUPATION** · **DR-PIXEL-WORLD-APS-MESH**.

## v1 exit (machine)

- [x] Mode + Tactical/World toggle; default Tactical
- [x] `PIXEL_WORLD_VIEW_SCAFFOLD` + `PixelWorldConsumerSystemSet`
- [x] Landform height/slope via `PixelWorldLandscapeIntent` → same RTT
- [x] Tactical icons policy + art-home charter
- [x] Landscape grammar presets witness green
- [x] `ArtPresentationTarget` + `PixelWorldLandscapePackIndex`
- [x] VoxelMassing columns gated to PixelWorld only
- [x] 10 ship landscape grammar presets tagged + seeded into `PixelWorldLandscapePackIndex`
- [x] `PixelWorldLandscapeConsume` + LG-5 atlas stamp queue into same RTT
- [x] `queue_pixel_world_landscape_atlas_stamps` — real LG-5 PNG blit (no stub)
- [x] Building iso atlas stamps (`stamp_request_for_site`) gated to PixelWorld only
- [x] Veg/cover packs tagged `presentation: pixel_world` + LG-5 burn/scar/regrowth stamp modulation
- [x] Ship building atlases registered (`PIXEL_WORLD_BUILDING_SHIP_ATLASES`) + index `presentation: pixel_world`
- [x] VoxelMassing collapses to floors=1 below 24 px/tile
- [x] `terrain_honesty_lint` run on GPU terrain path (art-home checklist #7)
- [ ] Extra building module-kit content waves (product / operator G4)
