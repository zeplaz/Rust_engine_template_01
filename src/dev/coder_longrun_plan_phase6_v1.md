# Coder long-run plan — Phase 6 `v1` (2026-06-14)

```text
⟦SYMLANG⟧⟐v1  ◈EXEC
⟨ID⟩ CODER-LONGRUN-PHASE6-001
Authority: witness JSON · coder_queue_hardening_rules_v1.md
Machine queue: tools/orchestrator/queues/post_drain_phase6_coder_queue.json
Prior: POST-DRAIN-PHASE-5-001 (drained) · VEGETATION-PROGRAM-001 v3 (82/82 coder rows)
Rule: lib green ≠ done · no empty stub modules · extend existing authorities
```

**Headline:** Phase 5 drained the **spine** (rewire, fire play, build-read lib, vegetation program A–G). Phase 6 is **months of product hardening + grammar depth + infra** — parallel dual-coder tracks with witness gates, not new architecture stubs.

---

## 1. Recent work review (honest)

### Closed (witness-backed — do not re-queue)

| Program | What shipped | Proof |
|:---|:---|:---|
| **Phase 4 drain** | SimEffect spine · MAP-PICK · zoom code · fire harness · build shape/site | `coder_drain_queue.json` seq 1–14 |
| **Phase 5 REWIRE** | placement_debug · pointer_gate · map_zoom · minimap · pilot_catalog · APS QC | `build_read_unwired_spine_v1.md` rows done |
| **Phase 5 FIRE** | G-PLAY demo fire · VFX highlight · ecology refresh · product verify | `coder_product_verify_queue.json` |
| **Phase 5 BUILD** | P0 verify · grammar v0-003 · visual-001 lib · pilot-001 | `grammar_diversity_witness.json` |
| **Landscape LG-0** | Charter · lexicon v1.4 · schema · 30 presets · composite grammar §1.17 | `landscape_grammar_v0.schema.json` |
| **Vegetation v3 drain** | 82 rows — harness → map rollout → preview → districts → population → snapshot | `vegetation_program_close_live.json` |

### Lib-green but still thin (Phase 6 hardening targets)

