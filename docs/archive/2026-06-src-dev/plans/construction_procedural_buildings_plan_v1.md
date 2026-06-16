# Procedural buildings + organic growth — architecture `v1`

| Field | Value |
|:---|:---|
| **Doc ID** | **CONSTRUCTION-PROC-GROWTH-001** |
| **Date** | 2026-05-27 |
| **Owner** | `@planner` |
| **Status** | **READY (planner finalized)** |
| **Parent** | [`construction_product_roadmap_phases_2_10_v1.md`](construction_product_roadmap_phases_2_10_v1.md) |
| **Invariants** | [`construction_invariants.md`](construction_invariants.md) |

**North star:** One **archetype + style pack + grammar** generates many buildings. Player **zones + infrastructure**; simulation **develops** districts — never 500 hand-authored catalog rows.

---

## Two systems (sequenced)

| System | Question it answers | Player role |
|:---|:---|:---|
| **A — Procedural assembly** | What mesh/layout is this building? | Picks zone / approves auto-build (optional) |
| **B — Organic growth** | What gets built where and when? | Zones land, builds roads/rail/power/water |

**Dependency:** Growth **queues** [`PlannedSite`](../../src/strategic/site/components.rs) / construction plans; assembly **instantiates** visuals after commit + stage pipeline (Phase 2).

---

## A — Module-based buildings (not House_500)

### Anti-pattern

```text
House_A.json … House_500.json  →  does not scale
```

### Target pattern

```text
BuildingArchetype (rules)
  + StylePack (meshes)
  + ProceduralBuildingRequest (width, depth, floors, style)
  → assembled segments (wall / window / door / roof / corner / prop)
  → RepresentationResult / GPU instance buffer
```

### Repo alignment

| Concept | Today | Target |
|:---|:---|:---|
| Catalog row | [`BuildingDefinition`](../../src/construction/building_definitions.rs) per JSON | **Archetype** + optional `catalog_id` alias |
| Family | [`BuildingFamily`](../../src/construction/building_catalog.rs) | maps to `BuildingUsage` |
| Footprint | [`FootprintMatrix`](../../src/construction/building_catalog.rs) | parametric + grammar layout |
| Shapes | [`_mock_shapes.ron`](../../assets/configs/buildings/_mock_shapes.ron) | seed for **footprint grammar**, not final art |
| Visual | Stage 5 extraction | **procedural extract** slot — no parallel LOD |

### Core types (sim authority — RON/JSON on disk)

```rust
pub enum BuildingUsage {
    Residential,
    Commercial,
    Industrial,
    Office,
    Government,
    Military,
}

pub struct BuildingArchetype {
    pub id: ArchetypeId,
    pub footprint: FootprintType,      // Rect | L | T | O | GrammarRef
    pub floors: std::ops::RangeInclusive<u32>,
    pub facade_sets: Vec<FacadeId>,
    pub roof_sets: Vec<RoofId>,
    pub usage: BuildingUsage,
    pub grammar: Option<GrammarRuleId>, // Phase B+ — shape grammar
}

pub struct ProceduralBuildingRequest {
    pub archetype: ArchetypeId,
    pub width: u32,
    pub depth: u32,
    pub floors: u32,
    pub style: StylePackId,
    pub seed: u64,
}

pub struct StylePack {
    pub id: StylePackId,
    pub label: String,                 // "Victorian", "Industrial Soviet", …
    pub wall_modules: Vec<ModuleId>,
    pub window_modules: Vec<ModuleId>,
    pub door_modules: Vec<ModuleId>,
    pub roof_modules: Vec<ModuleId>,
    pub corner_modules: Vec<ModuleId>,
    pub prop_sets: Vec<ModuleId>,
}

/// Runtime material instance variation — few tileable textures, many appearances.
pub struct MaterialVariation {
    pub hue_shift: f32,
    pub roughness_shift: f32,
    pub dirt_level: f32,
    pub wear_level: f32,
}

/// District-level rules (not mesh generation). OG-1/2 filter proposals.
pub struct DistrictStyleRules {
    pub allowed_archetypes: Vec<ArchetypeId>,
    pub allowed_roof_modules: Vec<ModuleId>,
    pub preferred_style_pack: StylePackId,
    pub style_tags: Vec<String>,       // port_district, railway_district, …
}
```

**Gameplay row** still commits via construction funnel; `ProceduralBuildingRequest` is **derived** at commit from ghost + district context — not a second spawn path.

### Assembly pipeline (render-only until Stage 5 contract)

```text
Commit site (sim)
  → ProceduralBuildingRequest resolved
  → generate_footprint_grid()   // W/D/W row grammar
  → generate_walls / windows / doors / roof / details
  → push to RepresentationResult building slice
```

