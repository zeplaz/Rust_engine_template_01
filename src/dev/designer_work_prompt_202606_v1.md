# @designer — work prompt `v1` (art · style · APS · UX)

Plan: [`plan_designer_work_202606_v1.md`](plan_designer_work_202606_v1.md)  
Industrial facility track: [`plan_industrial_facility_grammar_suite_v1.md`](plan_industrial_facility_grammar_suite_v1.md) · [`industrial_facility_grammar_prompts_v1.md`](industrial_facility_grammar_prompts_v1.md)  
Design system authority: [`aps_design_system_v1.md`](aps_design_system_v1.md)

---

## Situation (one paragraph)

APS UI/UX overhaul is **closed at 8/10** — pytest guards green, pipeline spine landed, copy pack shipped — but artists still feel **patchwork polish**: status helpers not universal, preview async incomplete, grammar chrome deeper than **G0 content**, landscape atlas not **production-ship**, no unified **style bible**, and **building grammars ignore power/factory needs** even though supply chains and activation exist in the engine. Your job is **spec authority** (no Python).

---

## Pick order (designer — no @designer-mcp unless noted)

### P0 — This week (unblocks pro feel + complex buildings)

```
1. DES-INDUSTRIAL-RESEARCH-001  [Track E — parallel with 2–3 OK]
   design_industrial_process_research_v1.md
   Concrete + aluminum chains + utility yards: footprint, power bands, adjacency, visual cues
   Authority: assets/configs/industrial_supply_chains.json

2. DES-FACILITY-SITE-ZONE-001
   design_facility_site_zone_taxonomy_v1.md
   Zone matrix per archetype; ref pilots/*_site_v0.json

3. DES-POWER-TIER-001
   design_power_tier_bands_v1.md
   Designer units → tier → APS glyph + module density

4. DES-APS-DS-V11-001
   design_aps_design_system_v11_delta_v1.md — status_atom migration map
   Ref: design_aps_uiux_overhaul_signoff_v1.md notes N1–N3

5. DES-APS-SMOOTHNESS-001
   design_aps_smoothness_charter_v1.md — Tk-realistic interaction rules
```

### P1 — Next (style + preview + facility APS)

```
6. DES-APS-FACILITY-NEEDS-001
   Facility Needs strip — chain step, power tier, I/O (catalog authority, no invented numbers)
   Ref: plan_industrial_facility_grammar_suite_v1.md

7. DES-STYLE-INDUSTRIAL-WEST-001
   Style bible — feeds FactoryCluster + kit002

8. DES-APS-PREVIEW-V2-001
   4-state preview contract — async-on-select, labels

9. DES-APS-MAT-BROWSE-001
   Materials tab IA for 300+ profiles

10. DES-APS-GRAM-TIER-004
    G0/G1 empty states — amend design_aps_grammar_tier_exposure_v1.md
```

### P2 — Sim product + site preview (wire for @coder / @coder-mcp)

```
11. DES-APS-SITE-PREVIEW-001 — footprint + site zone grid overlay
12. DES-MINIMAP-VEG-LEGEND-002 — collapsible legend UI wire spec
13. DES-ECOLOGY-PREVIEW-V2-001 — world preview ecology panel
```

---

## @designer-mcp parallel lane

```
DES-FACILITY-BINDING-001       — facility_binding schema draft (catalog authority)
DMCP-GRAM-FACTORY-CLUSTER-001  — factory_cluster_v1.ron + binding
DMCP-PILOT-CONCRETE-SITE-001   — 3-step concrete site layout JSON
DMCP-LG5-EXPAND-BAKE-001       — landscape atlas per expansion matrix
```

---

## @coder-mcp (after designer specs signed)

```
CMCP-GRAMMAR-FACILITY-BRIEF-001  — join grammar + catalog + chain
CMCP-SITE-ZONE-VALIDATE-001      — site zone grid validator
CMCP-FACILITY-NEEDS-PANEL-001    — APS Facility Needs strip
CMCP-GRAM-SWEEP-PROCESS-001      — process histogram in eval sweep
```

Full split: [`industrial_facility_grammar_prompts_v1.md`](industrial_facility_grammar_prompts_v1.md)

---

## Rules

- **No Python** — specs, wireframes, style bibles, copy packs only
- **No gate IDs** in artist-visible strings (ban-list in design system §2)
- **One word per concept** — Assembly, Piece, Module, Building style, Layout graph
- **NEEDS-DISPLAY** items flag operator pixel walk — do not claim visual fix without rubric
- Cite `aps_design_system_v1.md` in every deliverable header

---

## Verify (designer)

- Each deliverable: PASS table + sign-off row
- Cross-link in `development_plan_index.md` when signed
- Hand off to @coder-mcp with file paths + acceptance bullets only

```text
ΔWF→ DES-INDUSTRIAL-RESEARCH-001 + DES-FACILITY-SITE-ZONE-001 · then DES-APS-FACILITY-NEEDS-001
```
