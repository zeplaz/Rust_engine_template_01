# PARALLEL-WAVE-APS-VEG — Wave 0 dispatch orders `v1`

```text
⟨PARALLEL-WAVE-APS-VEG-DISPATCH-001⟩  🟡⏳⊗☊  wave=0  issued=2026-06-16
AUTH: MAT★ ⇢ APS★ ⇢ SNAP★ ⇢ WRK○ ⇢ ATL○ ⇢ RT○
Spine: $ref:tools/orchestrator/queues/mcp_aps_evolution_queue.json  E0→E5 after wave-0
Gate: BLANG:WIT-HON before every BLANG:Q✓
Baseline: $ref:debug_runs/agent_ops/parallel_wave_w0_wit_hon_baseline.json
```

---

## Orchestrator-mcp action (this document)

Wave **0** = **six owner lanes start NOW** · no cross-lane deps · wave **1** gated on E0 relaunch + IA sign.

---

## Lane orders (copy to each agent)

### @coder-mcp — **P0 spine E0**

```text
PRIMARY: ⟨APS-EVO-E0-RELAUNCH-001⟩  in_progress
PARALLEL: MCP-APS-IMPORT-GUARD-001 · MCP-APS-WIT-WRITER-001 · MCP-APS-STATE-SCAFFOLD-001
         MCP-APS-WIT-HON-HOOK-001 · MCP-LANDSCAPE-BROWSE-STUB-001
WIT-HON: validate-report witness_honesty debug_runs/aps_artist_tool_e2e_live.json
         → fix WIT-MISSING-ENVELOPE (MCP-APS-WIT-WRITER-001)
VERIFY:  pytest tools/mcp/python/tests -k aps
Q✓:      only after WIT-HON pass + import_guard_pass + green:true
Spine:   unblocks APS-EVO-E1-DOMAIN-ROUTER-001 (wave 1)
```

### @planner-mcp — **P0 schema (no E0 dep)**

```text
PRIMARY: ⟨APS-EVO-E3-VEGCATALOG-SCHEMA-001⟩  in_progress
PARALLEL: PLAN-VEG-RESOLVER-KEY-NAMING-001 · PLAN-VEG-RUNTIME-PROOF-001 · PLAN-WITNESS-REOPEN-001
         PLAN-VEG-BURN-RECONCILE-001 · PLAN-G-PLAY-SPLIT-001
DELIVER: tools/mcp/schemas/vegetation_variant_catalog_v1.schema.json
Q✓:      schema validates · @coder sign-off on variant_key naming doc
Spine:   unblocks DMCP-E3-VARIANT-KEY-SET-001 · APS-EVO-E3-VEG-STATE-AXIS-001 (wave 2)
```

### @designer-mcp — **wave 0 CLOSED**

```text
DONE: ⟨DMCP-E0-ARTIST-REVERDICT-001⟩ PASS WITH NOTES · witness dmcp_e0_artist_reverdict_live.json
DONE: parallel wave rows (E3/E4/E2/LG5/VEG-F01) — designer_mcp_parallel_wave0_live.json green
PICK: idle
```

### @designer — **P0 IA sign (blocks E1)**

```text
PRIMARY: ⟨DES-APS-E1-IA-OPTION-D-001⟩  in_progress
PARALLEL: DES-APS-PRESET-BROWSE-UX-001 · DES-APS-STATE-AXIS-LABELS-001 · DES-APS-STYLE-TOKENS-001
DELIVER: src/dev/design_aps_domain_ia_sign_v1.md  (Option D: domain on 5 tabs)
Q✓:      blocks APS-EVO-E1-DOMAIN-ROUTER-001 (wave 1)
```

### @coder A — **P0 LG4 pixel + WIT rollup**

```text
PRIMARY: ⟨CDR-A-LG4-PIXEL-REOPEN-001⟩  in_progress
PARALLEL: CDR-A-WIT-HON-ROLLUP-001 · CDR-A-VEG-HARVEST-001 · CDR-A-STAGE5-LIVE-ECO-001
WIT-HON: lg4 preview PASS · vegetation_program_close FAIL → CDR-A-WIT-HON-ROLLUP-001
EXIT:    pixel_heterogeneity_wired · topology_tint_visible_chunks>=1 · green:true
FORBID:  eval_math_without_render · witness_lib_green_without_green
REGRESS: cargo test -p proc_A_dine01 --lib landscape_grammar fire_ecology stage5
```

### @coder B — **P0 build consumer + resolver authority**

```text
PRIMARY: ⟨CDR-B-BUILD-CONSUMER-MCP-001⟩  in_progress
PARALLEL: CDR-B-VEG-RESOLVER-PARITY-001 · CDR-B-MAP-STAMP-CONTRACT-001 · CDR-B-WIT-HON-PHASE6-001
DELIVER: src/dev/veg_resolver_known_keys_v1.md (blocks E5 parity)
WITNESS: debug_runs/aps_dna_consumer_contract_live.json
REGRESS: cargo test -p proc_A_dine01 --lib construction
```

---

## Wave 1 gate (do not start until)

| Row | Requires |
|:---|:---|
| APS-EVO-E1-DOMAIN-ROUTER-001 | E0 Q✓ + DES-APS-E1-IA-OPTION-D-001 Q✓ |
| DMCP-E3-VARIANT-KEY-SET-001 | APS-EVO-E3-VEGCATALOG-SCHEMA-001 Q✓ |
| APS-EVO-E2-* | E1 domain router Q✓ |

---

## WIT-HON baseline (pre wave-0)

| Witness | Status | Owner action |
|:---|:---|:---|
| aps_artist_tool_e2e_live.json | ⚠ WIT-MISSING-ENVELOPE | MCP-APS-WIT-WRITER-001 |
| landscape_grammar_lg4_preview_live.json | 🟢 pass | CDR-A-LG4-PIXEL-REOPEN-001 exit fields |
| vegetation_program_close_live.json | 🔴 rollup child fail | CDR-A-WIT-HON-ROLLUP-001 |

---

## Anti-patterns

```text
Q✓ without WIT-HON  ✗
lib green = art-ship green  ✗
preempt wave-1 before E0 + IA  ✗
MCP bpy work during wave-0 coder lane  ✗ (unless owner=coder-mcp)
```

---

**ΔWF:** all six lanes parallel · orchestrator tracks rollup · ⟨BP:SHARE⟩ on lane close
