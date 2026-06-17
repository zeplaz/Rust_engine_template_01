# APS Option D — agent todo board (mockup → product)

| Field | Value |
|:---|:---|
| **Program** | APS-OPTION-D-001 |
| **Date** | 2026-06-16 |
| **Goal** | Match signed Option D IA + user mockup — not label-remap on building panels |
| **Machine queue** | [`tools/orchestrator/queues/aps_option_d_agent_queue.json`](../tools/orchestrator/queues/aps_option_d_agent_queue.json) |
| **Parallel wave** | [`parallel_wave_aps_veg_dispatch_v1.json`](../tools/orchestrator/queues/parallel_wave_aps_veg_dispatch_v1.json) |
| **IA sign** | [`design_aps_domain_ia_sign_v1.md`](design_aps_domain_ia_sign_v1.md) |
| **Capability plan** | [`plan_aps_evolution_veg_capability_20260616_v1.md`](plan_aps_evolution_veg_capability_20260616_v1.md) |
| **UX plan** | [`design_aps_uiux_style_quality_20260616_v1.md`](design_aps_uiux_style_quality_20260616_v1.md) |
| **Rule** | E1 is **FAIL** until tab-set swap + lane-scoped flow/pipeline match sign-off · WIT-HON before every Q✓ |

---

## Critical path (all agents)

```text
@designer sign chrome spec
    ↓
@coder-mcp E1-FIX tab swap + lane chrome (blocks everything landscape)
    ↓
@planner-mcp schema ∥ @designer-mcp content
    ↓
@coder-mcp E2 Presets + E3 States UI
    ↓
@coder_a LG4 pixel ∥ @designer-mcp matrix
    ↓
@coder-mcp E4 atlas ∥ @coder_b E5 resolver parity
```

---

## @orchestrator-mcp

| # | Pri | ID | Task | Exit |
|:--|:--|:---|:---|:---|
| 1 | P0 | ORCH-APS-001 | Sequence **aps_option_d_agent_queue.json** — E1-FIX before E2 content | **done** 2026-06-02 · `$ref:tools/orchestrator/queues/aps_option_d_dispatch_orders_v1.md` |
| 2 | P0 | ORCH-APS-002 | Block Q✓ on **APS-EVO-E1** until **APS-E1-TAB-SWAP-001** green | No false E1 close |
| 3 | P1 | ORCH-APS-003 | Run WIT-HON scan after each lane milestone | `validate-report witness_honesty` on APS witnesses |
| 4 | P0 | ORCH-APS-RECONCILE-001 | Sync E4/E5 across spine queues | **done** 2026-06-02 · `$ref:debug_runs/agent_ops/aps_option_d_queue_reconcile_live.json` |

**Copy-paste order:**
```text
Program APS-OPTION-D-001. Queue: tools/orchestrator/queues/aps_option_d_agent_queue.json.
Gate: APS-E1-TAB-SWAP-001 must green before any E2+ Q✓. WIT-HON on every witness.
```

---

## @coder-mcp (primary — APS UI)

| # | Pri | ID | Task | Exit |
|:--|:--|:---|:---|:---|
| 1 | **P0** | **APS-E1-TAB-SWAP-001** | **Fix E1:** landscape = 4 notebook pages (Presets·Grammar·States·Atlas); buildings = 5; **hide Materials in landscape**; no zip-label remap | `test_aps_lane_tab_swap.py` green |
| 2 | **P0** | **APS-E1-FLOW-LANE-001** | Lane-scoped flow bar: Landscape → Generate grammar · Bake states · Pack LG-5 atlas | Flow verbs change on lane switch |
| 3 | **P0** | **APS-E1-PIPELINE-LANE-001** | Lane-scoped pipeline: validity-aware pills per mockup; add **Stamp** step for landscape G5 | `test_aps_pipeline_landscape_validity.py` |
| 4 | **P0** | **APS-E1-CHROME-001** | Segmented lane control styling + lane chip pill (tokens `COLOR_LANE_*`) | Matches mockup structure |
| 5 | P0 | APS-EVO-E0-RELAUNCH-001 | Keep E2E + pytest `-k aps` green after E1 refactor | `aps_artist_tool_e2e_live.json` + WIT-HON |
| 6 | P1 | APS-EVO-E2-PRESET-BROWSE-001 | Presets tab: inline `validate-report landscape_grammar` plain PASS/FAIL | 10 presets listed + validate green |
| 7 | P1 | APS-E2-GRAMMAR-PANEL-001 | **New** `landscape_grammar_panel.py` — topology summary + preset editor (not Assembly reuse) | Grammar tab ≠ footprint grid |
| 8 | P1 | APS-EVO-E3-VEG-STATE-AXIS-001 | States tab: succession + burn axis UI after schema lands | **done** · `aps_veg_state_axis_live.json` |
| 9 | P1 | APS-EVO-E4-ATLAS-EXPAND-001 | LG-5 expanded atlas (16 keyframes + tile_batch) | **done** · `tile_landscape_expanded_live.json` |
| 10 | P1 | APS-EVO-E5-EXTRACT-PARITY-001 | Read-only extract parity panel | **done** · `aps_veg_extract_parity_live.json` |
| 11 | P1 | MCP-APS-EXTRACT-PARITY-STUB-001 | E5 stub (merged into E5 ship) | **done** · same witness as E5 |
| 12 | P1 | MCP-APS-ATLAS-LAND-REGISTER-001 | Atlas tab: `atlas_domain:landscape` → `_landscape_atlas_index` | **done** · `aps_atlas_land_register_live.json` |
| 13 | P2 | MCP-APS-CATALOG-LANDSCAPE-001 | catalog.py landscape preset list | **superseded** by E2 · `aps_landscape_preset_browse_live.json` |
| 14 | P2 | MCP-APS-VARIANTS-LAND-BRANCH-001 | variants_panel landscape branch | **superseded** by E3 States tab (Option D) |
| 15 | P2 | MCP-APS-ASSEMBLY-LAND-GRAMMAR-001 | assembly landscape grammar attach | **superseded** by `landscape_grammar_panel.py` (E2) |
| 16 | P2 | APS-E1-A11Y-001 | Ctrl+1/Ctrl+2 lane shortcuts per [`design_aps_domain_a11y_v1.md`](design_aps_domain_a11y_v1.md) | Keyboard guard green |

