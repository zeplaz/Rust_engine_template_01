# Coder master drain — Fire · Veg · Build `v1`

**Authority:** [`tools/orchestrator/queues/coder_master_drain_queue.json`](../tools/orchestrator/queues/coder_master_drain_queue.json)  
**Phase 4 drain:** closed — see `done_phase4` in JSON  
**Unwired spine:** [`build_read_unwired_spine_v1.md`](build_read_unwired_spine_v1.md)  
**Rule:** Pick **seq 1 → 24** · skip `blocked` / `deferred` · **REWIRE before BUILD verify**

---

## Single handoff (@coder)

```text
Drain coder_master_drain_queue.json top→bottom.
Wave 1 REWIRE: mod.rs spine (placement_debug, pointer_gate, map_zoom witness)
Wave 2 FIRE: G-PLAY-FIRE-001 → ecology refresh + vfx highlight
Wave 3 BUILD: P0 verify, pilot catalog refactor, grammar v0-003, visual-001, minimap
Wave 4 VEG: LG-2 succession → fire/build coupling → LG-3 districts → LG-4 population/preview
Regression: cargo test -p proc_A_dine01 --lib fire_ecology landscape_grammar construction stage5 sim_effects
```

---

## Track overview

| Track | Wave | Plain English | Owner |
|:---|:---:|:---|:---|
| **REWIRE** | 1 | Code on disk but not in binary — wire mod.rs first | B |
| **FIRE** | 2 | Normal play demo fire + refresh stale ecology JSON | A |
| **BUILD** | 3 | Pilot catalog authority, placement debug, post-commit visual | B |
| **VEG** | 4 | Landscape grammar LG-2+ — succession, districts, population | A |
| **PRODUCT** | 3 | Minimap widget (design PASS) | B |
| **DEFER** | 5–6 | Designer/MCP/DB — skip unless escalated | mixed |

**Parallel OK:** Wave 1 (B) + Wave 2 (A) · Wave 3 BUILD (B) + Wave 4 VEG (A) after rewire.

---

## Full drain table

| Seq | Track | ⟨ID⟩ | Owner | Status | Exit / witness |
|:---:|:---:|:---|:---|:---|:---|
| **1** | REWIRE | **BUILD-READ-REWIRE-003** | B | ⚡ready | `map_zoom_coherence_live.json` |
| **2** | REWIRE | **BUILD-READ-REWIRE-001** | B | ⚡ready | `placement_debug` in mod.rs |
| **3** | REWIRE | **BUILD-READ-REWIRE-002** | B | ⚡ready | `simulation_pointer_gate` in hud/mod.rs |
| 4 | REWIRE | MINIMAP-REWIRE-001 | B | 🟢 done | unblocks minimap impl |
| **5** | FIRE | **G-PLAY-FIRE-001** | A | ⚡ready | demo fire in normal play |
| **6** | FIRE | **VFX-FIRE-HIGHLIGHT-001** | A | ready | red box at strategic zoom |
| **7** | FIRE | **FIRE-ECOLOGY-REFRESH-001** | A | ready | `fire_ecology_live.json` F2 green |
| 8 | BUILD | BUILD-READ-P0-002 | B | ready | zoom witness after rewire-003 |
| 9 | BUILD | BUILD-READ-P0-003 | B | ready | pointer gate verify |
| 10 | BUILD | BUILD-READ-DEBUG-001 | B | ready | placement debug overlay |
| 11 | REWIRE | BUILD-READ-REWIRE-004 | B | 🟢 done | pilot hardcode lint |
| **12** | BUILD | **BUILD-READ-PILOT-001** | B | ⚡ready | ≥4 pilots, no warehouse Rust branches |
| 13 | BUILD | BUILD-READ-GRAMMAR-v0-003 | B | ready | DNA+β → massing pick |
| 14 | BUILD | BUILD-READ-VISUAL-001 | B | ready | lod0/tile visible in sim |
| 15 | PRODUCT | MINIMAP-WIDGET-IMPL-001 | B | ready | designer minimap spec |
| 16 | REWIRE | BUILD-READ-REWIRE-005 | B | ready | promote live proofs in dev/mod.rs |
| 17 | REWIRE | APS-QC-REWIRE-001 | B | ready | QC panel in gui/mod.rs |
| **18** | VEG | **LG-2-SUCCESSION-001** | A | ⚡ready | `landscape_grammar_lg2_live.json` |
| 19 | VEG | LG-2-FIRE-COUPLING-001 | A | ready | fire → DisturbanceHistory |
| 20 | VEG | LG-2-BUILD-CLEAR-001 | A | ready | construction → ⊖ disturbance |
| 21 | VEG | LG-3-DISTRICT-001 | A | ready | ag + industrial district presets |
| 22 | VEG | LG-4-POPULATION-001 | A | ready | graph-derived population |
| 23 | VEG | LG-4-PREVIEW-001 | A | ready | ≥3 topology kinds in preview |
| 24 | FIRE | SIM-STEWARD-FIRE-REGRESS-001 | steward | ready | replay + stage5 after slices |
| 25 | DEFER | BUILD-READ-PILOT-002 | designer-mcp | blocked | catalog rows only |
| 26 | DEFER | BUILD-READ-VISUAL-002 | coder-mcp | ready | tile bake promote |
| 27 | DEFER | BUILD-READ-DESIGN-001 | designer | ready | readability brief |
| 28 | DEFER | LG-5-ATLAS-001 | designer-mcp | 🧊 | after LG-4 |
| 29 | DEFER | FACTION-REACT-001 | A | 🧊 | after G-PLAY |
| 30 | DEFER | SIM-EFFECT-EMBED-DB-001 | — | 🧊 | GAME-STORE-GATE |

