# APS-OPTION-D-001 — dispatch orders `v1`

```text
⟨APS-OPTION-D-001⟩  🟡⏳⊗☊  issued=2026-06-02
Board: $ref:src/dev/aps_option_d_agent_todos_v1.md
Queue: $ref:tools/orchestrator/queues/aps_option_d_agent_queue.json
Gate: **APS-E1-TAB-SWAP-001** blocks ALL E2+ Q✓ · BLANG:WIT-HON before every Q✓
Witness: $ref:debug_runs/agent_ops/aps_option_d_dispatch_live.json
```

---

## Orchestrator-mcp ruling

| Rule | Enforcement |
|:---|:---|
| **E1 gate** | No `BLANG:Q✓` on seq ≥ 12 (E2+) until **APS-E1-TAB-SWAP-001** exit green |
| **E1 chain** | TAB-SWAP → FLOW-LANE → PIPELINE-LANE → CHROME (seq 2→5) before E2 content |
| **WIT-HON** | `validate-report witness_honesty <path> --compress 3` on every row with `witness` |
| **Parallel OK** | planner-mcp schema · planner resolver naming · coder_a LG4/WIT-HON · designer-mcp matrix |
| **Forbidden** | `label_remap_only` · `materials_visible_in_landscape` · false E1 close |

**Current gate status:** **APS-E1-TAB-SWAP-001 done** · `test_aps_lane_tab_swap.py` **5/5 green** · witness `debug_runs/aps_option_d_e1_live.json` **green** · E2+ unblocked.

---

## Drain order (seq)

```text
DONE   DES-APS-CHROME-MOCKUP-001 · DES-APS-GRAMMAR-PANEL-UX-001
DONE   APS-E1-TAB-SWAP-001 · APS-E1-FLOW-LANE-001 · APS-E1-PIPELINE-LANE-001 · APS-E1-CHROME-001
DONE   APS-EVO-E3-VEGCATALOG-SCHEMA-001 · PLAN-VEG-RESOLVER-KEY-NAMING-001  (@planner-mcp)
NOW    maintain APS-EVO-E0-RELAUNCH-001  (@coder-mcp)
DONE   MCP-APS-ATLAS-LAND-REGISTER-001 · APS-EVO-E4 · APS-EVO-E5 · MCP-APS-EXTRACT-PARITY-STUB-001
SUPER  MCP-APS-CATALOG-LANDSCAPE-001 · MCP-APS-VARIANTS-LAND-BRANCH-001 · MCP-APS-ASSEMBLY-LAND-GRAMMAR-001
       ∥ CDR-A-LG4-PIXEL-REOPEN-001  (@coder A)
       ∥ CDR-A-WIT-HON-ROLLUP-001  (@coder A)
       ∥ DMCP-E4-MATRIX-CHARTER-001  (@designer-mcp)
MAINT  APS-EVO-E0-RELAUNCH-001  (re-witness after each E1 slice)
```

---

## Lane orders (copy to each agent)

### @coder-mcp — **E1 spine CLOSED · E5 next**

```text
DONE:    ⟨APS-E1-TAB-SWAP-001⟩  pytest tools/mcp/python/tests/test_aps_lane_tab_swap.py 5/5
DONE:    ⟨APS-E1-FLOW-LANE-001⟩ · ⟨APS-E1-PIPELINE-LANE-001⟩ · ⟨APS-E1-CHROME-001⟩
WIT-HON: debug_runs/aps_option_d_e1_live.json pass
PRIMARY: maintain ⟨APS-EVO-E0-RELAUNCH-001⟩  pytest -k aps + witnesses
DONE:    ⟨MCP-APS-ATLAS-LAND-REGISTER-001⟩  `aps_atlas_land_register_live.json`
RECON:   ⟨MCP-APS-CATALOG-LANDSCAPE-001⟩ → E2 · ⟨MCP-APS-VARIANTS-LAND-BRANCH-001⟩ → E3 States · ⟨MCP-APS-ASSEMBLY-LAND-GRAMMAR-001⟩ → grammar panel
MAINT:   APS-EVO-E0-RELAUNCH-001 (pytest -k aps + aps_artist_tool_e2e_live.json)
```

### @planner-mcp — **CLOSED 2026-06-17 (no E1 dep)**

