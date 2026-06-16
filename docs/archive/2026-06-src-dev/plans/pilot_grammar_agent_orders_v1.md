# PILOT-GRAMMAR-001 — Agent orders + queue drain `v1`

| Field | Value |
|:---|:---|
| **Program** | PILOT-GRAMMAR-001 (IndustrialWarehouse · `style_industrial_west`) |
| **Execution** | [`pilot_grammar_001_execution_v1.md`](pilot_grammar_001_execution_v1.md) |
| **Queue** | [`tools/orchestrator/queues/grammar_continuation_queue.json`](../tools/orchestrator/queues/grammar_continuation_queue.json) |
| **HANDOFF** | [`tools/orchestrator/queues/HANDOFF.md`](../tools/orchestrator/queues/HANDOFF.md) |
| **Status** | Track B grammar E2E **done** · **ARCH-MAT / BUILD-WORKER / APS-MAT-002/003/008 done** · **PG-MODULE-AUDIT-002 P0–P3 done** · warehouse ship **blocked** on P6 manual keyframe + G4 |
| **Planner hub** | [`plan_three_track_execution_v1.md`](plan_three_track_execution_v1.md) — locked P0–P6 order |

---

## Locked planner order (three tracks — 2026-06-03)

| Priority | ID | Owner | Status |
|:---:|:---|:---|:---:|
| **P0** | **ARCH-MAT-001** / BUILD-WORKER-001 | @coder-mcp | **done** — snapshot `material_profile` → worker bake paths |
| **P1** | **APS-PREVIEW-001** | @coder-mcp | **done** — four slot thumbs + grammar “why” |
| **P2** | **APS-MAT-002** | @coder-mcp | **done** — Materials tab `studio_tree` at scale |
| **P3** | **APS-MAT-003** / **APS-MAT-008** | @coder-mcp | **done** — thumb cache rows + `validate-material-profiles` gate |
| **P4–P5** | GRAMMAR-001/002 | @coder | Massing / facade maturity |
| **P6** | Warehouse Track B | operator + @designer-mcp | **blocked** on real keyframe + G4 (integration test only) |

Warehouse ship is **P6**, not P0 — artists should preview in APS **without Blender** first (P1).

---

## Updated paradigms (all agents — mandatory)

| Rule | Meaning |
|:---|:---|
| **Material authority** | `material_profile` on assembly snapshot / APS only — **not** Blender viewport painting ([`arch_material_authority_001_v1.md`](arch_material_authority_001_v1.md)) |
| **P0 gate** | `validate-report assembly_p0` before Save/Preview ship paths — blocks `StylePackDrift`, thin footprint, missing grammar chain ([`assembly_grammar_verify.py`](../tools/mcp/python/rust_engine_mcp/validators/assembly_grammar_verify.py)) |
| **Validation-first** | `validate-report` / MCP validators — **no** raw cargo/blender logs unless `confidence < 0.7` ([`.cursor/rules/validation-first.mdc`](../.cursor/rules/validation-first.mdc)) |
| **Ship art** | Manual `keyframe_render.py` → 24 PNGs (3×8) → designer G4 — **not** `tile_compile_minimum_bake` / headless ortho ([`mcp_orchestrator_tile_fix_warehouse_slice_v2.md`](mcp_orchestrator_tile_fix_warehouse_slice_v2.md)) |
| **Witness honesty** | `mcp_pilot_grammar_001_live.json` stays `green: false` until Track B Phase 6; rejected folder de-indexed |
| **Placement-only** | Snapshots without `grammar_rule_chain` / archetype are **out of scope** for closing PILOT |

---

## Critical path (Track B warehouse — P6 only)

| # | ID | Owner | Status | Unblocks |
|:---:|:---|:---|:---:|:---|
| 0 | **ARCH-MAT-001** + **BUILD-WORKER-001** | @coder-mcp | **done** | Honest material apply on worker + combined thumb |
| 1 | **PILOT-GRAMMAR-E2E-001** | @coder-mcp | **done** | Grammar snapshot witness |
| 2 | **APS-PREVIEW-001** + **APS-MAT-STUDIO** | @coder-mcp | **done** | Slot previews + Materials tab Phase A |
| 3 | **APS-MAT-002/003/008** | @coder-mcp | **done** | Tree + thumb rows + material_profiles validator |
| 4 | **MCP-PILOT-GRAMMAR-001** | operator + @designer-mcp | **blocked** | Manual keyframe + G4 ([`pilot_grammar_operator_runbook_v1.md`](pilot_grammar_operator_runbook_v1.md)) |
| 5 | **Phase 7 register + map stamp** | @coder-mcp → @coder | **blocked** | After `proceed_ship: yes` |

