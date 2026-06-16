# Petroleum industry simulation runbook `v1`

> **STATUS:** Draft **v1** — design scaffold. Implementation **Pending**.

Version: `v1.0.0`  
Parent: [`simulation_expansion_orchestrator_v1.md`](simulation_expansion_orchestrator_v1.md)  
UI companion: [`ui/petroleum_industry_ui_snippet_v1.md`](ui/petroleum_industry_ui_snippet_v1.md)

---

## 1. Purpose

Treat **oil as a strategic industrial ecosystem**, not “generic liquid fuel.” Model **extraction quality**, **refining capability**, **infrastructure vulnerability**, **logistics**, **military targeting**, **seasonal/market shifts**, and **AI-relevant bottlenecks**.

---

## 2. Core design principle

**Crude oil is feedstock, not fuel.** **Refineries** determine **output composition**, **efficiency**, and **strategic capability** (aviation, diesel, chemicals, etc.).

---

## 3. Industrial pipeline

```
Oil deposit
  → Extraction
  → Storage
  → Transport
  → Refining
  → Fraction allocation
  → Strategic fuel distribution
  → Consumers (civilian / military / export)
```

---

## 4. Crude dimensions (gameplay variables)

| Property | Gameplay |
|:---|:---|
| Sulfur content | Refinery complexity |
| API gravity | Yield profile |
| Viscosity | Pumping / energy |
| Contamination | Maintenance |
| Wax content | Cold-weather issues |
| Extraction difficulty | Infrastructure cost |
| Instability | Accident / fire risk |

### Example component sketch

```rust
#[derive(Component, Reflect)]
#[reflect(Component)]
pub struct CrudeOilDeposit {
    pub crude_type: CrudeType,
    pub reserve_million_barrels: f32,
    pub pressure: f32,
    pub sulfur_content: f32,
    pub api_gravity: f32,
    pub extraction_difficulty: f32,
    pub contamination_factor: f32,
}
```

### Crude family enum (illustrative)

`LightSweet`, `HeavySour`, `OilSandsBitumen`, `ShaleOil`, `OffshoreDeepCrude`, `SyntheticCrude`, `Condensate`, …

---

## 5. Extraction infrastructure

| Infrastructure | Typical use |
|:---|:---|
| Pumpjack | Shallow conventional |
| Enhanced recovery | Depleted fields |
| Steam injection | Oil sands |
| Offshore platform | Deep water |
| Fracking | Shale |
| Arctic systems | Cold regions |

**Weather** affects freeze risk, storms (offshore), heat losses, flooding, sandstorms (maintenance).

**Decline:** fields deplete, need reinvestment, may water out or lose pressure.

---

## 6. Field failure / logistics backup

If export fails: **full storage** → **blocked pipelines** → **pressure backup** → **stress** → **shutdown / damage**. This creates **strategic targeting value** for pipelines, ports, refineries.

---

## 7. Refinery capabilities (not generic buildings)

Sketch:

```rust
pub struct RefineryProfile {
    pub sulfur_processing_capacity: f32,
    pub cracking_efficiency: f32,
    pub heavy_oil_support: bool,
    pub catalytic_reformer: bool,
    pub hydrocracker: bool,
    pub coking_capacity: f32,
}
```

Simple refineries: light sweet only. Advanced: heavy sour / bitumen / contaminated feed — at cost (power, chemicals, labor, maintenance).

---

## 8. Fractional outputs

| Fraction | Uses |
|:---|:---|
| LPG | Heating |
| Naphtha | Chemicals |
| Gasoline | Civilian transport |
| Kerosene | Jet |
| Diesel | Logistics / military |
| Heavy fuel oil | Ships / plants |
| Lubricants | Industry |
| Asphalt | Construction |
| Coke | Metallurgy |

**Allocation:** player / AI policies shift **military vs civilian vs export vs chemical** emphasis (e.g. wartime: more diesel + jet).

---

## 9. Fuel ecosystem

Different platforms consume **different fractions** (truck diesel, jet kerosene, ship bunker, etc.). **Strategic gap:** large **reserves** but **no refining depth** for aviation or military diesel — intentional gameplay.

---

## 10. Pipelines & export (active infrastructure)

Segment sketch:

```rust
pub struct PipelineSegment {
    pub throughput: f32,
    pub pressure: f32,
    pub leak_risk: f32,
    pub integrity: f32,
}
```

Failures: sabotage, freeze, overload, bombing — each ties to **weather**, **warfare**, and **chunk/LOD** updates (not decorative).

---

## 11. Storage & reserves

Tank farms (vulnerable), caverns (protected), field buffering, military reserves — **policy + physics** (see UI snippet for player-facing controls).

---

## 12. Market & demand shifts

Demand moves with **season**, **war**, **economy**, **disasters**, **doctrine**, **sanctions**.

---

## 13. AI concerns

AI should reason about **refinery bottlenecks**, **export vulnerability**, **reserve depletion**, **fuel mix vs doctrine**, **redundant routes**. **Fuel capability ≠ crude ownership.**

---

## 14. ECS module layout (target)

```
systems/petroleum/
  deposits/
  extraction/
  pipelines/
  refining/
  storage/
  logistics/
  fuel_mix/
  strategic_reserves/
  market/
  damage/
  ai/
```

---

## 15. Field + LOD integration

**Do not** simulate every barrel in every frame.

- **Chunk / throughput fields** + **strategic statistical flow** for bulk.  
- **Detailed agents** for visible convoys, sabotage, accidents, shortages.  

Align with [`chunk_scheduler_runbook_v1.md`](chunk_scheduler_runbook_v1.md).

---

## 16. Warfare integration

Primary targets: **production**, **transport**, **reserves**, **refining**, **export**. Outcomes: aviation shortage, regional shortages, offensive slowdown, etc.

---

## 17. Cross-system integration table

| System | Interaction |
|:---|:---|
| Weather | Extraction + transport |
| Roads / rail | Logistics |
| Power | Refinery demand |
| Concrete | Asphalt / construction byproducts |
| AI | Strategy / targeting |
| Economy | Export dependency |
| Chunk fields | Regional flow |

---

## 18. Step packs

[`../matrix/simulation_expansion/runbook/README.md`](../matrix/simulation_expansion/runbook/README.md)