---

## Territory map (no toe-stepping)

| Owner | Paths |
|:---|:---|
| **coder_a** | `src/systems/ecology/` · `src/systems/fire/` · `src/scenario/` · fire play vis |
| **coder_b** | `src/construction/` · `src/gui/hud/` · `src/gui/map_zoom_coherence.rs` · minimap |

---

## Witness index

| Path | Slices |
|:---|:---|
| `debug_runs/map_zoom_coherence_live.json` | REWIRE-003, P0-002 |
| `debug_runs/play_scenario_live.json` | G-PLAY-FIRE-001 |
| `debug_runs/fire_ecology_live.json` | FIRE-ECOLOGY-REFRESH, LG-2 fire coupling |
| `debug_runs/vfx_fire_test_highlight_live.json` | VFX-FIRE-HIGHLIGHT |
| `debug_runs/pilot_catalog_parity_live.json` | BUILD-READ-PILOT-001 |
| `debug_runs/build_read_visual_001_live.json` | BUILD-READ-VISUAL-001 |
| `debug_runs/landscape_grammar_lg1_live.json` | LG-1 (done) |
| `debug_runs/landscape_grammar_lg2_live.json` | LG-2-SUCCESSION |
| `debug_runs/landscape_grammar_lg4_preview_live.json` | LG-4-PREVIEW |
| `debug_runs/construction_stage_live.json` | BUILD P0 regression |

---

## Phase 4 closed (do not re-pick)

SimEffect spine · build two-click UX · MAP-PICK · zoom code · fire ignition · scenario trigger · BUILD-READ shape/site/world · event log UI.

Fire **depth** (F2 extract, smoke bridge, fuel spread) — lib green; product gap is **play loop + ecology JSON refresh** (seq 5–7).

---

## Source queues merged

| Source | Merged into master |
|:---|:---|
| `coder_drain_queue.json` | Phase 4 → `done_phase4` |
| `post_drain_phase5_queue.json` | REWIRE, FIRE, BUILD tail, LG-2 |
| `build_read_unwired_spine_v1.md` | Wave 1 pick order |
| `plan_landscape_grammar_exec_001_v1.md` | LG-2→LG-4 rows |
| `plan_operator_build_readability_exec_001_v1.md` | BUILD-READ todo board |
| `grammar_continuation_queue.json` | CODER-PILOT-REFACTOR → seq 12 |
| `post_drain_active_queue.json` | Fire F2 → `done_fire_depth` |
