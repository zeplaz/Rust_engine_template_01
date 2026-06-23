# Coder crisis filter — todo board `v1`

| Field | Value |
|:---|:---|
| **Problem** | INTEL-OFFICER bulk cull **reopened** ~9 coder picks that were already shipped (witness-green) |
| **Fix ritual** | test → reconcile → sync → scan |
| **Date** | 2026-06-21 |

## Ritual (run every session start)

```powershell
# 1) Refresh lib witnesses (no GPU)
cargo test -p proc_A_dine01 --lib product_verify_live
cargo test -p proc_A_dine01 --lib map_zoom_coherence
cargo test -p proc_A_dine01 --lib sim_effect_spine
cargo test -p proc_A_dine01 --lib landscape_grammar
cargo test -p proc_A_dine01 --lib aps_bevy_qc
cargo test -p proc_A_dine01 --lib fire_ecology

# 2) Close witness-green reopened rows
python tools/orchestrator/scripts/reconcile_coder_crisis.py
python tools/orchestrator/scripts/mark_pick_done_from_witness.py

# 3) Resync agent hub
python tools/orchestrator/scripts/sync_dispatch_subqueues.py
python tools/orchestrator/scripts/scan_queues_hub.py
```

## Filter tiers

### Tier A — **RECONCILE** (witness green, mark `done` — not new code)

| Task | Witness | Action |
|:---|:---|:---|
| TRIAGE-MAP-ZOOM-SMOOTH-001 | `map_zoom_coherence_live.json` | done |
| BUILD-READ-REWIRE-003 | `map_zoom_coherence_live.json` | done |
| SIM-EFFECT-QUEUE-001 | `sim_effect_spine_live.json` | done |
| SIM-EFFECT-TEL-001 | `sim_effect_spine_live.json` | done |
| FIRE-IGNITION-P0-001 | `fire_ecology_live.json` | done |
| PLAN-LANDSCAPE-GRAMMAR-001 | `landscape_grammar_lg1_live.json` | done |
| LG-2-SUCCESSION-001 | `landscape_grammar_lg2_live.json` | done |
| APS-QC-REWIRE-001 | `aps_bevy_qc_hud_001_live.json` | done |

### Tier B — **REAL OPEN** (keep on board)

| Task | Owner | Why still open |
|:---|:---|:---|
| **PERF-INSTR-VFX-002** | coder_b | Needs `cargo run … -- --test vfx` acceptance (display) |
| **BUILD-READ-REWIRE-004** | coder | `pilot_hardcode_lint_live.json` **red** (6 violations) — catalog authority tail |

### Tier C — **NOT CODER** (route off main pick)

| Task | Owner | Note |
|:---|:---|:---|
| G-PLAY-01 | operator | Play scenario checklist |
| VEG G4 art-ship | operator / designer-mcp | G4 signoff YAML |
| PLAN-SIM-EFFECT-SPINE-001 | planner | Doc sign-off only — unblocks nothing once Tier A closed |

## Drain pass 2026-06-21 (session 2)

**31 additional rows** closed (MCP + veg + nested `coder_active` picks). Fixed broken queue JSON (`parallel_wave_aps_veg_dispatch_v1.json`, `planner_active_queue.json`).

| Lane | pick_now before | pick_now after |
|:---|:---|:---|
| coder | 0 | **0** |
| coder-mcp | 8 | **0** |
| coder_a | 9 | **0** |
| coder_b | 1 | **1** (PERF only) |
| operator | 4 | 4 (human) |

## Drain pass 2026-06-21 (session 1)

**21 rows** auto-closed via `reconcile_coder_crisis.py`. **`pick_now.coder`: 9 → 0.**

| Before | After |
|:---|:---|
| 9 reopened coder picks flooding hub | **0** coder picks |
| Downstream agents treating stale `reopened` as new work | Witness-green rows marked `done` |

**Still open (real work):**

| Task | Owner | Status |
|:---|:---|:---|
| PERF-INSTR-VFX-002 | coder_b | in_progress — needs display VFX run |
| BUILD-READ-REWIRE-004 | coder | **deferred** — pilot lint transitional until BUILD-SET-GUARD-002 |

## After filter — expected `pick_now.coder`

**0 rows.** Primary lane: **coder_b → PERF-INSTR-VFX-002**.

## Downstream rule

Do **not** treat `status: reopened` as actionable without witness check. **Witness JSON wins** over queue status.
