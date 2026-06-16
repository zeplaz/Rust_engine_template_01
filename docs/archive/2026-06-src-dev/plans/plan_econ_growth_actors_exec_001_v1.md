# PLAN-ECON-GROWTH-ACTORS-001 — Growth actors + market niche exec `v1`

| Field | Value |
|:---|:---|
| **Queue ID** | **PLAN-ECON-GROWTH-ACTORS-001** |
| **Parent** | [`construction_economy_growth_vision_v1.md`](construction_economy_growth_vision_v1.md) |
| **Organic exec** | [`plan_organic_growth_exec_001_v1.md`](plan_organic_growth_exec_001_v1.md) — **extends OG-1** (rich fields) |
| **Settlement** | [`plan_settlement_hierarchy_exec_005_v1.md`](plan_settlement_hierarchy_exec_005_v1.md) |
| **PG-1** | [`plan_procedural_build_gen_exec_001_v1.md`](plan_procedural_build_gen_exec_001_v1.md) |
| **UX** | [`design_organic_growth_ux_v1.md`](design_organic_growth_ux_v1.md) |
| **Validation** | [`plan_validation_runtime_v1.md`](plan_validation_runtime_v1.md) · skill **validation-first** |
| **Version** | `1.0.0` |
| **Date** | 2026-06-02 |
| **Owner** | `@planner` → `@coder B` (primary) |
| **Status** | **SIGNED — READY** |
| **Horizon** | **2 weeks** (3 PRs after SET-P5-001 + PG-1 partial) |

**Problem:** [`plan_organic_growth_exec_001_v1.md`](plan_organic_growth_exec_001_v1.md) OG-1 defines **stub** `DistrictMetrics` + scalar `DevelopmentPressure`. Product vision requires **who builds what** (state vs private infill), **employment/housing deficits**, and **market saturation** so commercial proposals compete for niches — not infinite corner shops.

**Rule:** All commits still funnel through `ConstructionPlanQueue` → `SiteConstructionPhase::Planned`. Actor layer changes **who may enqueue** and **proposal priority/suppression** — not spawn authority.

---

## 1. Actor layers (authoritative)

| Layer | `GrowthActorLayer` | Who commits | Examples | Enqueue path |
|:---|:---|:---|:---|:---|
| **State / player** | `State` | Player execute (+ scenario seeds) | Factory, power, water, rail, admin, military | Zone + parametric → **CON-P2** |
| **Growth (sim)** | `Growth` | Approve / auto policy | Shop, apartments, warehouse, school | **OG-2** proposal → queue |
| **Market (sim-only)** | `Market` | *never commits* | Saturation scoring, duplicate suppression | OG-1 tick → modifies priority |
| **Catalog legacy** | `LegacyCatalog` | Migration alias | Existing `BuildingDefinition` rows | PG-1 `catalog_id` until retired |

```rust
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum GrowthActorLayer {
    State,
    Growth,
    // Market is not a commit layer — scoring only
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum BuildingUsage {
    Government,
    Industrial,
    Commercial,
    Residential,
    Office,
    Military,
    Civic,      // schools, clinics — policy-gated growth
    Logistics,  // warehouse, depot — freight-driven
}
```

**Mapping v1:** `BuildingUsage` on `BuildingArchetype` (PG-1). `GrowthActorLayer::State` usages: `Government`, `Industrial` (player factories), `Military`. `Growth` usages: `Commercial`, `Residential`, `Office`, `Civic`, `Logistics` (workshop-scale).

**Invariant G-ACTOR-01:** `Market` layer **never** writes `ConstructionPlanQueue` directly.

---

## 2. Extended metrics (OG-1 rich)

Extends stub structs in [`construction_procedural_buildings_plan_v1.md`](construction_procedural_buildings_plan_v1.md) § B.

```rust
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct DistrictMetrics {
    // --- existing OG-1 stub ---
    pub population_density: f32,
    pub employment_density: f32,
    pub wealth: f32,
    pub desirability: f32,
    pub transport_access: f32,
    pub services: f32,
    pub pollution: f32,
    pub crime: f32,
    // --- PLAN-ECON-GROWTH-ACTORS extensions ---
    pub employment_demand: f32,   // jobs wanted − jobs filled (normalized 0..1)
    pub housing_deficit: f32,     // population − housing capacity (normalized)
    pub freight_access: f32,      // graph reach to rail/port (0 until INFRA-E1-004)
    pub utility_service: f32,     // power + water coverage 0..1
    pub civic_pressure: f32,      // schools/clinics deficit (policy)
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct DevelopmentPressure {
    pub residential: f32,
    pub commercial: f32,
    pub industrial: f32,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct MarketSaturation {
    /// Per-archetype count in district / cap from district rules
    pub by_archetype: HashMap<ArchetypeId, SaturationCell>,
    /// Usage-level rollup for UX + suppression
    pub by_usage: HashMap<BuildingUsage, f32>, // 0 = empty niche, 1 = saturated
}

pub struct SaturationCell {
    pub count: u32,
    pub cap: u32,
    pub saturation: f32, // count / cap clamped 0..1
}
```

