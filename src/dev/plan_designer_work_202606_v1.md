# PLAN-DESIGNER-WORK-202606-001 — art · style · APS · product UX `v1`

```text
⟦SYMLANG⟧⟐v1  ◈PLAN
⟨ID⟩ PLAN-DESIGNER-WORK-202606-001
Date: 2026-06-17
Status: **SIGNED** (@planner)
Owner: @designer (IA/UX) · @designer-mcp (art/content) · @coder-mcp (APS impl)
Parent index: $ref:src/dev/development_plan_index.md
```

**Headline:** APS UI/UX overhaul closed at **8/10** — machine guards green, but the tool still reads **incremental, not pro-grade**. Grammar content is **G0** while chrome assumes G2+. Art lanes need a **unified visual language** for buildings, modules, landscape, and materials — not one-off pilot specs.

**North star:** An artist opens APS or plays the sim and feels **one studio** — same words, same status language, same density, same preview fidelity ladder, same style families from concept → module → tile → in-engine read.

**Rejected:** replatform off Tk · AI-generated final art · exposing every grammar axis at G0 · pixel sign-off without operator rubric.

---

## 0. Current status (honest)

| Surface | Score / state | Gap |
|:---|:---|:---|
| **APS UI/UX overhaul** | CLOSED · 8/10 artist accept | N1 status_atom not everywhere · N2 preview async partial · N3 assembly strip confirm · landscape atlas **ship:false** |
| **Design system** | `aps_design_system_v1.md` **LOCKED** | Implementation drift — literals, three status dialects linger in edge panels |
| **Grammar maturity** | **G0** (1 archetype) | Tier exposure **signed** — content does not match chrome depth |
| **Landscape art** | Pilot 3-tile LG-5 · catalog on disk | Expanded matrix **chartered** · bake **not production-ship** |
| **Building art** | Rowhouse production v1 · module kit partial | Victorian/industrial **style packs** thin · concept sheets missing |
| **Materials** | Category tree v1 · profiles growing | 300+ profile **browse UX** + swatch/a11y not signed for scale |
| **Sim HUD** | SIM-HUD product closed | Minimap veg legend **designed, not UI-wired** · ecology burn merge pending @coder B |

---

## 1. Five designer tracks (parallel OK)

```text
Track A — APS professional polish     (@designer → @coder-mcp)
Track B — Art production spine        (@designer-mcp → @coder-mcp)
Track C — Style & concept charter     (@designer + @designer-mcp)
Track D — In-engine product UX        (@designer → @coder / @coder B)
Track E — Industrial facility grammar (@designer research → @designer-mcp content → @coder-mcp tools)
Track F — In-game HUD Phase 2       (@designer reflection → @coder Bevy/egui polish)  ★ build menus P0
Track G — Power grid construction   (@designer line UX → @coder draw/commit → combat/repair P2)
```

**Gate:** Track A **P0** unblocks APS; Track F **sim HUD**; Track G **power lines** — INFRA-E4 completion + [`plan_power_grid_construction_ux_v1.md`](plan_power_grid_construction_ux_v1.md).

---

## Track A — APS UI/UX professional polish

**Authority:** [`aps_design_system_v1.md`](aps_design_system_v1.md) · sign-off notes [`design_aps_uiux_overhaul_signoff_v1.md`](design_aps_uiux_overhaul_signoff_v1.md)

### A1 — Design system v1.1 (close implementation gaps)

| ID | Owner | Deliverable | Unblocks |
|:---|:---|:---|:---|
| **DES-APS-DS-V11-001** | @designer | `design_aps_design_system_v11_delta_v1.md` — status_atom migration map (every panel → helper) | OVR-P5-TAIL-001 |
| **DES-APS-INTERACTION-001** | @designer | Interaction spec: primary-action feedback, disabled reason placement, spine click affordance, list selection persist | coder-mcp polish sprint |
| **DES-APS-SMOOTHNESS-001** | @designer | **Smoothness charter** (Tk-realistic): no modal dead-ends · inline spinners · 300ms debounce rules · log scroll cap · focus return after generate | artist 9/10 target |

**Smoothness principles (Tk ceiling):**

| # | Rule | Artist feel |
|:---:|:---|:---|
| 1 | Every primary button → immediate inline ack (`⟳` at button, not log-only) | Responsive |
| 2 | Disabled actions show **why** adjacent, not silent | Trustworthy |
| 3 | Tab switch preserves selection where possible | Stable |
| 4 | Pipeline spine: current step `▣` + status glyph — never two “current” indicators | Oriented |
| 5 | No layout jump on validate (reserve banner height) | Calm |

