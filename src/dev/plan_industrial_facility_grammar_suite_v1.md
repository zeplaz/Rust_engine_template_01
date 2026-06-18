# PLAN-INDUSTRIAL-FACILITY-GRAMMAR-001 — power · process · site · APS iterate `v1`

```text
⟦SYMLANG⟧⟐v1  ◈PLAN
⟨ID⟩ PLAN-INDUSTRIAL-FACILITY-GRAMMAR-001
Date: 2026-06-18
Status: **SIGNED** (@planner)
Owner: @designer (research + IA) · @designer-mcp (content) · @coder-mcp (APS tools) · @coder (engine read)
Parent: $ref:src/dev/plan_designer_work_202606_v1.md · $ref:src/dev/plan_aps_grammar_evolution_v1.md
```

**Headline:** The engine already models **power, supply chains, and activation** — but **building grammars are visual-only**. Artists cannot iterate “what this factory needs” inside APS; coders cannot validate that a generated warehouse matches its `supply_chain_role` or power tier. This program **binds process research → facility site design → grammar content → suite tools**.

**North star:** Pick **Factory Cluster** in APS → see **power band, inputs/outputs, chain step, site zones** → generate → sweep proves massing **and** process fit → ship path registers the same semantics the sim activates.

**Rejected:** mega-factory JSON that collapses chains · grammar without catalog authority · APS-only fake power numbers · research in chat without structured brief JSON.

---

## 0. What exists today (honest)

| Layer | State | Path / note |
|:---|:---|:---|
| **Supply chains** | Authoritative | [`industrial_supply_chains.json`](../../assets/configs/industrial_supply_chains.json) — steps, power, produces/consumes |
| **Building catalog** | Per-step JSON | `assets/configs/buildings/*.json` — `supply_chain_role`, `power_consumption`, `utility_role` |
| **Activation bridge** | Shipped | [`industrial_activation_pipeline.md`](industrial_activation_pipeline.md) — role → ECS + grid load |
| **Visual grammar** | G0 singleton | `industrial_warehouse_v1.ron` — **no** process binding |
| **Site zone pilots** | 4 grids, 1 full pilot | `assets/configs/buildings/pilots/*_site_v0.json`, `logistics_rail_warehouse_pilot_v1.json` |
| **Grammar iterate loop** | Shipped | [`design_grammar_iterate_tooling_v1.md`](design_grammar_iterate_tooling_v1.md) — massing/roof only |
| **Power UX research** | Questions doc | [`power_damage_ui_persistence_v1.md`](../../docs/reference/designer_questions/production_economy/power_damage_ui_persistence_v1.md) |

**Gap:** three ontologies on disk — **visual grammar**, **site zone grid**, **process catalog** — with **no join contract** and **no APS iterate surface**.

---

## 1. Three-layer facility model (do not collapse)

```text
Layer 1 — BuildingGrammar     massing · roof · facade · modules   (exists)
Layer 2 — FacilitySiteGrammar  zones · adjacency · yard % · rail spur  (pilots only)
Layer 3 — ProcessBinding       supply_chain_role · power · I/O tags    (catalog only)
```

**Join rule:** Layer 3 is **authority** (catalog + chains JSON). Layer 1–2 **express** Layer 3 visually — never invent power or inputs in grammar alone.

```text
industrial_supply_chains.json
        ↓
building catalog JSON (catalog_id)
        ↓
facility_binding on grammar / pilot (references catalog_id + site_template)
        ↓
APS Facility Needs strip + site preview
        ↓
grammar_eval_sweep (+ process histogram)
        ↓
engine activation (existing bridge)
```

---

## 2. Agent split (designer ↔ coder)

| Phase | @designer | @designer-mcp | @coder-mcp | @coder |
|:---|:---|:---|:---|:---|
| **Research** | Process facility briefs, power tiers, site zone taxonomy, APS IA | Reference packs, chain layout sketches | — | — |
| **Schema** | Sign-off on binding block + APS panels | RON/JSON examples, pilot expansion | Schema + validators + CLI brief | Optional Rust loader |
| **Iterate tools** | Facility Needs strip, site preview wireframes | Content sweeps, pilot sites | APS panels, `grammar-facility-brief`, sweep extension | — |
| **Ship gate** | Operator rubric for complex sites | G1+ archetypes with binding | Coverage + facility parity witness | Activation smoke |

**Handoff rule:** designer delivers **structured brief** → designer-mcp fills **catalog-linked content** → coder-mcp ships **tools that read the same JSON** → coder wires **engine consumers** only after validator green.

---

## 3. Track E1 — Research & design charter (@designer)

### E1-A — Process facility research