**Parallel:** **PG-MODULE-AUDIT-002** / **assembly_p0** — keep green on pilot assembly before P6 ship attempt.

**Planner / planner-mcp:** **on-call** — no new mega-plans; thin doc only if PBG massing slice is requested ([`arch_pbg_massing_placement_v1.md`](arch_pbg_massing_placement_v1.md)).

---

## Queue-drain protocol (coder, coder-mcp, designer, designer-mcp)

On **every** session:

1. Read [`grammar_continuation_queue.json`](../tools/orchestrator/queues/grammar_continuation_queue.json) — filter `agent` = your role, `status` ∈ `ready`, `in_progress`.
2. Work **critical path #2→#3** first if your role is `coder-mcp` and those rows are not `done`.
3. If **blocked**, set `blocked_by`, then drain **next ready row in your lane** (do not return wait-only).
4. **Skip** rows marked `done` — do not redo: APS-TAGS-002, UI-003b, MATERIAL-BROWSER, PREVIEW-002/004, PILOT-GRAMMAR-E2E-001, PG-MATERIAL-GENERATION (Rust/Python defaults).
5. **Cross-lane backlog** (when grammar lane idle): [`fleet_coder_workload_queue_20260602_v1.md`](fleet_coder_workload_queue_20260602_v1.md) · [`coder_unified_backlog_v1.md`](coder_unified_backlog_v1.md) — pick **one** ready row, same validation-first rules.
6. Checkpoint [`HANDOFF.md`](../tools/orchestrator/queues/HANDOFF.md): last ID, witness path, next ready ID.

```powershell
# Optional queue sync (if orchestrator CLI wired)
cd C:\dev\github\Rust_engine_template_01
cargo orchestrate --agent-queue-next coder-mcp --queue grammar
```

---

## Paste — @coder-mcp (primary — drain steps 2–3 + parallel P0 modules)

```text
You are @coder-mcp on Rust_engine_template_01 (branch master). PILOT-GRAMMAR Track B prep — NOT ship G4 yet.

READ FIRST (in order):
- docs/archive/2026-06-src-dev/plans/pilot_grammar_agent_orders_v1.md  (this file — paradigms + drain)
- docs/archive/2026-06-src-dev/plans/pilot_grammar_001_execution_v1.md   (Track A done / Track B blocked)
- docs/archive/2026-06-src-dev/plans/plan_material_studio_phase_v1.md    (Phase A = APS-MAT-001/002/006/007)
- docs/archive/2026-06-src-dev/plans/arch_blender_worker_contract_v1.md   (BUILD-WORKER roles)
- docs/archive/2026-06-src-dev/plans/pg_module_audit_warehouse_v1.md     (PG-MODULE-AUDIT-002 P0 job list)
- tools/mcp/README.md

CRITICAL PATH (do in order; ship witness each slice):

STEP 2 — Material Studio Phase A (APS-MAT-001…007 MVP)
- Add Materials notebook tab (app.py) — library + preview stack per plan_material_studio_phase_v1.md § A1–A3
- Shared material_browser widget: category filter + search
- Preview modes: sphere, wall strip (building section optional/degraded)
- Assembly: "open in Materials tab" + applied profile display
- Witness: debug_runs/aps_material_studio_live.json
- Do NOT reopen Assembly-only browser work (MCP-APS-MATERIAL-BROWSER-001 = done)

STEP 3 — BUILD-WORKER-001 (verify + harden)
- Snapshot material_profile → assembly_import apply_material_profile_to_meshes (authority path)
- Witness: debug_runs/build_worker_001_live.json — refresh if you change apply
- Manual keyframe_render MUST use same apply — NEVER use tile_compile_minimum_bake / keyframe_minimum_cell for ship proof
- Hook validate-report material profiles (APS-MAT-008) if missing

PARALLEL — PG-MODULE-AUDIT-002 (P0 green)
- Fix style_industrial_west production corner_L (not Victorian corner_L_production_run001)
- Production door_warehouse for bdef slot door_wide
- Regenerate warehouse assembly; validate-report assembly_p0 MUST pass before declaring prep done

ALREADY DONE — DO NOT REDO:
- PILOT-GRAMMAR-E2E-001 (debug_runs/pilot_grammar_001_grammar_e2e_live.json)
- APS scroll + P0 gate button in assembly_panel
- bevy_preview_worker / preview-assembly CLI

VALIDATION-FIRST:
python -m rust_engine_mcp.cli validate-report assembly_p0 assets/staging/assemblies/<id>.json --compress 3
python -m pytest tools/mcp/python/tests/ -q

OUT OF SCOPE:
- designer G4 / proceed_ship
- tile_compile_minimum_bake for warehouse ship
- promote rejected tile_warehouse_industrial_v2_minimum_g4/

EXIT: Update grammar_continuation_queue.json rows → done/blocked + HANDOFF.md next = designer-mcp MCP-PILOT-GRAMMAR-001 when steps 2–3 + P0 green on pilot assembly.
```