**Compute order (sim tick):**

```text
1. Roll up sites in district → employment_demand, housing_deficit
2. Read utility + transport stubs (boolean → scalar v1)
3. compute_district_pressure_system → DevelopmentPressure
4. compute_market_saturation_system → MarketSaturation
5. (OG-2) score proposals using pressure × (1 - saturation) × desirability
```

---

## 3. Employment + housing hooks (economy spine)

| Signal | Source (v1) | Source (later) |
|:---|:---|:---|
| `employment_demand` | `TownRecord.jobs` vs `DistrictMetrics.employment_density` rollup | activation / logistics throughput |
| `housing_deficit` | `TownRecord.housing` vs population proxy | civic housing archetypes count |
| `freight_access` | stub `1.0` if district has rail edge within N tiles | `TransportGraph` + INFRA-E1-004 |
| `utility_service` | AND of power + water service flags on district rect | `UtilityConnection` graph (INFRA E4) |

**Files (read-only v1):**

- `src/strategic/settlement/town.rs` — `TownBook` rollup
- `src/economy/activation/` — **do not** duplicate factory sim; read `IndustrialFacilityActivated` count per district for employment **filled**
- `src/construction/building_definitions.rs` — alias `BuildingFamily` → `BuildingUsage` until PG-1 lands

**Anti-pattern:** second economy solver for growth — growth **reads** activation counts, does not run recipes.

---

## 4. Market saturation + niche competition

### 4.1 Caps (district rules)

Extend `DistrictStyleRules` (settlement plan §5) with optional caps:

```ron
(
    allowed_archetypes: ["corner_shop", "grocery", "warehouse"],
    archetype_caps: (
        corner_shop: 2,
        grocery: 1,
        warehouse: 3,
    ),
    usage_caps: (
        Commercial: 4,
        Residential: 12,
    ),
)
```

Default cap when omitted: `usage_caps.Commercial = 3`, `Residential = 8` (designer tunable in RON).

### 4.2 Suppression rule

When scoring a candidate `GrowthProposal`:

```text
priority_base = w_r * pressure.residential + w_c * pressure.commercial + ...
niche_factor  = 1.0 - saturation.by_usage[proposal.usage]
priority      = priority_base * niche_factor * transport_access * utility_service
```

If `saturation.by_archetype[id].saturation >= 1.0` → **reject** proposal (do not enqueue).

**UX copy (designer):** proposal card `Reason:` includes `"market saturated"` when suppressed — see [`design_organic_growth_ux_v1.md`](design_organic_growth_ux_v1.md) §3.

### 4.3 Competition narrative (acceptance)

Same district, high commercial pressure, **two** corner shops already → third shop proposal **suppressed** or **low priority**; grocery (different archetype) may still queue if under cap.

---

## 5. Growth proposal enrichment (OG-2 handoff)

Extend `GrowthProposal` (OG-2) fields — **this plan owns schema; OG-2 owns queue wiring**:

```rust
pub struct GrowthProposal {
    pub district_id: DistrictId,
    pub block_id: Option<BlockId>,
    pub archetype_id: ArchetypeId,
    pub usage: BuildingUsage,
    pub actor_layer: GrowthActorLayer, // always Growth for auto proposals
    pub anchor_tile: IVec2,
    pub priority: f32,
    pub seed: u64,
    pub reason_codes: Vec<GrowthReasonCode>, // TransportHigh, EmploymentDemand, ...
    pub saturation_at_submit: f32,
}
```

`reason_codes` feed district inspector + proposal card — no LLM prose in sim.

---

## 6. Module layout

```
src/strategic/settlement/
  mod.rs
  district.rs          # DistrictMetrics, DevelopmentPressure (extended)
  pressure.rs          # compute_district_pressure_system
  market.rs              # NEW — MarketSaturation, compute_market_saturation_system
  actors.rs            # NEW — GrowthActorLayer, BuildingUsage, commit policy helpers
  growth.rs            # OG-2 — consumes saturation (separate PR)
```