| Gap | Symptom | Phase 6 fix |
|:---|:---|:---|
| **FULL_APP ecology extract** | `stage5_full_app_live.json` ecology rows harness-fed | Wire `publish_climate_visual_aggregate` from live `LandscapeProgramOnChunk` |
| **LG-5 art** | Atlas blocked on designer-mcp / coder-mcp | Coder: registry stamp + consumer path only — **no bpy in src/** |
| **LG-3 districts** | Witness green; coupling depth varies | Replace remaining coord heuristics with transport/hydro/construction reads |
| **Build visual runtime** | `BUILD-VERIFY-VISUAL-001` done — operator pixel sign-off open | `--test visual` post-commit mesh/tile proof |
| **IND-E02 default JSON** | Commit path green; default writer `ind_e02_green: false` | Default industrial play witness writer |
| **G-PLAY-01** | Operator checklist — not coder | Coder unblocks with product verify keys only |

### Explicitly rejected (never queue)

- Empty `.rs` stub modules “for later”
- Biome → density → sprite spine
- `VegetationPopulation`-only without topology graph
- Re-declaring Phase 5 rows done without witness `green: true`

---

## 2. North star (6 months)

```text
PRODUCT     G-PLAY closed · build readable in sim · fire+veg visible at operational zoom
GRAMMAR     Building v0 → v1 depth · Landscape LG-1..4 hardened · LG-5 consumer ready
INFRA       World layers E0–E6 · transport R8 construction slice
GROWTH      Organic infill + procedural PG tails (after infra column)
SIM         Stage 7 M1 behavioral · replay ring · faction stress hooks
```

**Regression spine (every slice):**

```powershell
cargo test -p proc_A_dine01 --lib landscape_grammar fire_ecology construction stage5 sim_effects
python -m rust_engine_mcp.cli validate-report cargo --compress 3
```

---

## 3. Dual-coder territory (no toe-stepping)

| Owner | Paths | Long-run focus |
|:---|:---|:---|
| **Coder A** | `src/systems/ecology/` · `src/systems/fire/` · `src/scenario/` · `src/sim/effects/` · `src/substrate/hydrology/` | Veg hardening · fire↔population · scenario/product witnesses |
| **Coder B** | `src/construction/` · `src/gui/hud/` · `src/gui/` · `src/infrastructure/` · `src/strategic/` | Build grammar · infra column · minimap/HUD · growth books |

**Parallel OK:** A on VEG-HARD-* while B on BUILD-GRAMMAR-* or INFRA-*.

---

## 4. Phase 6 waves (pick top→bottom per track)

### Wave 1 — Truth hardening (2–3 weeks) ⚡ start here

Close the gap between **lib witnesses** and **running sim / FULL_APP**.

| Seq | ID | Owner | Deliverable | Witness |
|:---:|:---|:---|:---|:---|
| 1 | **VEG-HARD-FULLAPP-001** | A | Ecology visual snapshot from live program query in FULL_APP refresh | `stage5_full_app_live.json` `ecology_rows_source: live` |
| 2 | **VEG-HARD-PREVIEW-PIXEL-001** | A | Preview tint heterogeneity in visual harness (≥3 topology kinds pixel-diff) | `landscape_grammar_lg4_preview_live.json` |
| 3 | **BUILD-HARD-VISUAL-RUN-001** | B | Post-commit extract visible in `--test visual` (not lib-only) | `build_read_visual_001_live.json` `runtime_sim_verified` |
| 4 | **IND-E02-DEFAULT-JSON-001** | B | Default play writer sets `ind_e02_green: true` when seed fires | `play_scenario_live.json` |
| 5 | **BQ128-APPLY-001** | B | Apply BQ-128 editor path plan slice | per `bq128_editor_path_plan_v1.md` |
| 6 | **SIM-STEWARD-COMBINED-001** | steward | `stage5` + `fire_ecology` + `landscape_grammar` combined regression | lib green |

### Wave 2 — Landscape grammar depth (3–4 weeks)

**While LG-5 atlas blocked** — deepen evaluator + composite topology (lexicon §1.17).

| Seq | ID | Owner | Deliverable |
|:---:|:---|:---|:---|
| 7 | **VEG-COMPOSITE-EVAL-001** | A | `MACRO-*` registry — composite recipes resolve to topology subgraph |
| 8 | **VEG-PRESET-INDUSTRIAL-002** | A | Industrial/military presets wired to settlement + transport anchors |
| 9 | **VEG-λ-LIVE-001** | A | λ pressure from live hydrology + weather fields (no coord hack) |
| 10 | **VEG-MINIMAP-OVERLAY-002** | B | Topology tint legend on minimap (planning glyphs §1 debug mode) |
| 11 | **VEG-FIRE-CORRIDOR-FULLAPP-001** | A | Fire corridor reads population fuel in FULL_APP extract |
| 12 | **VEG-SNAPSHOT-PLAY-001** | A | Save/load vegetation snapshot in play scenario roundtrip |
| 13 | **VEG-DIAG-COMPOSITE-001** | A | Diagnostics panel: nested topology tree + disturbance timeline |

### Wave 3 — Building grammar v1 (4–6 weeks)

Mirror landscape depth for construction readability.

| Seq | ID | Owner | Deliverable |
|:---:|:---|:---|:---|
| 14 | **BUILD-GRAMMAR-PROGRAM-001** | B | `ProgramGraph` stub — site zones from ARCH-DNA + β (read-only v1) |
| 15 | **BUILD-GRAMMAR-PILOT-EXPAND-001** | B | ≥8 pilots in `_pilot_catalog.ron` — rect/T/O/L variants |
| 16 | **BUILD-GRAMMAR-SITE-ZONE-001** | B | FootprintMatrix obeys site 15–40% occupancy rule |
| 17 | **BUILD-GRAMMAR-β-WORLD-001** | B | β pressure reads transport + land-use influence |
| 18 | **BUILD-GRAMMAR-WITNESS-002** | B | `grammar_diversity_witness.json` — ≥3 massing picks per DNA family |
| 19 | **BUILD-READ-CONSUMER-MCP-001** | B | When MCP unblocks: wire APS snapshot DNA+β (consumer only) |

**Blocked on designer-mcp:** BUILD-READ-VISUAL-002 tile bake · BUILD-READ-GRAMMAR-v0-002 APS UI.

### Wave 4 — Infrastructure column (6–8 weeks, parallel)

From [`fleet_longrun_prompts_20260602_v1.md`](fleet_longrun_prompts_20260602_v1.md) — still valid horizon.

| Seq | ID | Owner | Deliverable |
|:---:|:---|:---|:---|
| 20 | **INFRA-E0-003** → **E6-004** | A/B split | World layers exec plan column |
| 21 | **INFRA-TRANSPORT-R8-001** | B | Construction corridor slice in transport snapshot |
| 22 | **INFRA-UTILITY-OVERLAY-001** | B | Utility graph overlay in sim HUD (design PASS) |
| 23 | **INFRA-VM-DEEP-001** | A | VM-09/10/11 audit fixes from triage backlog |

### Wave 5 — Construction + growth product (6+ weeks)

| Seq | ID | Owner | Deliverable |
|:---:|:---|:---|:---|
| 24 | **PROC-PG-2-TAIL-001** | A | Module mesh authority — lod0/production only |
| 25 | **PROC-OG-4-001** | A | Town rollup books |
| 26 | **PROC-OG-UX-WIRE-001** | B | Growth approve HUD wired |
| 27 | **CON-R4-TAIL-001** | B | Construction Round 4 product board slices |
| 28 | **PT-5-002** | A | Fire frame tick in tile resolver |

### Wave 6 — Sim depth (after Wave 1 green)

| Seq | ID | Owner | Deliverable |
|:---:|:---|:---|:---|
| 29 | **S7B-M1-001** | A | Stage 7 behavioral M1 per exec plan |
| 30 | **REPLAY-RING-001** | B | Replay ring exec tail |
| 31 | **FACTION-REACT-002** | A | SimEffect telemetry → faction stress (extend) |
| 32 | **SIM-EFFECT-PLUGIN-001** | B | Register sim effects plugin in FULL_APP (telemetry on disk unwired) |

---

## 5. Blocked lanes — skip unless escalated

| ID | Blocker | Coder action |
|:---|:---|:---|
| VEG-F01/F02 | designer-mcp / coder-mcp atlas | Consumer registry only |
| VEG-C14 | operator human sign-off | Wait |
| BUILD-READ-PILOT-002 | designer-mcp catalog rows | Wait |
| BUILD-READ-DESIGN-001/002 | designer sign-off | Wait |
| LG-6 flowers | deferred | Wait |
| SIM-EFFECT-EMBED-DB-001 | GAME-STORE-GATE | Deferred |

---

## 6. Session ritual (every pick)

```text
1. Read row in post_drain_phase6_coder_queue.json
2. Read exit_predicate witness on disk — all must pass before done
3. Implement ≤3 files per PR · extend existing module
4. cargo test filters from row
5. Refresh witness JSON (live_sim_required rows: FULL_APP or play harness)
6. Mark row done · BLANG:Q✓ · regression spine
```

**Handoff one-liner for @coder:**

```text
Drain post_drain_phase6_coder_queue.json seq 1→32 top-down.
Wave 1 VEG-HARD + BUILD-HARD first. Parallel: A=ecology/fire B=construction/gui/infra.
Read vegetation_system_honest_status_v1.md before claiming veg done.
Lexicon composite rules: landscape_grammar_lexicon_v1.md §1.17 when adding topology.
```

---

## 7. Planner / MCP gates that unblock later waves

| Gate | Unblocks |
|:---|:---|
| MCP kit002 unfreeze plan | PT production tiles · VEG-F02 atlas |
| ARCH-002 variant graph schema | Tile variant matrix |
| BUILD-READ-DESIGN-001 sign | HUD copy + readability acceptance |
| PLAN-AUDIT-020 | Post G-PLAY operator close |

---

## Changelog

| Ver | Date | Note |
|:---|:---|:---|
| v1.0.0 | 2026-06-14 | Phase 6 long-run plan after Phase 5 + veg v3 drain review |

```text
⟦/CODER-LONGRUN-PHASE6-001⟧  ΔWF→@coder drain post_drain_phase6_coder_queue.json
```
