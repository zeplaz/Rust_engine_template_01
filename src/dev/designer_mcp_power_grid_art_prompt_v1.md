# Designer + designer-mcp — power grid art prompt `v1`

**Art plan:** [`plan_power_grid_art_assets_v1.md`](plan_power_grid_art_assets_v1.md)  
**UX/sim:** [`plan_power_grid_construction_ux_v1.md`](plan_power_grid_construction_ux_v1.md) · [`plan_nuclear_power_failure_meltdown_v1.md`](plan_nuclear_power_failure_meltdown_v1.md)

---

## @designer — style + 2D read (no bpy)

### P0

```
1. DES-ART-UTILITY-STYLE-001
   design_utility_industrial_style_v1.md
   Substation yard, transformer pad, bus geometry language, warning/read colors
   Ref: power_substation_yard_site_v0.json · UiPalette gold/cyan

2. DES-ART-POWER-OVERLAY-001
   Amend design_infra_network_overlay_v1.md — damaged, destroyed, island dim strokes

3. DES-ART-POWER-GLYPHS-001
   Node + state glyph sheet: transformer, substation, tee, SCRAM, meltdown, diesel
   Output: assets/ui/infrastructure/power_glyphs_spec_v1.md + PNG keyframes

4. DES-ART-HUD-POWER-ICONS-001
   Build rail + tool sheet icons: line tool, L/M/H voltage, curved vs 90°
```

### P1

```
5. DES-ART-COAL-PLANT-001       — massing concept + yard zones
6. DES-ART-NUCLEAR-PLANT-001    — PWR dome, cooling towers, switchyard, diesel yard
7. DES-ART-LINE-STRUCTURES-001  — HV tower vs MV pole spacing/silhouette
8. DES-ART-VFX-GRID-001         — line cut flash, overload spark charter
9. DES-ART-VFX-NUCLEAR-001      — SCRAM vent, heat haze, meltdown column brief
10. DES-ART-PLANT-CARD-001      — gauges for diesel, core heat (layout only)
```

---

## @designer-mcp — MCP specs + production (no Tk)

### P0 (unblocks grid nodes)

```
DMCP-SPEC-SUBSTATION-YARD-001
  kit_substation_yard_production_001
  Footprint 4×3 · matches grid_substation.json · pilot site zones
  Modules: bus bay, fence, gravel, prop_transformer slot

DMCP-SPEC-TRANSFORMER-PAD-001
  prop_transformer_production_run001
  Replace lod0 stub · 1u pad · insulators readable @ 32px
```

### P1

```
DMCP-SPEC-COAL-PLANT-001        — utilities_coal_plant assembly
DMCP-SPEC-TOWER-HV-001          — lattice tower for HV lines
DMCP-SPEC-POLE-MV-001           — MV distribution pole
DMCP-SPEC-JUNCTION-TEE-001      — line tee prop
```

### P2 (after DES-ART-NUCLEAR-PLANT-001 signed)

```
DMCP-SPEC-NUCLEAR-PWR-001       — containment silhouette + aux buildings (Phase 1)
DMCP-VFX-SPEC-SPARK-001         — line cut spark (deterministic seed)
DMCP-VFX-SPEC-MELTDOWN-001      — smoke column (P3 sim gate)
DMCP-MAT-UTILITY-PACK-001       — 12 material profiles
```

---

## MCP workflow (each spec)

```text
1. designer style doc signed
2. AssetSpec JSON in assets/staging/specs/
3. validate-report mcp_spec / mcp_job
4. headless bpy → staging GLB
5. validate-report asset_glb
6. promote → assets/models/modules/
7. _module_index.ron update
8. witness JSON in debug_runs/art_pipeline/
```

---

## Rules

- **mcp-production-rules** on every job — seeded, no AI art
- **Reuse** stack_chimney_1u_production where possible for coal/nuclear stacks
- **Nuclear:** dome silhouette legible before interior detail
- **Lines P0:** overlay stroke OK — do not block line-draw tool on tower GLB
- **Iso tiles:** do not start until DES-ART-UTILITY-TILE-001 gate

```text
ΔWF→ DES-ART-UTILITY-STYLE-001 → DMCP-SPEC-SUBSTATION-YARD-001
```