### A2 — Preview & onboarding tail (overhaul notes N2, N4)

| ID | Owner | Deliverable |
|:---|:---|:---|
| **DES-APS-PREVIEW-V2-001** | @designer | 4-state preview contract: clean / night / damaged / burning — async-on-select, labelled thumbs, empty-state copy |
| **DES-APS-ONBOARD-SPEC-002** | @designer | Formalize `design_aps_uiux_onboard_outline_v1.md` → full spec (first 10s path) |
| **DES-APS-OPERATOR-RUBRIC-002** | @designer | Pixel walk checklist v2 — MIN window, landscape lane, preview thumbs (operator sign) |

### A3 — Grammar-tier UX (active program)

| ID | Owner | Deliverable | Ref |
|:---|:---|:---|:---|
| **DES-APS-GRAM-TIER-004** | @designer | G1 empty states + kit-hint off copy when ≥3 archetypes | [`design_aps_grammar_tier_exposure_v1.md`](design_aps_grammar_tier_exposure_v1.md) |
| **DES-APS-PREVIEW-LADDER-001** | @designer | Preview fidelity ladder G0→G4 (wireframe → massing → materials → variants) | grammar evolution plan |
| **DES-APS-MANUAL-FALLBACK-001** | @designer | Collapsed “Manual footprint” lane — when shown, when deprecated banner | tier exposure |

**Machine queue:** [`aps_grammar_evolution_queue.json`](../tools/orchestrator/queues/aps_grammar_evolution_queue.json)

---

## Track B — Art production spine (@designer-mcp)

### B1 — Landscape LG-5 (production)

| ID | Owner | Deliverable | Status |
|:---|:---|:---|:---|
| **DMCP-LG5-EXPAND-BAKE-001** | @designer-mcp | Execute expanded matrix bake per [`design_landscape_lg5_expansion_matrix_v1.md`](design_landscape_lg5_expansion_matrix_v1.md) | **done** — Phase A Q✓ · Phase B G4 manual |
| **DMCP-LG5-KEYFRAME-QC-001** | @designer-mcp | Keyframe still QC per [`design_landscape_keyframe_burn_reqs_v1.md`](design_landscape_keyframe_burn_reqs_v1.md) | **done** — teach tier PASS WITH NOTES · G4 corridor regrowth manual |
| **DMCP-VEG-ATLAS-SHIP-001** | @designer-mcp | G4/G5 art-ship criteria sign-off when atlas registers | blocks engine LG-5 consumer |

### B2 — Building tiles & modules

| ID | Owner | Deliverable |
|:---|:---|:---|
| **DMCP-TILE-ROWHOUSE-V2-001** | @designer-mcp | Rowhouse variant completeness — damage + burning frames operator-visible | **done** — 14 variants Q✓ |
| **DMCP-MODULE-KIT002-001** | @designer-mcp | kit_production_002 concept + module manifest sketch (planner-mcp unfreeze) | **done** — concept Q✓ · G4 open |
| **DMCP-ATLAS-QC-PLAIN-002** | @designer-mcp | Plain-language QC copy v2 for warehouse/shopfront/bunker batches |

### B3 — Materials library at scale

| ID | Owner | Deliverable |
|:---|:---|:---|
| **DES-APS-MAT-BROWSE-001** | @designer | Materials tab IA for 300+ profiles — tree + search + recents + “unsorted” bucket |
| **DES-APS-MAT-SWATCH-001** | @designer | Swatch grid spec — glyph+word status, not color-alone ([`design_aps_color_a11y_audit_v1.md`](design_aps_color_a11y_audit_v1.md)) |
| **DMCP-MAT-PROFILE-PILOT-002** | @designer-mcp | 24-profile pilot pack per category tree leaf (industrial steel, residential brick, …) | **done** — spec_only pack on disk |

---

## Track C — Model styles & concept charter

**Goal:** One **visual language document** per lineage — informs grammar archetypes, module kits, tile bakes, and APS preview.

### C1 — Style family bibles

