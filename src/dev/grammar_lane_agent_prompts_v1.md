# Grammar program — agent prompts + queue-drain protocol

| Field | Value |
|:---|:---|
| **Program** | PLAN-BUILDING-GRAMMAR-001 |
| **Plan** | [`plan_building_grammar_evolution_v1.md`](plan_building_grammar_evolution_v1.md) |
| **Prereq done** | ARCH-BUILD-GRAMMAR-001/002/003 |
| **Queue file** | [`tools/orchestrator/queues/grammar_continuation_queue.json`](../tools/orchestrator/queues/grammar_continuation_queue.json) |

---

## Queue-drain rule (all agents)

**Do not end a session with only “waiting on X.”**

Each agent owns a lane queue. On every turn:

1. **Try the stop-point slice** for this session (below).
2. If **blocked** (dependency not `done`), set that item `status: blocked` in the queue, record `blocked_by: [IDs]`, then **immediately take the next `ready` item in your lane** (same agent field).
3. Repeat until you ship something (code, schema, witness, G4 note) **or** the lane queue is empty / all items blocked.
4. **Checkpoint** in `tools/orchestrator/queues/HANDOFF.md`: last completed ID, next ready ID, blockers.

**Stop point** = milestone that unblocks the *next* agent’s stop point (not “whole program done”).

| Agent | Stop point (this phase) | Unblocks |
|:---|:---|:---|
| **@planner** | APS-TAGS-001 + ARCH-ASSEMBLY-GRAPH-002 schemas committed | @coder-mcp APS-TAGS-002, inspector |
| **@coder** | PG-QUALITY-001 witness green + Rust uses grammar in snapshot path | @designer pilot criteria |
| **@designer** | PG-MODULE-AUDIT-001 sign-off doc + warehouse G4 checklist draft | PILOT-GRAMMAR-001 bake |

**Sync (MCP — do not load full queue into chat):**

```text
agent_queue_next("planner"|"coder"|"designer", queue="grammar", mark_in_progress=true)
# … work …
agent_queue_update(slice_id, "done", note="witness or schema path")
```

Also: `token_savings_guide()` once per session · `handoff_brief()` instead of full HANDOFF.md.

---

## @planner — paste prompt

```text
Lane: PLAN-BUILDING-GRAMMAR-001 (T1–T3 only — no tile/atlas/validators).

Stop point: APS-TAGS-001 + ARCH-ASSEMBLY-GRAPH-002 schemas landed and indexed in plan_building_grammar_evolution_v1.md.

Queue-drain: Read tools/orchestrator/queues/grammar_continuation_queue.json. Work planner-owned items in priority order. If a slice is blocked, mark it blocked and continue with the next ready planner item — do not return wait-only.

Deliverables this phase:
1. APS-TAGS-001 — schema: Location / Architectural / Detail / Condition tag categories; map from flat APS checkboxes.
2. ARCH-ASSEMBLY-GRAPH-002 — extend assembly_graph_node_v1 / snapshot contract: role, material_profile, style, weathering, semantic placement_tags.
3. APS-PREVIEW-004 — one-page architecture only (Bevy preview worker), no implementation.

Read first: src/dev/plan_building_grammar_evolution_v1.md, src/dev/arch_build_grammar_001_schema_v1.md, tools/mcp/schemas/assembly_graph_node_v1.schema.json.

Out of scope: MCP bpy, tile bake, promotion, render extract.

Exit: Update grammar_continuation_queue.json (PLAN-* rows → done/blocked) + HANDOFF.md next ready slice for @coder.
```

---

## @coder — paste prompt

```text
Lane: PLAN-BUILDING-GRAMMAR-001 — Rust T1 core only.

Stop point: PG-QUALITY-001 witness JSON green; procedural snapshot path can consume GrammarGenerateResult (footprint + slot_overrides + rule_chain on snapshot).

Queue-drain: Read tools/orchestrator/queues/grammar_continuation_queue.json. Drain coder-owned ready items when blocked on planner schemas — e.g. wire grammar into build_assembly_snapshot while waiting on APS-TAGS-001.

Deliverables this phase:
1. Wire building_grammar::generate into src/construction/procedural/assembly_snapshot.rs (grammar before footprint fill; persist rule_chain in reference_tags or dedicated field per planner schema).
2. PG-QUALITY-001 — debug_runs/grammar_diversity_witness.json: seed sweep metrics (massing count, roof slot diversity) for IndustrialWarehouse + industrial_west.
3. Tests: cargo test -p proc_A_dine01 --lib construction::procedural::building_grammar construction::procedural::assembly_snapshot

Read first: src/construction/procedural/building_grammar.rs, footprint_grid.rs, assembly_snapshot.rs. Do not touch tools/mcp tile jobs.

Validators: validate-report cargo --compress 3 (no raw log unless confidence < 0.7).

Exit: Queue CODER-* rows updated + HANDOFF.md with witness path and flags.
```

---

## @designer — paste prompt

```text
Lane: PLAN-BUILDING-GRAMMAR-001 — module kit + ship criteria (no ECS/render).

Stop point: PG-MODULE-AUDIT-001 complete + PILOT-GRAMMAR-001 G4 checklist draft for IndustrialWarehouse (manual keyframe path — not headless minimum bake).

Queue-drain: Read tools/orchestrator/queues/grammar_continuation_queue.json. If PILOT-GRAMMAR-001 is blocked on APS-UI or materials, complete PG-MODULE-AUDIT-001 and doc-only G4 gates instead — do not idle.

Deliverables this phase:
1. PG-MODULE-AUDIT-001 — audit assets/configs/buildings/_module_index.ron vs categories (walls, roofs, corners, windows, doors, stacks, vents, pipes, platforms, signs, lights, AC, cranes) for warehouse/industrial west.
2. PILOT-GRAMMAR-001 prep — checklist: grammar snapshot → clean assembly blend (ASSEMBLY only) → Tile_iso_rig_v1 + materials → keyframe_render → G4 stills 128px.
3. Reject: tile_compile_minimum_bake / headless ortho as ship art (see mcp_orchestrator_tile_fix_warehouse_slice_v2.md).

Read first: src/dev/plan_building_grammar_evolution_v1.md, utils/TILE_ISO_RIG_README.md, assets/configs/buildings/style_packs/style_industrial_west.ron.

Exit: src/dev/pg_module_audit_warehouse_v1.md (or signoff yaml) + queue DESIGN-* rows + HANDOFF.md.
```

---

## Orchestrator one-liner (parent chat)

```text
Run grammar program with queue-drain: agents use grammar_continuation_queue.json, advance to lane stop points, never wait-only — drain own lane when blocked. Order: planner schemas → coder Rust witness → designer audit/G4 prep → coder-mcp APS (separate chat).
```

---

## Optional: @coder-mcp / @designer-mcp (parallel APS lane)

Use when APS UI / Blender / MCP is ready (after planner APS-TAGS-001):

**@coder-mcp stop point:** APS-TAGS-002 + APS-GRAMMAR-INSPECTOR-001 + APS-UI-003b footprint grid (grammar inspector reads rule_chain).

**@designer-mcp stop point:** PG-MODULE-AUDIT-002 gap jobs signed + PILOT-GRAMMAR-001 G4 on real keyframe stills.

Same queue-drain rules; agent field `coder-mcp` / `designer-mcp` in JSON.