---

## Paste — @designer-mcp (step 4 — only after coder-mcp prep)

```text
You are @designer-mcp on Rust_engine_template_01. MCP-PILOT-GRAMMAR-001 Track B — human keyframe + G4 ONLY.

HARD BLOCK until ALL true:
- validate-report assembly_p0 passed on pilot assembly (no StylePackDrift)
- BUILD-WORKER material apply verified on assembly blend preview (not grey slabs)
- Material Studio Phase A minimum: profiles assignable + witness aps_material_studio_live.json green
- Operator confirms APS preview acceptable

READ FIRST:
- docs/archive/2026-06-src-dev/plans/pilot_grammar_agent_orders_v1.md
- docs/archive/2026-06-src-dev/plans/pilot_grammar_001_g4_checklist_v1.md (Phases 4–6)
- docs/archive/2026-06-src-dev/plans/mcp_orchestrator_tile_fix_warehouse_slice_v2.md
- debug_runs/art_pipeline/mcp_pilot_grammar_001_rejected_live.json (what NOT to repeat)

OPERATOR + YOU (Phase 4–6):
1. cleanup_assembly_blends.py — ASSEMBLY collection only (no truck/rig embedded)
2. Blender: append Tile_iso_rig_v1 at bake time; keyframe_render.py — 3 states × 8 facings = 24 PNGs
3. tile-atlas-pack from MANUAL folder (not minimum_g4 staging)
4. G4: art_quality keyframe_manual; facing rotation must differ (promotion gate FacingRotationMissing)
5. write-tile-fix-designer-g4-witness — proceed_ship: yes ONLY if operator accepts stills

FORBIDDEN:
- tile_compile_minimum_bake.py / designer_mcp_pilot_grammar_keyframe.py headless ship path
- Marking green on schema-only or 1×1 / identical facings
- Blender viewport material painting (APS snapshot authority)

VALIDATION-FIRST:
validate-report tile_batch / asset_glb / mcp_spec — compress 3

EXIT: mcp_pilot_grammar_001_live.json green:true ONLY after Phase 6; then hand off Phase 7 to @coder-mcp.
```

---

## Paste — @coder-mcp (step 5 — register, after G4)

```text
You are @coder-mcp — PILOT Phase 7 ONLY. Blocked until designer-mcp sets proceed_ship: yes.

READ: docs/archive/2026-06-src-dev/plans/pilot_grammar_001_g4_checklist_v1.md Phase 7, pilot_grammar_agent_orders_v1.md

DO:
1. tile-atlas-register (or CLI register) for manual batch id — re-index _tile_atlas_index.ron
2. validate-report tile_promotion / facing / art_quality gates on REAL 24 PNGs
3. Witness: debug_runs/art_pipeline/mcp_pilot_grammar_001_live.json green:true

THEN @coder: map stamp smoke (TileAtlasRegistry resolves pilot batch in sim view).

DO NOT register minimum_g4 / procedural rejected folder.
```

---

## Paste — @coder (step 5b + backlog drain)