| ID | Owner | Deliverable | Feeds |
|:---|:---|:---|:---|
| **DES-STYLE-INDUSTRIAL-WEST-001** | @designer | Style bible: massing, roof, door/window rhythm, palette, weathering — **concept sheet + 3 refs** | IndustrialWarehouse grammar · kit002 |
| **DES-STYLE-VICTORIAN-ROW-001** | @designer | Style bible for rowhouse production v1 — bay rhythm, brick bands, night windows | tile_rowhouse_victorian |
| **DES-STYLE-LANDSCAPE-RIparian-001** | @designer-mcp | Riparian/agri visual language — canopy mass, edge softness, burn read | landscape presets |
| **DES-STYLE-ISO-READ-001** | @designer | Global iso readability rules — silhouette, roof legibility @ 64px, fire read @ operational zoom | all atlases |

### C2 — Grammar / archetype content (G1)

| ID | Owner | Deliverable | Ref |
|:---|:---|:---|:---|
| **DES-GRAM-ARCHETYPE-FACTORY-001** | @designer-mcp | `FactoryCluster` or `RailEdge` spec — massing strategies + district | **done** via DMCP-GRAM-ARCHETYPE-FACTORY-001 |
| **DES-GRAM-ARCHETYPE-CIVIC-001** | @designer-mcp | Third archetype — civic/commercial seed | **done** — CivicBlock concept Q✓ · RON GRAM-CONTENT-005 |
| **DES-STYLE-PACK-REGISTRY-001** | @designer | Map `style_pack_id` → visual bible → module whitelist → tile batch | `_module_index` |

### C3 — Concept workflow (artist-facing)

| ID | Owner | Deliverable |
|:---|:---|:---|
| **DES-CONCEPT-WORKFLOW-001** | @designer | Concept → module → tile promotion checklist (what ships in APS vs staging only) |
| **DES-CONCEPT-THUMBNAIL-001** | @designer | Thumbnail grid spec for APS Catalog/Modules — aspect, labels, missing-module placeholder |

---

## Track D — In-engine product UX (@designer)

| ID | Owner | Deliverable | Coder consumer |
|:---|:---|:---|:---|
| **DES-MINIMAP-VEG-LEGEND-002** | @designer | Wire spec: legend UI (not tint-only) + burn scar tokens | CDR-B VEG-MINIMAP-LEGEND-UI-001 |
| **DES-ECOLOGY-PREVIEW-V2-001** | @designer | World preview ecology panel — topology + burn frame read | ecology preview |
| **DES-BUILD-READ-HUD-002** | @designer | Building grammar read HUD v2 — DNA/β human labels in sim | [`design_build_grammar_read_hud_v1.md`](design_build_grammar_read_hud_v1.md) |
| **DES-G-PLAY-OPERATOR-V2-001** | @designer | Operator veg/fire checklist v2 for G-PLAY-OPERATOR-01 | play acceptance |
| **DES-S7B-INTEL-MINIMAP-001** | @designer | Stage 7 intel overlay on minimap — recon/logistics stress (D-S7-02) | S7B-M3 |

**Charter:** [`guide_landscape_grammar_v1.md`](guide_landscape_grammar_v1.md) · [`ui_boundary_guide_v1.md`](../prompts/guides/ui_boundary_guide_v1.md)

---

## Track E — Industrial facility grammar (power · process · site)

**Authority:** [`plan_industrial_facility_grammar_suite_v1.md`](plan_industrial_facility_grammar_suite_v1.md) · [`industrial_supply_chains.json`](../../assets/configs/industrial_supply_chains.json) · [`design_grammar_iterate_tooling_v1.md`](design_grammar_iterate_tooling_v1.md)

**Problem:** Visual grammars and runtime supply chains are **disconnected** — artists cannot see power/inputs when generating; site zone pilots exist but do not iterate in APS.

### E1 — Research (@designer)

| ID | Deliverable |
|:---|:---|
| **DES-INDUSTRIAL-RESEARCH-001** | Process facility research — concrete + aluminum + utility yards |
| **DES-FACILITY-SITE-ZONE-001** | Site zone taxonomy (primary, loading, utility, rail, …) |
| **DES-POWER-TIER-001** | Power tier bands → APS glyph + module density |

### E2 — APS IA (@designer → @coder-mcp)

| ID | Deliverable |
|:---|:---|
| **DES-APS-FACILITY-NEEDS-001** | Facility Needs strip — chain step, power, I/O from catalog |
| **DES-APS-SITE-PREVIEW-001** | Building footprint + site zone grid overlay |
| **DES-APS-CHAIN-BROWSER-001** | Chain step picker → archetype pre-fill |

