# DamFlood APS meshes — shipped

**Owner:** `@designer-mcp`  
**Wave:** DF-4→DF-9→DF-10 companion art — batch `kit_dam_flood_aps_001`  
**Witness:** `debug_runs/art_pipeline/kit_dam_flood_aps_001_live.json`

## Shipped

Seeded GLB module kits for dam kinds already catalogued as buildings:

- `dam_embankment_v1` → `dam_embankment_v1_production_run001` (72 verts)
- `dam_gravity_v1` → `dam_gravity_v1_production_run001` (72 verts)
- `dam_weir_v1` → `dam_weir_v1_production_run001` (48 verts)
- `dam_arch_v1` → `dam_arch_v1_production_run001` (144 verts)

Pipeline: AssetSpec → MCP geometry job (`module_prop` + dam `prop_kind`) → validate GLB → promote → Bevy registry. Deterministic seeds (`42`+offset); material `concrete_grey_01`; no diffusion / pasted bpy as final art.

## Status

**SHIPPED** (G0–G5 pass). Engine DF-4 placement + barrier install remain mesh-independent for Accept; meshes are now available in `_module_index` for representation attach when the engine wire lands.
