# PLAN-POWER-GRID-ART-ASSETS-001 — art · modules · VFX · HUD `v1`

```text
⟦SYMLANG⟧⟐v1  ◈PLAN
⟨ID⟩ PLAN-POWER-GRID-ART-ASSETS-001
Date: 2026-06-18
Status: **SIGNED** (@planner)
Owner: @designer (charter + HUD glyphs) · @designer-mcp (specs + QC) · @coder-mcp (MCP bake)
Parent programs:
  $ref:src/dev/plan_power_grid_construction_ux_v1.md
  $ref:src/dev/plan_nuclear_power_failure_meltdown_v1.md
  $ref:src/dev/plan_industrial_facility_grammar_suite_v1.md
```

| **Downstream queue** | [`power_grid_art_downstream_queue.json`](../tools/orchestrator/queues/power_grid_art_downstream_queue.json) |
| **Downstream dispatch** | [`power_grid_art_downstream_dispatch_v1.md`](../tools/orchestrator/queues/power_grid_art_downstream_dispatch_v1.md) |

**Headline:** Power grid gameplay needs **visible authority** — lines, nodes, damage, nuclear states — not gold strokes alone. Catalog JSON and site pilots exist; **most utility GLBs are missing or lod0-only**. This plan sequences **design → MCP module jobs → overlay/VFX → HUD icons** in lockstep with sim phases P1–P3.

**North star:** Player recognizes **transmission vs distribution**, **live vs cut vs islanded**, and **SCRAM vs meltdown** without reading logs.

**Rejected:** AI-generated final art · power lines as only 2D overlay forever · one generic “power plant” mesh for coal and nuclear · skipping lod0 audit before production kit.

---

## 0. Asset inventory (honest gap)

| Asset / role | On disk today | Gap |
|:---|:---|:---|
| **Grid substation** | `grid_substation.json` — no dedicated module kit | Needs yard + bus art |
| **Distribution transformer** | `grid_distribution_transformer.json` | `prop_transformer` index entry — **production GLB path TBD** |
| **Coal plant** | `utilities_coal_plant.json` | **No promoted module / assembly** |
| **Nuclear PWR** | `pwr_4loop_1100mw_v1` in plant_definitions only | **No building catalog + no containment art** |
| **Substation yard pilot** | `power_substation_yard_site_v0.json` + mock shape | Site zones only — **no ship GLB** |
| **Power lines** | Overlay stroke `#e8c040` 2px | **No 3D towers/poles/cables** · no break mesh |
| **Chimney / stack modules** | `stack_chimney_1u_production` ✓ | Reuse for coal/nuclear cooling read |
| **HUD icons** | Build rail generic utilities | **No line tool / voltage / SCRAM / meltdown glyphs** |
| **VFX** | Fire/sparks reference folders | **No grid spark · cut flash · meltdown column** |
| **Iso tiles** | Rowhouse/industrial tiles | **No utility plant tile batch** (defer unless iso-first view) |

---

## 1. Art lanes (parallel OK)

```text
Lane A — Utility module kit (3D)       @designer-mcp → @coder-mcp MCP jobs
Lane B — Power line props (3D/LOD)      @designer-mcp · stroke fallback P0
Lane C — Map overlay + node glyphs      @designer → @coder B compositor
Lane D — HUD + icon atlas               @designer → @coder
Lane E — VFX + nuclear state read       @designer + @designer-mcp → @coder
Lane F — Style + materials              @designer → material profiles
Lane G — Iso / tile (optional P3)       @designer-mcp · only if product needs iso read
```

**Gate:** Lane C **P0** (overlay states) unblocks line-draw UX before Lane B production towers ship.

---

## 2. Lane A — Utility module kit (MCP)

**Authority:** [`blender-geometry`](../../.cursor/skills/blender-geometry/SKILL.md) · [`mcp-production-rules`](../../.cursor/skills/mcp-production-rules/SKILL.md) · `_module_index.ron`

### A1 — Substation & transformer (P0 — unblocks grid play)

| ID | Owner | Deliverable | Feeds |
|:---|:---|:---|:---|
| **DES-ART-UTILITY-STYLE-001** | @designer | `design_utility_industrial_style_v1.md` — substation yard, transformer pad, bus read | all utility modules |
| **DMCP-SPEC-SUBSTATION-YARD-001** | @designer-mcp | AssetSpec → `kit_substation_yard_production_001` | `grid_substation.json` |
| **DMCP-SPEC-TRANSFORMER-PAD-001** | @designer-mcp | Production `prop_transformer` replace lod0 stub | `grid_distribution_transformer.json` |
| **DMCP-SPEC-SUBSTATION-4X3-001** | @designer-mcp | Full assembly from `power_substation_yard_v0` pilot | build rail utilities |