**Phase A1 (coder):** data model + RON loaders + witness only — **no** runtime mesh yet.  
**Phase A2:** greybox module instancing (placeholder art).  
**Phase A3:** shape grammar (CityEngine-style rules).  

### Shape grammar (advanced — Phase A3)

```text
Residential → Apartment
Apartment   → Base + Floors + Roof
Base        → Door + WindowSet
Floors      → Repeat(WindowSet)
Roof        → FlatRoof | PitchedRoof
```

Same rules + different `StylePack` → Victorian vs Modern vs Military.

---

## B — Organic growth (Republic-style)

### Player loop

```text
Zone land (paint)
  → Build infrastructure (road, power, water, rail)
  → Simulation ticks DevelopmentPressure / DistrictMetrics
  → System queues PlannedSite rows (same execute funnel)
  → Optional: player approve / auto-build policy per district
```

### Pressure model

```rust
pub struct DevelopmentPressure {
    pub residential: f32,
    pub commercial: f32,
    pub industrial: f32,
}

pub struct DistrictMetrics {
    pub population_density: f32,
    pub employment_density: f32,
    pub wealth: f32,
    pub desirability: f32,
    pub transport_access: f32,
    pub services: f32,
    pub pollution: f32,
    pub crime: f32,
}
```

**Inputs:** population, jobs, land value, transport graph reach, services coverage, pollution, crime (stagger — start with transport + employment + zoning).

**Output:** `GrowthProposal { archetype, tile, usage, priority }` → `ConstructionPlanQueue` — **never** direct entity spawn.

**District rules (inbound):** proposals must satisfy `DistrictRecord.style_rules` — allowed archetypes + roof modules + preferred style pack. Districts emit **demand**, not Blender jobs.

### Hierarchy (growth reads at district level)

```text
Building → Block → District → Town → Region → State → Nation
```

Growth system reads **District** book; writes **Block**-scoped proposals; commit creates **Building** sites.

---

## Combined stack (beyond Republic)

```text
Procedural mesh assembly
  + District growth simulation
  + Economic demand (economy/logistics)
  + Transport accessibility (R8 / graph)
  + Style packs (designer)
  + GIS context (Phase 8 — terrain, trade routes)
```

Town **looks** and **grows** differently because of terrain, rail, industry, policy — not random prefab scatter.

---

## Deliverables map

| ID | Doc | Owner |
|:---|:---|:---|
| **DESIGN-PROC-MODULE-KIT-001** | [`design_procedural_module_kit_v1.md`](design_procedural_module_kit_v1.md) | @designer |
| **PLAN-PROC-BUILD-EXEC-001** | [`plan_procedural_build_gen_exec_001_v1.md`](plan_procedural_build_gen_exec_001_v1.md) | @planner → @coder |
| **DESIGN-ORGANIC-GROWTH-UX-001** | [`design_organic_growth_ux_v1.md`](design_organic_growth_ux_v1.md) | @designer |
| **PLAN-ORGANIC-GROWTH-EXEC-001** | [`plan_organic_growth_exec_001_v1.md`](plan_organic_growth_exec_001_v1.md) | @planner → @coder |
| **PLAN-ART-DESIGN-INBOUND-ALIGN-001** | [`plan_art_design_inbound_alignment_v1.md`](plan_art_design_inbound_alignment_v1.md) | @planner |
| **PLAN-DESIGNER-MCP-ART-TOOLCHAIN-001** | [`plan_designer_mcp_art_toolchain_exec_001_v1.md`](plan_designer_mcp_art_toolchain_exec_001_v1.md) | @planner → tools/mcp |

---

## Execution order (within product roadmap)

| Step | Lane |
|:---:|:---|
| 1–3 | Placement validation, construction pipeline, scaling audit (existing roadmap) |
| **4a** | Designer module kit (10×5 modules) |
| **4b** | Coder PG-1 archetype + StylePack RON |
| **5** | District / Town books |
| **6a** | Coder OG-1 pressure + queue proposals |
| **6b** | Designer growth UX (proposals, district identity) |
| **7+** | PG-2 mesh assembly, OG-2 auto-build policies, grammar |

---

## Anti-patterns

- 500 unique building JSON files as the primary content strategy
- Growth system spawning `Operational` entities without construction queue
- Procedural generator writing gameplay state (slab, economy) from render
- Separate military building catalog fork
- Instant zone paint → built structures without pipeline stages

---

## Verification

```powershell
cargo test -p proc_A_dine01 --lib construction procedural_build organic_growth
```

Witness targets (future): `construction_procedural_build_001`, `construction_organic_growth_001` in `construction_stage_live.json`.

---

## Changelog

| Version | Date | Notes |
|:---|:---|:---|
| v1.0.0 | 2026-05-27 | Module + growth architecture |