### E3 — Content (@designer-mcp)

| ID | Deliverable |
|:---|:---|
| **DES-FACILITY-BINDING-001** | `facility_binding` schema on BuildingGrammar (catalog authority) | **done** — schema v1 on disk |
| **DMCP-GRAM-FACTORY-CLUSTER-001** | `factory_cluster_v1.ron` with binding | **done** — all 3 G1 grammars bound |
| **DMCP-PILOT-CONCRETE-SITE-001** | Multi-step concrete site layout JSON | **done** — 3-step `concrete_portland_chain_pilot_v1` |

### E4 — Tools (@coder-mcp · after specs signed)

| ID | Tool |
|:---|:---|
| **CMCP-GRAMMAR-FACILITY-BRIEF-001** | Join grammar + catalog + chain |
| **CMCP-SITE-ZONE-VALIDATE-001** | Site zone grid validator |
| **CMCP-FACILITY-NEEDS-PANEL-001** | APS Facility Needs strip |
| **CMCP-GRAM-SWEEP-PROCESS-001** | Process histogram in eval sweep |

**Prompts:** [`industrial_facility_grammar_prompts_v1.md`](industrial_facility_grammar_prompts_v1.md)

---

## Track F — In-game HUD professional polish (sim product)

**Authority:** [`plan_sim_hud_professional_polish_v1.md`](plan_sim_hud_professional_polish_v1.md) · [`design_sim_hud_reflection_audit_v1.md`](design_sim_hud_reflection_audit_v1.md)  
**Prior close:** SIM-HUD-PRODUCT-001 (2026-06-03) — witnesses green; **player polish gap** remains.

### F1 — Reflection & build menus (P0)

| ID | Deliverable |
|:---|:---|
| **DES-SIM-HUD-COHESION-001** | Bevy/egui parity charter |
| **DES-SIM-HUD-BUILD-PICKER-001** | Rail-anchored build picker sheet (replaces sloppy submenus) |
| **DES-SIM-HUD-TRAY-BUILD-001** | Context tray Build tab — legend, staging, queue |
| **DES-SIM-HUD-POPUP-TIERS-001** | Popup migration map (no ad-hoc anchors) |
| **DES-SIM-HUD-COPY-REGISTRY-001** | Single sim HUD copy registry |

### F2 — Ops / overlays / session (P1–P2)

| ID | Deliverable |
|:---|:---|
| **DES-SIM-HUD-OPS-002** | Ops strip v2 |
| **DES-SIM-HUD-OVERLAY-002** | Info panel IA |
| **DES-SIM-HUD-MINIMAP-002** | Minimap legend dock |
| **DES-SIM-HUD-PAUSE-002** | Pause menu polish |

**Coder consumers:** COD-SIM-HUD-BUILD-PICKER-001 · COD-SIM-HUD-TRAY-BUILD-001 · COD-SIM-HUD-POPUP-MIGRATE-001 · COD-SIM-HUD-EGUI-THEME-001

**Prompt:** [`designer_sim_hud_prompt_v1.md`](designer_sim_hud_prompt_v1.md)

---

## Track G — Power grid construction (lines · islanding · repair)

**Authority:** [`plan_power_grid_construction_ux_v1.md`](plan_power_grid_construction_ux_v1.md) · [`design_power_line_construction_ux_v1.md`](design_power_line_construction_ux_v1.md)  
**Infra:** INFRA-E4 utility graph · road/rail path UX pattern · gold overlay [`design_infra_network_overlay_v1.md`](design_infra_network_overlay_v1.md)

| ID | Deliverable |
|:---|:---|
| **DES-POWER-LINE-TOOL-SHEET-001** | Tool sheet — curved/90°, voltage, snap, commit |
| **DES-POWER-ROUTING-MODE-001** | Curved vs orthogonal routing rules |
| **DES-POWER-VOLTAGE-PICKER-001** | Low / MV / HV picker + mismatch copy |
| **DES-POWER-MAP-OVERLAY-002** | Line states: live, preview, damage, island |
| **DES-POWER-TARGETING-001** | Cut line / transformer KO preview (P2) |
| **DES-POWER-REPAIR-PANEL-001** | Repair queue UX (P2) |

**Coder:** COD-POWER-LINE-DRAW-001 · COMMIT · ORTHOGONAL/SPLINE routers · island toast