```text
You are @coder on Rust_engine_template_01. Grammar Rust lane is IDLE unless regression breaks.

WHEN PILOT Phase 7 lands:
- Wire TileAtlasRegistry / map tactical stamp for warehouse pilot batch id
- cargo test witness for atlas handle resolution

QUEUE-DRAIN (grammar idle):
- Read tools/orchestrator/queues/grammar_continuation_queue.json — coder-owned ready rows only
- Else ONE row from docs/archive/2026-06-fleet-drain/fleet_closed/fleet_coder_workload_queue_20260602_v1.md (construction P3 / infra — not grammar MCP UI)

DO NOT: tools/mcp Python (coder-mcp), designer G4, reopen PG-QUALITY / snapshot wire (done).

VALIDATION-FIRST: validate-report cargo --compress 3
```

---

## Paste — @designer (on-call + audit tails)

```text
You are @designer. Grammar prep lane DONE (PG-MODULE-AUDIT-001 + G4 checklist).

WHEN @designer-mcp requests review:
- Consumer G4 on manual 128px stills per pilot_grammar_001_g4_checklist_v1.md
- Reject headless minimum / identical facings / magenta slots

PARALLEL (doc-only, no bpy):
- PG-MODULE-AUDIT-002 sign-off when coder-mcp lands production corner/door
- Charter updates for new production modules (delegate geometry to @designer-mcp)

DO NOT: mark PILOT done on grammar code alone; do not reopen placement-only pilot.

READ: docs/archive/2026-06-src-dev/plans/pilot_grammar_agent_orders_v1.md, agent_mcp_consumer_guide_v1.md § @designer
```

---

## Paste — @planner / @planner-mcp (on-call)

```text
Grammar planner stop points are DONE. Do not replan APS-TAGS / ARCH-ASSEMBLY-GRAPH / PREVIEW-004.

ON-CALL ONLY:
- Thin slice doc if user requests PBG mesh-face massing (arch_pbg_massing_placement_v1.md) — orthogonal to PILOT ship
- Extend material_profiles_v1.json categories for APS-MAT-006 if coder-mcp asks (schema/category fields only)

DRAIN elsewhere: PLAN-SETTLEMENT-HIERARCHY-005, PLAN-CONSTRUCTION-SCALING-AUDIT-003 per fleet_planner_designer_prompts_20260602_v2.md

NO: bpy, tile bake, reopen L1315 material authority debate.
```

---

## Paste — @orchestrator (parent chat)

```text
Run PILOT-GRAMMAR-001 critical path with queue-drain:
docs/archive/2026-06-src-dev/plans/pilot_grammar_agent_orders_v1.md

Order: (1 E2E done) → coder-mcp Material Studio A → coder-mcp BUILD-WORKER + PG-MODULE-AUDIT-002/P0 → designer-mcp manual keyframe G4 → coder-mcp register → coder map stamp.

Never mark ship green on headless minimum bake. Keep mcp_pilot_grammar_001_live.json false until Phase 6.

When coder-mcp blocked on designer, drain PG-MODULE-AUDIT-002 or fleet coder workload — never wait-only.
```

---

## Verification chain

```powershell
cd C:\dev\github\Rust_engine_template_01\tools\mcp\python

# P0 (must pass before designer handoff)
python -m rust_engine_mcp.cli validate-report assembly_p0 assets/staging/assemblies/industrial_west_7x5_s39_9fa1.json --compress 3

# Track A regression
python -m rust_engine_mcp.cli validate-report assembly_grammar_verify assets/staging/assemblies/<id>.json --compress 3

# Material studio witness (after step 2)
# expect: debug_runs/aps_material_studio_live.json

# BUILD-WORKER (after step 3)
# expect: debug_runs/build_worker_001_live.json — materials.ok true; ship stills = manual only

# Rust grammar
cd ..\..
cargo test -p proc_A_dine01 --lib building_grammar assembly_snapshot_grammar_wire
python -m pytest tools/mcp/python/tests/test_assembly_grammar_verify.py -q
```

---

## Changelog

| Version | Date | Notes |
|:---|:---|:---|
| v1.0.0 | 2026-06-03 | Agent paste orders aligned to planner critical path 1–5 + updated paradigms |