**Module categories (whitelist):**

| Category | Modules | Power tier cue |
|:---|:---|:---|
| Yard void | fence, gravel pad, warning sign | — |
| Primary | bus bars, breaker bays (simplified) | MV |
| Service | control shack, relay cabinet | — |
| Props | `prop_transformer`, cable drum, oil containment berm | MV/LV |

### A2 — Generation sites (P1 — coal before nuclear)

| ID | Owner | Deliverable |
|:---|:---|:---|
| **DES-ART-COAL-PLANT-001** | @designer | Coal plant massing concept — stack, boiler hall, coal yard |
| **DMCP-SPEC-COAL-PLANT-001** | @designer-mcp | `utilities_coal_plant` assembly spec |
| **DES-ART-NUCLEAR-PLANT-001** | @designer | PWR containment read — dome, cooling towers, diesel yard, switchyard |
| **DMCP-SPEC-NUCLEAR-PWR-001** | @designer-mcp | Containment + auxiliary building kit (Phase 1 silhouette) |

**Nuclear art states (design — sim drives variant):**

| State | Visual (3D + icon) |
|:---|:---|
| Operational | Normal steam plume (light) |
| SCRAM | Reduced plume · amber status light |
| Diesel running | Diesel exhaust + yard lights |
| Cooling degraded | Extra vent steam · heat shimmer |
| Meltdown | Dark smoke column + red status (VFX Lane E) |
| Breach | Radiation zone overlay (WSS) |

### A3 — Line support structures (P1–P2)

| ID | Owner | Deliverable |
|:---|:---|:---|
| **DES-ART-LINE-STRUCTURES-001** | @designer | HV tower vs MV pole vs LV street — spacing + silhouette rules |
| **DMCP-SPEC-TOWER-HV-001** | @designer-mcp | Lattice tower module (HV transmission) |
| **DMCP-SPEC-POLE-MV-001** | @designer-mcp | Single-circuit pole (MV distribution) |
| **DMCP-SPEC-JUNCTION-TEE-001** | @designer-mcp | Line tee / splice box prop |

**P0 fallback:** compositor **stroke + pole glyph** only — 3D props land P1 without blocking COD-POWER-LINE-DRAW.

---

## 3. Lane B — Power line rendering

| Phase | Render | Art |
|:---|:---|:---|
| **P0** | GPU overlay centerline (existing gold stroke) + pole **glyph** at control points | Designer glyph spec only |
| **P1** | Instanced pole/tower meshes at spans | DMCP specs A3 |
| **P2** | Damaged segment: broken mesh + spark VFX anchor | Lane E |

**Curved vs 90° read:**

| Mode | Map read |
|:---|:---|
| Curved | Smooth stroke + towers at max span |
| 90° | Axis-aligned stroke + corner **insulator** glyph at bends |

Ref: [`design_power_line_construction_ux_v1.md`](design_power_line_construction_ux_v1.md) §4.

---

## 4. Lane C — Overlay & node glyphs (@designer)

| ID | Deliverable |
|:---|:---|
| **DES-ART-POWER-OVERLAY-001** | Extend [`design_infra_network_overlay_v1.md`](design_infra_network_overlay_v1.md) — damaged/destroyed/island strokes |
| **DES-ART-POWER-GLYPHS-001** | Node glyph sheet: transformer, substation, tee, plant, **SCRAM**, **meltdown** |
| **DES-ART-ISLAND-READ-001** | Dim + boundary edge for grid island (electrical) |

**Deliverable format:** PNG keyframe stills + vector spec in `assets/ui/infrastructure/` (create folder).

---

## 5. Lane D — HUD & build rail icons

| ID | Owner | Deliverable |
|:---|:---|:---|
| **DES-ART-HUD-POWER-ICONS-001** | @designer | Icon atlas brief: line tool, voltage tiers (L/M/H), routing mode (curve/90°), diesel, SCRAM |
| **COD-ART-HUD-ICON-ATLAS-001** | @coder | Register in `icon_atlas.rs` / build rail |
| **DES-ART-PLANT-CARD-001** | @designer | Plant focus card layout + gauge art (core heat, diesel fuel) |

Align sim HUD Phase 2: [`plan_sim_hud_professional_polish_v1.md`](plan_sim_hud_professional_polish_v1.md).

---

