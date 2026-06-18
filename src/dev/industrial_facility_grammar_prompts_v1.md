# Industrial facility grammar — agent prompts `v1`

Plan: [`plan_industrial_facility_grammar_suite_v1.md`](plan_industrial_facility_grammar_suite_v1.md)  
Parent backlog: [`plan_designer_work_202606_v1.md`](plan_designer_work_202606_v1.md)  
Iterate spine: [`design_grammar_iterate_tooling_v1.md`](design_grammar_iterate_tooling_v1.md)

---

## @designer — research + IA (no Python)

### P0 — Research foundation

```
1. DES-INDUSTRIAL-RESEARCH-001
   design_industrial_process_research_v1.md
   Sections: concrete_portland + aluminum_primary + utility yards
   For each step: role, typical footprint, power band, adjacency, visual cues
   Authority: assets/configs/industrial_supply_chains.json (cite catalog_ids)

2. DES-FACILITY-SITE-ZONE-001
   design_facility_site_zone_taxonomy_v1.md
   Zone ids + required/optional matrix per archetype (warehouse, factory, smelter, substation)
   Ref: assets/configs/buildings/pilots/*_site_v0.json

3. DES-POWER-TIER-001
   design_power_tier_bands_v1.md
   Map designer units → tier → APS glyph + module density hints
   Ref: economy/supply_chain.rs electrical_from_power_units
```

### P1 — APS surfaces (after P0 signed)

```
4. DES-APS-FACILITY-NEEDS-001  — strip spec (chain step, power, I/O)
5. DES-APS-SITE-PREVIEW-001    — footprint + site grid overlay
6. DES-APS-CHAIN-BROWSER-001   — read-only chain picker → archetype pre-fill
```

Sign-off **DES-FACILITY-BINDING-001** schema doc after reviewing designer-mcp draft.

---

## @designer-mcp — content (RON + pilots, no Tk)

```
1. Draft design_facility_binding_schema_v1.md (optional grammar block — catalog authority)
2. DMCP-GRAM-FACTORY-CLUSTER-001 — factory_cluster_v1.ron + facility_binding
3. DMCP-PILOT-CONCRETE-SITE-001 — 3-step site JSON from research templates
4. Run: grammar-facility-brief (when CMCP ships) + designer_grammar_quality_loop --full
```

---

## @coder-mcp — suite tools (Python/APS only)

**Pick after DES-FACILITY-BINDING-001 + DES-APS-FACILITY-NEEDS-001 signed.**

```
1. CMCP-GRAMMAR-FACILITY-BRIEF-001
   CLI + MCP: join grammar_id → catalog JSON → chain row → witness JSON
   No invented power — load power_consumption from catalog only

2. CMCP-SITE-ZONE-VALIDATE-001
   validate-report site_zone_grid — zone %, orphan cells, taxonomy refs

3. CMCP-FACILITY-NEEDS-PANEL-001
   APS strip per DES-APS-FACILITY-NEEDS-001 — status_atom, design system tokens

4. CMCP-GRAM-SWEEP-PROCESS-001
   Extend grammar_eval_sweep — power_tier + role histogram in witness
```

**Verify:** pytest + `designer_grammar_quality_loop --full --write-witness` still green.

---

## @coder — engine read (after CMCP brief green)

```
1. COD-FACILITY-BINDING-READ-001 — optional facility_binding on grammar types (read-only)
2. COD-BUILD-READ-PROCESS-001 — HUD human labels for chain step + power tier
   Ref: plan_designer_work Track D DES-BUILD-READ-HUD-002

Do NOT pick COD-SITE-MULTI-PLACE-001 until site preview operator-signed.
```

---

## Rules (all agents)

- **Layer 3 authority:** catalog + `industrial_supply_chains.json` — grammar never overrides power/IO
- **No mega-factories:** one catalog_id per grammar binding; chains stay multi-building
- **Validation-first:** reason on `ValidationReport` / witness JSON — not raw CLI walls
- **Tier gating:** site preview collapsed at G0; promote per `design_aps_grammar_tier_exposure_v1.md`

```text
ΔWF→ DES-INDUSTRIAL-RESEARCH-001 → DES-FACILITY-BINDING-001 → CMCP-GRAMMAR-FACILITY-BRIEF-001
```