**≤3 files per PR** — split below.

---

## 7. PR train

| PR | ID | Owner | Files | Exit |
|:---:|:---|:---|:---|:---|
| 1 | **ECON-OG-1-A** | B | `actors.rs`, `district.rs` (types), `mod.rs` | `BuildingUsage` + extended metrics compile; unit tests for serde |
| 2 | **ECON-OG-1-B** | B | `pressure.rs`, `market.rs` | pressure + saturation tick; test: 3 shops → 4th suppressed |
| 3 | **ECON-OG-1-C** | B | `live_proof.rs`, witness JSON | keys below green |

**Blocked by:**

| Gate | Need |
|:---|:---|
| **SET-P5-001** | `DistrictBook` + `DistrictRecord.style_rules` |
| **PG-1 partial** | `ArchetypeId` + `BuildingUsage` on archetype RON (can stub 3 archetypes) |
| **CON-P2-001** | commit → `Planned` (growth must not skip stages) |

**Parallel OK:** ECON-OG-1-A types-only while SET-P5-001 in flight (use test fixtures).

**Does not block:** MCP art lane, INFRA graph (freight stays stub until E1-004).

---

## 8. Witness schema (validation-first)

Agents consume **ValidationReport-style JSON** in witness — not raw test stdout.

| Pointer | Type | Pass when |
|:---|:---|:---|
| `/construction_organic_growth_001/pressure_wired` | bool | `DevelopmentPressure` non-zero in fixture district |
| `/construction_organic_growth_001/employment_demand_wired` | bool | metric updates when jobs rollup changes |
| `/construction_organic_growth_001/market_saturation_active` | bool | duplicate commercial suppressed |
| `/construction_organic_growth_001/growth_market_saturation_active` | bool | alias for dashboard — same as above |
| `/construction_organic_growth_001/proposals_queued` | number | OG-2 (may be 0 in OG-1-only PR) |
| `/construction_organic_growth_001/execute_via_pipeline` | bool | no Operational on proposal enqueue |
| `/construction_organic_growth_001/green` | bool | rollup |

**Verification (coders):**

```powershell
cargo test -p proc_A_dine01 --lib organic_growth settlement market_saturation
python -m rust_engine_mcp.cli validate-report bevy -p proc_A_dine01 --compress 3
```

Extend witness writer only — hand-edit `debug_runs/construction_stage_live.json` forbidden.

---

## 9. Scenario matrix (lib fixture)

| District fixture | Setup | Expect |
|:---|:---|:---|
| `mixed_use_high_transport` | jobs > housing, transport 0.9 | `commercial` pressure > `residential` |
| `commercial_saturated` | 3 commercial sites at cap | 4th archetype proposal **rejected** |
| `industrial_freight` | rail stub flag | `Logistics` usage priority boost |
| `no_utilities` | utility_service 0 | all growth proposals priority → 0 |

---

## 10. MCP / art / tile lanes (consumers only)

| Lane | This plan |
|:---|:---|
| **PG-2 modules** | Commercial/residential **archetypes** reference module ids — not actor logic |
| **validation-first** | Witness + `validate-report bevy` after each PR |
| **tile-generation** | **Out of scope** — district pressure overlays are sim HUD, not orthographic tile bake |

Planner documents witness keys only — no bpy in this exec.

---

## 11. Anti-patterns

| Don't | Do |
|:---|:---|
| 500 building JSON content strategy | Archetype + caps in district RON |
| Growth spawns `Operational` | Queue → CON-P2 stages |
| Separate economy for auto-build | Read activation + town rollup |
| LLM-generated proposal reasons | `GrowthReasonCode` enum |
| Duplicate `Town` in infrastructure | G-TOWN-ONE |
| Infinite commercial proposals | `MarketSaturation` reject |

---

## 12. Coder handoff

| After merge | Unblocks |
|:---|:---|
| ECON-OG-1-C green | **PROC-OG-2-001** rich proposals + queue |
| + SET-P5-002 | District picker + block scoping |
| + PG-1 | Full archetype catalog |

**Pull order:** SET-P5-001 → ECON-OG-1-A/B/C → PROC-OG-2-001 → PROC-OG-3-001 (UX PASS already).

---

## 13. Changelog

| Version | Date | Notes |
|:---|:---|:---|
| v1.0.0 | 2026-06-02 | Actor layers, market saturation, employment hooks; extends OG-1 |