**Unblocked (2026-06-02):** E4/E5 + atlas register **closed** — maintain `APS-EVO-E0-RELAUNCH-001` after changes.

**Do not pick until E1-FIX done:** ~~E2 grammar content, E3 axis, E4 expansion~~ — **lifted** (E1 + deps green).

---

## @planner-mcp (schemas + authority — parallel to E1)

| # | Pri | ID | Task | Exit |
|:--|:--|:---|:---|:---|
| 1 | **P0** | **APS-EVO-E3-VEGCATALOG-SCHEMA-001** | `vegetation_variant_catalog_v1.schema.json` | **done** 2026-06-17 · witness `mcp_aps_e3_veg_catalog_schema_sign_live.json` |
| 2 | **P0** | **PLAN-VEG-RESOLVER-KEY-NAMING-001** | variant_key naming doc — @coder_b sign-off | **done** 2026-06-17 · `plan_veg_variant_key_naming_v1.md` |
| 3 | P0 | PLAN-APS-E5-ENGINE-AUTHORITY-001 | Map-stamp / RepresentationResult contract for E5 | Plan doc signed |
| 4 | P1 | PLAN-ATLAS-BUDGET-SIZING-001 | Topology×state matrix sizing framework for E4 | `plan_landscape_atlas_budget_v1.md` |
| 5 | P1 | PLAN-TILE-BATCH-LAND-EXAMPLE-001 | Expanded tile_batch JSON example with burn rows | Example spec on disk |
| 6 | P1 | PLAN-WITNESS-REOPEN-001 | Matrix: phase6/VEG done rows vs WIT-HON failures | Reopen list |
| 7 | P2 | PLAN-THREE-GREENS-VOCAB-001 | schema / bake / runtime / art-ship labels for all APS witnesses | Vocab doc |
| 8 | P2 | PLAN-INTEGRATION-CROSSREF-001 | Cross-ref 6 neighbor plans with APS evolution | Stale flags explicit |

---

## @designer-mcp (content + G4 sign — parallel)

| # | Pri | ID | Task | Exit |
|:--|:--|:---|:---|:---|
| 1 | P0 | DMCP-E0-ARTIST-REVERDICT-001 | **DONE** PASS WITH NOTES | `design_aps_artist_ship_review_20260616_v1.md` · `dmcp_e0_artist_reverdict_live.json` |
| 2 | **P0** | **DMCP-E4-MATRIX-CHARTER-001** | First expansion matrix: topology kinds × states for atlas v1 | Blocks E4 bake |
| 3 | P0 | DMCP-E3-VARIANT-KEY-SET-001 | Draft `_vegetation_variant_catalog.ron` content (after schema) | Validator green |
| 4 | P1 | DMCP-E2-PRESET-QC-CRITERIA-001 | What artist reads on preset browse (plain language) | QC criteria doc |
| 5 | P1 | DMCP-LG5-KEYFRAME-REQS-001 | Keyframe still reqs per burn/scar/recovery variant | Req doc |
| 6 | P1 | DMCP-TILE-BATCH-EXPAND-SPEC-001 | Signed expanded tile_batch content spec | JSON in staging/specs |
| 7 | P1 | DMCP-VEG-F01-ART-SHIP-001 | G4/G5 art-ship criteria for expanded LG-5 atlas | Charter update |
| 8 | P2 | DMCP-BURN-VISUAL-LANG-001 | Burn/scar/recovery readable at iso zoom | Visual language doc |
| 9 | P2 | DMCP-ATLAS-QC-PLAIN-001 | Plain-language atlas QC copy for landscape domain | Copy doc for coder-mcp |

---

## @designer — **design lane for E1/E2 chrome: CLOSED** ✅