```text
DONE: ⟨APS-EVO-E3-VEGCATALOG-SCHEMA-001⟩ · ⟨PLAN-VEG-RESOLVER-KEY-NAMING-001⟩
WIT:  debug_runs/mcp_aps_e3_veg_catalog_schema_sign_live.json
SHIP: assets/configs/landscape/_vegetation_variant_catalog.ron (29 keys · 8 burn)
ΔWF:  DMCP-E3-VARIANT-KEY-SET-001 ready · CDR-B-VEG-RESOLVER-PARITY-001 ready
```

### @designer-mcp — **wave 0 CLOSED**

```text
DONE: DMCP-E0-ARTIST-REVERDICT-001 · DMCP-E3/E4/E2/LG5/VEG-F01 (parallel wave)
WITNESS: debug_runs/art_pipeline/dmcp_e0_artist_reverdict_live.json
DELIVER: src/dev/design_aps_artist_ship_review_20260616_v1.md (PASS WITH NOTES 7/10)
PICK: idle — @coder-mcp APS-EVO-E3-VEG-STATE-AXIS-001 / E4 atlas expand
```

### @designer — **maintain only**

```text
MAINTAIN: DES-APS-E1-IA-OPTION-D-001 sign + FAIL criteria for label-remap E1
NO NEW PICKS until coder-mcp E1-CHROME lands
```

### @coder A — **parallel sim (gates E4, not APS UI)**

```text
PRIMARY: ⟨CDR-A-LG4-PIXEL-REOPEN-001⟩  in_progress
PARALLEL: ⟨CDR-A-WIT-HON-ROLLUP-001⟩
WIT-HON: validate-report witness_honesty debug_runs/landscape_grammar_lg4_preview_live.json
         validate-report witness_honesty debug_runs/vegetation_program_close_live.json
FORBID:  implement Grammar/States Tk panels (coder-mcp owns APS UI)
```

### @coder B — **parallel resolver doc**

```text
BLOCKED: ⟨CDR-B-VEG-RESOLVER-PARITY-001⟩ until PLAN-VEG-RESOLVER-KEY-NAMING-001 Q✓
PARALLEL OK: CDR-B-BUILD-CONSUMER-MCP-001 (phase6 row)
```

### @orchestrator-mcp — **track**

```text
TRACK:   E1 gate · WIT-HON on APS witnesses after each milestone
SCAN:    validate-report witness_honesty --scan debug_runs --compress 3  (report-only OK)
BLOCK:   any E2+ Q✓ attempt while APS-E1-TAB-SWAP-001 ≠ done
WITNESS: debug_runs/agent_ops/aps_option_d_dispatch_live.json
```

---

## E2+ block list (lifted 2026-06-02)

**APS-E1-TAB-SWAP-001** = `done` + WIT-HON pass — rows below are **unblocked**:

| ID | Owner |
|:---|:---|
| APS-EVO-E2-PRESET-BROWSE-001 | coder-mcp |
| APS-E2-GRAMMAR-PANEL-001 | coder-mcp |
| APS-EVO-E3-VEG-STATE-AXIS-001 | coder-mcp |
| APS-EVO-E4-ATLAS-EXPAND-001 | coder-mcp |
| APS-EVO-E5-EXTRACT-PARITY-001 | coder-mcp |
| DMCP-E0-ARTIST-REVERDICT-001 | designer-mcp (partial — needs E1-CHROME too) |

---

## WIT-HON baseline (APS lane)

| Witness | Status | Action |
|:---|:---:|:---|
| `aps_domain_router_live.json` | 🟢 | maintain after E1 slices |
| `aps_artist_tool_e2e_live.json` | ⚠ envelope | re-witness after E0 maintain |
| `aps_landscape_preset_browse_live.json` | 🟢 | E2 Q✓ allowed (TAB-SWAP done) |
| `aps_option_d_e1_live.json` | 🟢 | E1 critical path rollup |
| `landscape_grammar_lg4_preview_live.json` | 🔴 | CDR-A-LG4-PIXEL-REOPEN-001 |
| `vegetation_program_close_live.json` | 🔴 | CDR-A-WIT-HON-ROLLUP-001 |

```text
[/APS-OPTION-D-001] E1-FIX = done · TAB-SWAP green · E5 extract parity = next coder-mcp pick
```
