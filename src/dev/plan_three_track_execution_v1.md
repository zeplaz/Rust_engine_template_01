# PLAN-THREE-TRACK-001 — APS · Spine · Content (parallel lanes) `v1`

| Field | Value |
|:---|:---|
| **Program ID** | **PLAN-THREE-TRACK-001** |
| **Source** | Planner correction (2026-06-03) — fixes drift: warehouse pilot ≠ production workflow definition |
| **Status** | **ACTIVE** |
| **Index** | [`development_plan_index.md`](development_plan_index.md) |

---

## Planner correction (read first)

**Wrong framing:** “Pause warehouse until materials.”

**Right framing:**

| Statement | Meaning |
|:---|:---|
| **Warehouse stays active** | Track B **integration test** — does the spine work? |
| **Production sign-off blocked** | Until ARCH-MAT-001 + honest validators + clear preview state |
| **Warehouse pilot answers** | “Does snapshot → worker → render → atlas → runtime work?” |
| **Warehouse pilot does NOT answer** | “Is this the final artist workflow?” |

Three tracks run **in parallel**, not as one serial gate.

---

## Track A — APS Product

**Goal:** Can an artist work efficiently?

| Success | Means |
|:---|:---|
| Preview works | Selected module + assembly visible before bake |
| Materials work | ARCH-MAT-001 in APS, not Blender |
| Assembly editing works | Authoring tool, not raw data editor |
| Validation understandable | Plain-language gates, P0 vs ship |
| Pipeline visible | Status: Grammar / Assembly / Materials / Variants / Preview / Atlas / Validation |

**Primary files:** `tools/mcp/art_pipeline_suite/*`, `rust_engine_mcp/assembly*.py`, `assembly_preview.py`

**Pain today:** Assembly panel is **functional but not informative** — shows `steel_panel_01` but not *what it looks like*, *why generated*, *what mesh*.

### Near-term priority order (planner — 2026-06-03)

| Priority | ID | Deliverable |
|:---|:---|:---|
| **P0** | **ARCH-MAT-001** | Enforce snapshot material authority — [`arch_mat_001_material_authority_v1.md`](arch_mat_001_material_authority_v1.md) |
| **P1** | **APS-PREVIEW-001** | Selected slot: module + material (wall/sphere) + combined + context — [`aps_preview_001_spec_v1.md`](aps_preview_001_spec_v1.md) |
| **P2** | **APS-MAT-002** | Real material browser at scale (not combobox for 300 profiles) |
| **P3** | **APS-MAT-003** | Thumbnails + Industrial/Residential category tree |
| **P4** | **GRAMMAR-001** | Archetype → massing layer maturity (Track C) |
| **P5** | **GRAMMAR-002** | Facade + roof strategies |
| **P6** | **Warehouse Track B** | Spine completion — **PAUSED** ([`plan_bevy_hud_grammar_parallel_v1.md`](plan_bevy_hud_grammar_parallel_v1.md)) — does not block A / A′ / C / product HUD |

**Rationale:** previews unlock understanding; warehouse remains integration test but ship waits on honest preview + authority.

### Lane A backlog (detail)

| ID | Deliverable | Status |
|:---|:---|:---:|
| APS-PREVIEW-001 | Four-panel slot previews + grammar “why” hint | **done** (UI); context thumb from assembly preview = follow-up |
| ARCH-MAT-001 | Hard rule doc + worker enforcement | **active** — BUILD-WORKER-001 pending |
| APS-MAT-002/003 | Material Studio browser at scale | **partial** — grid exists; APS-MAT-002 = dedicated tab scale-up |
| APS-UX-AUTHORING-001 | Pipeline status row | pending |
| APS-PREVIEW-002/004 | Assembly-level Bevy/browser | **done** |
| Grammar inspector | Republic-style chain + seed + material strategy | **expanded** |
| Material browser (assign) | Click-apply on snapshot | **done** |

---

## Track B — Pipeline Spine

**Goal:** Can the asset move through the system?

**Success:** Not pretty. Not final art. **Proven.**

```text
Snapshot → Worker → Render → Atlas → Runtime
```

| Step | Proof |
|:---|:---|
| Snapshot | Grammar E2E witness — [`pilot_grammar_001_grammar_e2e_live.json`](../../debug_runs/pilot_grammar_001_grammar_e2e_live.json) |
| Worker | `assembly-build-run`, material apply (BUILD-WORKER-001), `bevy_preview_worker` |
| Render | Real manual `keyframe_render` — **not** headless minimum bake as ship |
| Atlas | `tile-atlas-pack` from manual PNGs |
| Runtime | `--register`, map stamp smoke |

**Primary files:** `tools/mcp/python/rust_engine_mcp/tile_*`, `tools/mcp/blender/`, `assets/staging/tiles/`, runtime atlas lookup

**Warehouse** = Track B **integration test** ([`pilot_grammar_001_execution_v1.md`](pilot_grammar_001_execution_v1.md)).