**Prompt:** [`designer_power_grid_prompt_v1.md`](designer_power_grid_prompt_v1.md)

---

## Track H — Power grid art & assets

**Authority:** [`plan_power_grid_art_assets_v1.md`](plan_power_grid_art_assets_v1.md)

| Lane | Owner | P0 deliverables |
|:---|:---|:---|
| **Style + glyphs** | @designer | Utility style bible · overlay glyphs · HUD power icons |
| **Modules** | @designer-mcp | Substation yard · transformer pad GLB |
| **Nuclear massing** | @designer → @designer-mcp | PWR concept → containment kit (P1) |
| **VFX** | @designer + @designer-mcp | Grid spark · SCRAM/meltdown charter |

**Prompt:** [`designer_mcp_power_grid_art_prompt_v1.md`](designer_mcp_power_grid_art_prompt_v1.md)

---

## 2. Priority order (next 8 weeks — conceptual)

```text
Week 1–2  A1 DS v1.1 + E1 industrial research + site zone taxonomy
Week 2–3  B1 LG-5 expand bake + E2 facility_binding schema sign-off
Week 3–4  C2 G1 archetypes (designer-mcp) + CMCP grammar-facility-brief
Week 4–5  A1 interaction/smoothness + E2 APS Facility Needs strip spec
Week 5–6  E3 concrete site pilot + C1 victorian bible
Week 6–7  D minimap legend wire + E2 site preview panel spec
Week 7–8  Operator rubric v2 + CMCP process sweep in quality loop
```

---

## 3. Success metrics

| Metric | Target |
|:---|:---|
| APS artist ship score | **9/10** (from 8) after A1+A2 tail |
| Grammar tier on disk | **G1** (≥3 archetypes) |
| Landscape LG-5 expanded atlas | **ship:true** in registry |
| Design system violations | **0** ban-list · **0** font floor failures |
| Visible status dialects | **1** (`status_atom` only) |
| Sim ecology @ minimap | legend UI wired + burn override visible |
| Grammars with facility_binding | ≥2 at G1 |
| APS shows catalog power/IO | when binding present |

---

## 4. Agent routing

| Agent | Pick | Do not pick |
|:---|:---|:---|
| **@designer** | Track A P0 + Track C style bibles + Track D wire specs + **Track E research/IA** + **Track F sim HUD P0** | Python/Tk · Rust |
| **@designer-mcp** | Track B bakes + Track C archetype RON + **Track E binding content/pilots** | Bevy HUD code |
| **@coder-mcp** | Implements signed APS specs + **Track E brief/validate/sweep tools** | Design authority |
| **@coder** | Track E engine read + **Track F HUD wire (picker, tray, theme)** | Design authority |
| **@coder B** | Track D after designer wire specs land | APS Tk |
| **Operator** | DES-APS-OPERATOR-RUBRIC-002 pixel walks | — |

---

## 5. Machine queue seeds

Add to `designer_active_queue.json` active section:

- DES-APS-DS-V11-001 (P0)
- DES-APS-SMOOTHNESS-001 (P0)
- DES-APS-PREVIEW-V2-001 (P1)
- DES-STYLE-INDUSTRIAL-WEST-001 (P1)
- DES-APS-MAT-BROWSE-001 (P1)
- DES-MINIMAP-VEG-LEGEND-002 (P2)
- DES-INDUSTRIAL-RESEARCH-001 (P0 — Track E)
- DES-FACILITY-SITE-ZONE-001 (P0 — Track E)
- DES-POWER-TIER-001 (P0 — Track E)
- DES-APS-FACILITY-NEEDS-001 (P1 — Track E)
- DES-SIM-HUD-BUILD-PICKER-001 (P0 — Track F)
- DES-SIM-HUD-COHESION-001 (P0 — Track F)
- DES-SIM-HUD-TRAY-BUILD-001 (P0 — Track F)
- DES-POWER-LINE-TOOL-SHEET-001 (P0 — Track G)
- DES-POWER-ROUTING-MODE-001 (P0 — Track G)

---

## Changelog

| Ver | Date | Note |
|:---|:---|:---|
| v1.3.0 | 2026-06-18 | Track G — power grid line construction + strategic read |
| v1.0.0 | 2026-06-17 | Initial designer backlog — APS polish + art + style + sim UX |

```text
⟦/PLAN-DESIGNER-WORK-202606-001⟧  ΔWF→@designer A1 · @designer-mcp B1+C2
```