| ID | Deliverable | Research scope |
|:---|:---|:---|
| **DES-INDUSTRIAL-RESEARCH-001** | `design_industrial_process_research_v1.md` | Concrete Portland chain (mine→kiln→mixer): typical footprints, power step-up, adjacency |
| **DES-INDUSTRIAL-RESEARCH-002** | extend v1 § aluminum chain | Bauxite→refinery→smelter→fab: heavy power asymmetry (22→200), buffer yards |
| **DES-INDUSTRIAL-RESEARCH-003** | extend v1 § utility yard | Substation, transformer pad, coal plant — **utility_role** not supply_chain |

**Output shape (each chain section):**

| Field | Example |
|:---|:---|
| `chain_id` | `concrete_portland` |
| `steps[]` | role, typical W×D, power band, must_adjacent_to |
| `site_zone_requirements` | loading wing, utility %, buffer min |
| `visual_cues` | stacks, pipe racks, substation yard, rail spur |
| `grammar_archetype_hint` | `FactoryCluster` / `IndustrialWarehouse` / utility pilot |

**Refs:** [`industrial_supply_chains.json`](../../assets/configs/industrial_supply_chains.json) · [`recovery_construction.md`](recovery_construction.md) § aluminum + power.

### E1-B — Site zone & power taxonomy

| ID | Deliverable |
|:---|:---|
| **DES-FACILITY-SITE-ZONE-001** | `design_facility_site_zone_taxonomy_v1.md` — zone ids (primary, loading, utility, rail, service, parking, buffer); required vs optional per archetype |
| **DES-POWER-TIER-001** | `design_power_tier_bands_v1.md` — bands map designer units → APS glyph + grammar detail density |

**Power tier bands (draft — designer confirms):**

| Tier | Designer units | Typical roles | Visual grammar hint |
|:---|:---:|:---|:---|
| **Light** | 0–30 | mine, mixer, parking | low stack density |
| **Medium** | 31–80 | kiln, refinery, fab | pipe racks, vent modules |
| **Heavy** | 81–200 | smelter | dedicated utility yard, cooling read |
| **Grid** | utility infra | plant, substation | yard_complex, transformer modules |

### E1-C — Utility & logistics grammar (concepts)

| ID | Deliverable |
|:---|:---|
| **DES-UTILITY-GRAMMAR-001** | Substation yard + tank farm concept sheets (pilots exist — formalize) |
| **DES-LOGISTICS-SITE-001** | Rail warehouse site read — tie to `logistics_rail_warehouse_pilot_v1.json` |

---

## 4. Track E2 — Schema & content (@designer-mcp)

### E2-A — `facility_binding` contract

| ID | Owner | Deliverable |
|:---|:---|:---|
| **DES-FACILITY-BINDING-001** | @designer-mcp + @designer sign-off | `design_facility_binding_schema_v1.md` + schema patch proposal |

**Proposed optional block on `BuildingGrammar`:**

```ron
facility_binding: (
    catalog_id: "concrete_cement_kiln",
    chain_id: "concrete_portland",
    supply_chain_role: "cement_kiln",
    power_tier: "medium",  // derived from catalog; override forbidden
    site_template_id: "cement_kiln_yard_v0",  // optional Layer 2
    program_axes: ( storage: "low", loading: "medium", office: "low" ),
)
```

**Rule:** `power_consumption` always loaded from catalog at tool time — grammar stores **tier label** for sweep/UI only.

### E2-B — G1 archetypes with binding

| ID | Grammar | Binding target |
|:---|:---|:---|
| **DMCP-GRAM-FACTORY-CLUSTER-001** | `factory_cluster_v1.ron` | `concrete_mixer_plant` or generic `factory` stub |
| **DMCP-GRAM-RAIL-EDGE-001** | `rail_edge_v1.ron` | `logistics_rail_warehouse_pilot_v1` |
| **DMCP-PILOT-CONCRETE-SITE-001** | 3-step site layout JSON | concrete chain steps as zone clusters |
| **DMCP-PILOT-ALUMINUM-SITE-001** | 4-step site layout JSON | aluminum chain + heavy power yard |

### E2-C — Module kit for process read

| ID | Deliverable |
|:---|:---|
| **DMCP-MODULE-PROCESS-READ-001** | Module whitelist for stacks, pipes, substation pads, cooling — maps to power tier |

---

## 5. Track E3 — APS iteration tools (@designer IA → @coder-mcp)

**Authority:** [`design_grammar_iterate_tooling_v1.md`](design_grammar_iterate_tooling_v1.md) — extend loop, do not fork.

### E3-A — Designer IA (spec only)

| ID | Deliverable |
|:---|:---|
| **DES-APS-FACILITY-NEEDS-001** | Facility Needs strip — when archetype selected: chain step, power tier glyph, top 3 inputs/outputs, link to catalog |
| **DES-APS-SITE-PREVIEW-001** | Two-level preview: building footprint + site zone grid overlay (collapsed default G0, promoted G2) |
| **DES-APS-CHAIN-BROWSER-001** | Read-only chain diagram in APS Advanced — pick step → pre-fill archetype + district |

### E3-B — Coder-mcp tools