| # | Pri | ID | Status | Deliverable |
|:--|:--|:---|:---|:---|
| 1 | P0 | DES-APS-CHROME-MOCKUP-001 | **PASS** | [`design_aps_chrome_mockup_spec_v1.md`](design_aps_chrome_mockup_spec_v1.md) |
| 2 | P1 | DES-APS-GRAMMAR-PANEL-UX-001 | **PASS** | [`design_aps_grammar_panel_v1.md`](design_aps_grammar_panel_v1.md) |
| 3 | P1 | DES-APS-PIPELINE-PILLS-001 | **PASS** | [`design_aps_pipeline_pills_v1.md`](design_aps_pipeline_pills_v1.md) |
| 4 | P0 | DES-APS-E1-IA-OPTION-D-001 | **PASS** (prior) | [`design_aps_domain_ia_sign_v1.md`](design_aps_domain_ia_sign_v1.md) |

**Next designer picks (E3/E5 prep):**

| # | Pri | ID | Task |
|:--|:--|:---|:---|
| 5 | P1 | DES-APS-PRESET-BROWSE-UX-001 | Presets list layout (supports E2 polish) |
| 6 | P1 | DES-APS-STATE-AXIS-LABELS-001 | Succession + burn labels (blocks E3 UI) |
| 7 | P1 | DES-APS-PARITY-PANEL-UX-001 | E5 extract parity panel mock |
| 8 | P2 | DES-APS-STYLE-TOKENS-001 | FONT_TITLE + PAD_* |

---

## @designer (original E1/E2 spec rows — archived above)

---

## @coder A (sim / LG-4 — parallel, gates E4)

| # | Pri | ID | Task | Exit |
|:--|:--|:---|:---|:---|
| 1 | **P0** | **CDR-A-LG4-PIXEL-REOPEN-001** | LG-4: pixel_heterogeneity + topology_tint_visible | `landscape_grammar_lg4_preview_live.json` green |
| 2 | P0 | CDR-A-WIT-HON-ROLLUP-001 | Fix veg_runtime_proof rollup child | `vegetation_program_close` WIT-HON pass |
| 3 | P1 | CDR-A-VEG-HARVEST-001 | harvest_disturbances >= 1 | lg2 witness |
| 4 | P1 | CDR-A-VEG-RECOVERY-001 | recovery_ticks >= 1 | lg2 witness |
| 5 | P1 | CDR-A-NESTED-DEPTH-003 | nested_depth_max >= 3 | lg2 witness |
| 6 | P1 | CDR-A-EXTRACT-SPRITE-001 | Real-sprite variant_key in extract (not tint-only) | extract witness |
| 7 | P2 | CDR-A-ECOLOGY-HARNESS-CLEAN-001 | Remove harness-only stage5 ecology injection when live path green | stage5 witness |

**Not APS UI** — do not implement Grammar/States panels here.

---

## @coder B (resolver / consumer — parallel, gates E5)

| # | Pri | ID | Task | Exit |
|:--|:--|:---|:---|:---|
| 1 | P0 | CDR-B-VEG-RESOLVER-PARITY-001 | Document known variant_keys for VegetationExtractFrame | `veg_resolver_known_keys_v1.md` |
| 2 | P1 | CDR-B-BUILD-CONSUMER-MCP-001 | APS DNA+β consumer in Rust (phase6 ready row) | `aps_dna_consumer_contract_live.json` |
| 3 | P1 | CDR-B-TILE-RESOLVER-VEG-001 | TileVariantResolver landscape domain branch | Resolver tests green |
| 4 | P1 | CDR-B-MAP-STAMP-CONTRACT-001 | Map stamp: landscape atlas UV → chunk render | Contract witness |
| 5 | P2 | CDR-B-REPRESENTATION-PARITY-001 | Review with @planner-mcp E5 engine authority plan | Sign-off note |

---

## @planner (engine — light touch)

| # | Pri | ID | Task | Exit |
|:--|:--|:---|:---|:---|
| 1 | P1 | PLAN-VEG-RUNTIME-PROOF-001 | Finalize lib vs operator vs art-ship proof tiers | Plan doc |
| 2 | P2 | PLAN-G-PLAY-SPLIT-001 | G-PLAY lib vs operator split | Plan doc |

---

## Definition of done — “matches mockup”

- [ ] Landscape lane shows **4 tabs only** (Presets · Grammar · States · Atlas)
- [ ] Buildings lane **byte-identical** to pre-lane behavior (5 tabs)
- [ ] Lane switch clears cross-lane selection; no building footprint visible in landscape Grammar
- [ ] Flow bar verbs change per lane
- [ ] Pipeline bar shows validity-aware pills + landscape **Stamp** step
- [ ] Presets: inline landscape_grammar validate (plain language)
- [ ] Grammar: topology-graph workspace (dedicated panel)
- [ ] States: succession + burn axis (after catalog schema)
- [ ] Atlas: landscape register + G0–G5 scope-explicit QC (not one green)
- [ ] E2E + WIT-HON green on APS witnesses
- [ ] LG-4 pixel proof green before E4 art-ship expansion

```text
[/APS-OPTION-D-001] dispatch → aps_option_d_agent_queue.json · mockup = target · E1-FIX = gate
```