## 6. Lane E — VFX & nuclear state (@designer + MCP)

| ID | Owner | Deliverable | Trigger |
|:---|:---|:---|:---|
| **DES-ART-VFX-GRID-001** | @designer | Grid overload spark · line cut flash · transformer KO flash | overload / damage |
| **DES-ART-VFX-NUCLEAR-001** | @designer | SCRAM vent · core heat haze · meltdown column · breach glow | nuclear state machine |
| **DMCP-VFX-SPEC-SPARK-001** | @designer-mcp | Reuse elemental_sparks charter for **line cut** anchor | damage |
| **DMCP-VFX-SPEC-MELTDOWN-001** | @designer-mcp | Smoke column + ember falloff spec (deterministic seed) | P3 sim |
| **COD-VFX-WIRE-001** | @coder | Hook VFX to `NuclearScramEvent` / segment damage | after sim P1 |

**Ref:** [`assets/vfx/reference/elemental_sparks/README.md`](../../assets/vfx/reference/elemental_sparks/README.md)

---

## 7. Lane F — Materials & style

| ID | Owner | Deliverable |
|:---|:---|:---|
| **DES-ART-UTILITY-MAT-001** | @designer | Material profiles: galvanized steel, ceramic insulator, concrete pad, warning paint |
| **DMCP-MAT-UTILITY-PACK-001** | @designer-mcp | 12-profile pilot in `material_profiles_v1.json` |
| **DES-STYLE-UTILITY-ISO-001** | @designer | Iso silhouette rules @ 64px — stacks, dome, tower legibility |

Feeds [`plan_industrial_facility_grammar_suite_v1.md`](plan_industrial_facility_grammar_suite_v1.md) DMCP-MODULE-PROCESS-READ-001.

---

## 8. Lane G — Iso tiles (defer gate)

**Only if** product confirms iso-first utility read (like rowhouse production).

| ID | Condition | Deliverable |
|:---|:---|:---|
| **DES-ART-UTILITY-TILE-001** | Gate: iso viewport proof | Tile states: coal day/night, substation clean/damaged |
| **DMCP-TILE-COAL-PLANT-001** | After DMCP-SPEC-COAL-PLANT | Keyframe bake per [`design_tile_bake_spine_convergence_v1.md`](design_tile_bake_spine_convergence_v1.md) |

**Default:** 3D module assemblies in sim view — **iso lane P3 optional**.

---

## 9. Phasing vs sim programs

| Sim phase | Art minimum ship |
|:---|:---|
| **Power P0 — line draw** | Overlay strokes + pole glyphs + HUD line icon |
| **Power P1 — commit/graph** | Transformer + substation production GLB · MV/HV stroke weights |
| **Power P2 — damage/repair** | Cut spark VFX · damaged stroke · repair scaffold on segment |
| **Nuclear P1 — LOOP/SCRAM** | SCRAM icon · amber plant badge · diesel yard prop (small) |
| **Nuclear P2 — heat crisis** | Heat haze · rising gauge art |
| **Nuclear P3 — meltdown** | Meltdown column VFX · breach zone read |

---

## 10. Agent routing

| Agent | Pick |
|:---|:---|
| **@designer** | Lanes C, D, F style docs, E VFX charters, A1 style bible |
| **@designer-mcp** | Lane A specs, E MCP VFX specs, G tiles if gated |
| **@coder-mcp** | bpy jobs, validate, promote, registry |
| **@coder** | HUD atlas, overlay compositor, VFX spawn hooks |
| **@coder B** | Minimap/compositor stroke polish |

**Prompt:** [`designer_mcp_power_grid_art_prompt_v1.md`](designer_mcp_power_grid_art_prompt_v1.md)

---

## 11. Success metrics

| Metric | Target |
|:---|:---|
| Substation + transformer production GLB | **ship:true** in `_module_index` |
| Coal plant assembly | catalog-backed visual |
| Nuclear PWR silhouette | readable @ tactical zoom |
| Line tool HUD icon | on build rail |
| SCRAM / meltdown | distinct glyphs — not recolored fire |
| Overlay damaged vs live | distinguishable greyscale |

---

## Changelog

| Ver | Date | Note |
|:---|:---|:---|
| v1.0.0 | 2026-06-18 | Initial art/assets plan for power + nuclear programs |

```text
⟦/PLAN-POWER-GRID-ART-ASSETS-001⟧  ΔWF→@designer DES-ART-UTILITY-STYLE-001 · @designer-mcp DMCP-SPEC-SUBSTATION-YARD-001
```