| ID | Tool / CLI | Purpose |
|:---|:---|:---|
| **CMCP-GRAMMAR-FACILITY-BRIEF-001** | `grammar-facility-brief [--grammar-id]` | Join grammar + catalog + chain → JSON brief |
| **CMCP-SITE-ZONE-VALIDATE-001** | `validate-report site_zone_grid <path>` | Zone % vs taxonomy; orphan zones |
| **CMCP-FACILITY-NEEDS-PANEL-001** | APS `FacilityNeedsStrip` | Renders brief fields; no invented numbers |
| **CMCP-SITE-PREVIEW-PANEL-001** | APS site grid canvas | Reads `site_zone_grid_v1` + pilot JSON |
| **CMCP-GRAM-SWEEP-PROCESS-001** | Extend `grammar_eval_sweep` | Histogram: power_tier, role, zone coverage |
| **CMCP-CHAIN-PILOT-GENERATE-001** | `chain-site-pilot --chain-id` | Emit starter site JSON from research template |

**Extended iterate loop:**

```text
1. designer_grammar_quality_loop (fast)
2. grammar-facility-brief --grammar-id <id>
3. Edit grammar RON + site pilot + catalog (designer-mcp)
4. validate-report site_zone_grid <site.json>
5. grammar_eval_sweep --archetype <id> --district <style>  (+ process histogram)
6. designer_grammar_quality_loop --full --write-witness
7. grammar-set-tier --write-witness
```

**Witness paths:**

| Path | When |
|:---|:---|
| `debug_runs/grammar_facility_brief_live.json` | After brief CLI |
| `debug_runs/site_zone_validate_live.json` | After site validate |
| `debug_runs/grammar_sweep_process_live.json` | After process sweep |

---

## 6. Track E4 — Engine consumer (@coder, gated)

| ID | Task | Gate |
|:---|:---|:---|
| **COD-FACILITY-BINDING-READ-001** | Rust: optional `facility_binding` on grammar types; **read-only** for debug HUD | CMCP brief green |
| **COD-BUILD-READ-PROCESS-001** | Extend build grammar read HUD with chain step + power tier (human labels) | DES-BUILD-READ-HUD-002 |
| **COD-SITE-MULTI-PLACE-001** | Multi-building site placement from site grid | **Future** — spec only until construction site API stable |

**Do not pick COD-SITE-MULTI-PLACE-001** until Track E3 site preview operator-signed.

---

## 7. Priority order

```text
P0  DES-INDUSTRIAL-RESEARCH-001 + DES-FACILITY-SITE-ZONE-001 + DES-POWER-TIER-001
P0  DES-FACILITY-BINDING-001 (schema sign-off)
P1  CMCP-GRAMMAR-FACILITY-BRIEF-001 + DES-APS-FACILITY-NEEDS-001
P1  DMCP-GRAM-FACTORY-CLUSTER-001 (with binding)
P2  CMCP-SITE-ZONE-VALIDATE-001 + DES-APS-SITE-PREVIEW-001 + CMCP-SITE-PREVIEW-PANEL-001
P2  DMCP-PILOT-CONCRETE-SITE-001
P3  CMCP-GRAM-SWEEP-PROCESS-001 + COD-FACILITY-BINDING-READ-001
```

---

## 8. Success metrics

| Metric | Target |
|:---|:---|
| Grammars with `facility_binding` | ≥2 at G1 |
| APS shows real catalog power/IO | 100% when binding present |
| Site zone pilots validated | ≥3 chains or utilities |
| Process sweep in quality loop | `green: true` on full loop |
| Operator can explain smelter yard | G-PLAY industrial rubric pass |

---

## 9. Dependencies

| Upstream | This program |
|:---|:---|
| [`plan_aps_grammar_evolution_v1.md`](plan_aps_grammar_evolution_v1.md) | Tier exposure for site preview promotion |
| [`plan_designer_work_202606_v1.md`](plan_designer_work_202606_v1.md) Track C | Style bibles feed visual cues in research |
| [`industrial_activation_pipeline.md`](industrial_activation_pipeline.md) | Layer 3 authority unchanged |
| [`design_grammar_archetype_family_g1_v1.md`](design_grammar_archetype_family_g1_v1.md) | FactoryCluster / RailEdge content |
| [`plan_power_grid_construction_ux_v1.md`](plan_power_grid_construction_ux_v1.md) | Line draw between transformer nodes · island/repair UX |
| [`plan_nuclear_power_failure_meltdown_v1.md`](plan_nuclear_power_failure_meltdown_v1.md) | Nuclear LOOP/SCRAM/meltdown when offsite power lost (grid islanding) |

---

## Changelog

| Ver | Date | Note |
|:---|:---|:---|
| v1.0.0 | 2026-06-18 | Initial — research → binding → APS tools → engine read |

```text
⟦/PLAN-INDUSTRIAL-FACILITY-GRAMMAR-001⟧  ΔWF→@designer E1 · @coder-mcp E3-B
```