| Track B row | Status |
|:---|:---|
| PILOT-GRAMMAR-E2E-001 (grammar snapshot) | **done** |
| MCP-PILOT-GRAMMAR-001 (ship / G4) | **blocked** — art rejection; spine retest after BUILD-WORKER + real keyframe |
| BUILD-WORKER-001 | **pending** |
| Register / runtime | **pending** on G4 |

---

## Track C — Content Production

**Goal:** Can we actually make buildings that look good?

**Success:** Warehouse, Factory, Office, Power Plant — **content quality**, not spine proof.

| Layer | Maturity needed |
|:---|:---|
| Grammar | archetype → massing → facade → roof → detail → **material strategy** → aging |
| Materials | APS-MAT library as important as module catalog |
| Modules | PG-MODULE-AUDIT-002 production GLBs |

**Risk:** Weak grammar → better previews/atlases/materials still yield **boring buildings** (Republic-style depth).

**Primary files:** `assets/configs/buildings/grammars/`, `building_grammar.rs`, style packs, module index

**Lane C backlog:**

| ID | Focus |
|:---|:---|
| PLAN-BUILDING-GRAMMAR-001 | Hierarchy evaluator (in progress) |
| ARCH-PBG-MASSING-001 | Perimeter vs mesh-face (doc gate) |
| APS-MAT-003+ | Material recipes / layers (later) |
| Content pilots | Per archetype **after** Track B spine green once |

**Rule:** Track C does **not** block Track B spine proof; it blocks **production sign-off**.

---

## Parallel execution (agents)

**Warehouse keyframe ship is PAUSED** — assign lanes below without waiting on G4.

```text
Lane A  (APS Tk)       BUILD-WORKER-001 · APS-MAT-002/003 · APS-PREVIEW follow-ups
Lane A′ (Bevy tools)   bevy_preview_worker polish · APS-BEVY-QC-HUD-001 egui QC
Lane C  (Grammar)      GRAMMAR-001/002 · GRAMMAR-ITER-001 iterative UX
Lane product           SIM-HUD-PRODUCT-001 — in_game_hud / PLAY-01 (not Tk APS)
Lane B  (Spine test)   warehouse keyframe + G4 — PAUSED until artist-ready
```

Hub: [`plan_bevy_hud_grammar_parallel_v1.md`](plan_bevy_hud_grammar_parallel_v1.md)

---

## Production sign-off gates (all tracks)

Sign-off **blocked** until:

1. **ARCH-MAT-001** — materials only via snapshot/APS  
2. **Validators honest** — no fake `keyframe_manual`; G4 on real stills  
3. **Preview state clear** — artist sees module + material + assembly before bake  

Warehouse G4 **`proceed_ship: yes`** is Track B + C bar, not Track A complete.

---

## What to stop doing

| Stop | Do instead |
|:---|:---|
| Treat warehouse pilot as final APS workflow spec | Track A roadmap for artist efficiency |
| “Pause all warehouse work” | Run spine tests; block **sign-off** only |
| Another validation checkbox before preview | **APS-PREVIEW-001** selected-slot thumb |
| Collapse grammar + ship + UX into one todo | Three tracks, three witnesses |

---

## Track D — OPS Witness Spine (telemetry)

**Goal:** Machine-readable three-track status for all agents — no HANDOFF-only truth.

| ID | Deliverable | Status |
|:---|:---|:---:|
| OPS-WITNESS-SPINE-001 | `unified_witness_index.json` + `ops_report_latest.json` | **done** |
| OPS-WITNESS-SPINE-002 | `agent_run_event_v1` schema + HANDOFF `-OpsEvent` | **done** |
| OPS-WITNESS-SPINE-003 | Art paths in `debug_run_envelope.rs` + `honest_gate` summary | **done** |
| OPS-WITNESS-SPINE-004 | Skill + all-agent `OPS_WITNESS_SPINE.md` contract | **done** |
| OPS-WITNESS-SPINE-005 | `OPS_LANE_REGISTRY.json` — engine + art programs | **done** |
| OPS-WITNESS-SPINE-006 | Construction sub-witness rollup + HANDOFF priorities in ΔWF | **done** |

```powershell
powershell -File tools/orchestrator/scripts/ops_intelligence_scan.ps1
```

Skill: [`.cursor/skills/operations-intelligence/SKILL.md`](../../.cursor/skills/operations-intelligence/SKILL.md) · Agent: `@operations-intelligence`

---

## Witness index

| Witness | Track |
|:---|:---|
| `pilot_grammar_001_grammar_e2e_live.json` | B — grammar spine |
| `aps_preview_002_live.json` / `preview_worker_smoke_live.json` | A — assembly preview |
| `aps_material_browser_live.json` | A — material assign |
| `mcp_pilot_grammar_001_rejected_live.json` | B — ship blocked |
| `pilot_grammar_001_ship_live.json` (future) | B — spine + G4 pass |

---

## Changelog

| Version | Date | Notes |
|:---|:---|:---|
| v1.0.0 | 2026-06-03 | Three-track model + planner drift fix |
