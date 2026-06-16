# GRAMMAR-ITER-001 — Agent orders (orchestrator dispatch) `v1`

| Field | Value |
|:---|:---|
| **Program** | GRAMMAR-ITER-001 |
| **Lang** | [`agent_lang_v1.md`](agent_lang_v1.md) — `⟨ID⟩` · `$ref:` · BLANG |
| **Spec** | $ref:docs/archive/2026-06-src-dev/plans/grammar_iter_001_spec_v1.md — **SPEC READY** |
| **Schema** | $ref:tools/mcp/schemas/grammar_iterate_request_v1.schema.json |
| **Queue** | $ref:tools/orchestrator/queues/grammar_continuation_queue.json |
| **Parallel** | APS artist tool · Bevy preview · **⏸** keyframe — $ref:tools/orchestrator/queues/defer_registry.json |

---

## Orchestrator assignment (locked)

| Order | ⟨ID⟩ | Agent | Deliverable | φ |
|:---:|:---|:---|:---|:---:|
| 0 | ⟨GRAMMAR-ITER-001-SPEC⟩ | @planner-mcp | COMMIT:SPEC | 🟢 |
| 1 | ⟨GRAMMAR-ITER-001-UI⟩ | @designer | $ref:prompts/designer_questions/grammar_iter_wireframe_v1.md | 🟢 |
| 2 | ⟨GRAMMAR-ITER-001-APS1⟩ | @coder-mcp | footprint diff UI Phase 1 | ○ |
| 3 | ⟨GRAMMAR-ITER-001-API⟩ | @coder-mcp + @coder | `iterate_grammar` massing/material/placement | ○ |
| 4 | ⟨GRAMMAR-ITER-001-RUST⟩ | @coder | Rust parity | ○ |
| 5 | ⟨GRAMMAR-ITER-001-E2E⟩ | @coder-mcp | COMMIT:WIT $ref:debug_runs/grammar_iter_001_massing_live.json | 🧩⟨3⟩ |

**Does not wait on:** ⏸ WH-TRACK-B · 🧊 MCP-PILOT-GRAMMAR-001 · G4.

---

## Paste — @designer (GRAMMAR-ITER-001-UI)

```text
⟨GRAMMAR-ITER-001-UI⟩ 🟢 — archived paste
$ref:docs/archive/2026-06-src-dev/plans/grammar_iter_001_spec_v1.md§APS-UX
$ref:prompts/designer_questions/grammar_iter_wireframe_brief_v1.md

TASK: wireframe + control labels for Assembly "Iterate grammar" panel.

DELIVER:
1. Annotated wireframe — prompts/designer_questions/grammar_iter_wireframe_v1.md (ASCII or markup)
2. Review/edit assets/configs/buildings/grammars/grammar_labels_v1.json (IndustrialWarehouse pilot — human labels for massing/facade/roof)
3. Footprint diff legend: green=added, red=removed, yellow=changed (align with spec)
4. Accessibility: Mode + Apply iteration visible without hover-only; Pin district/age checkboxes labeled
5. Sign-off note in wireframe doc: PASS | PASS WITH NOTES

OUT OF SCOPE: Rust/Python implementation, Blender, warehouse keyframe ship.

HANDOFF: Queue row GRAMMAR-ITER-001-UI → done; unblock GRAMMAR-ITER-001-APS1 layout polish after wireframe lands.
```

---

## Paste — @coder-mcp (GRAMMAR-ITER-001-APS1)

```text
BLANG:Q+("coder-mcp") · ⟨GRAMMAR-ITER-001-APS1⟩
$ref:docs/archive/2026-06-src-dev/plans/grammar_iter_001_spec_v1.md§Phase-1
$ref:docs/archive/2026-06-src-dev/plans/grammar_iter_agent_orders_v1.md
$ref:tools/mcp/schemas/grammar_iterate_request_v1.schema.json
$ref:tools/mcp/art_pipeline_suite/grammar_inspector.py

TASK: Phase 1 read-only + diff UI (Apply stub until API 🟢).

DELIVER:
1. grammar_inspector: show grammar_lineage (parent, iteration_seq, mode) + grammar_overrides (pinned keys bold)
2. Footprint grid diff highlight when snapshot replaced in-session (before/after in memory): added/removed/changed legend
3. Iterate grammar LabelFrame shell per spec wireframe — controls present; Apply calls stub → "Phase 2: grammar-iterate CLI"
4. On load: display optional grammar_lineage / grammar_overrides from snapshot if fields present
5. COMMIT:WIT $ref:debug_runs/grammar_iter_001_aps1_live.json

PARALLEL OK ⟨GRAMMAR-ITER-001-API⟩ — coordinate diff contract.

BLANG:PY · ⏸ keyframe
BLANG:WIT → BLANG:Q✓
```

---

## Paste — @coder-mcp + @coder (GRAMMAR-ITER-001-API / RUST)

```text
⟨GRAMMAR-ITER-001-API⟩ + ⟨GRAMMAR-ITER-001-RUST⟩

@coder-mcp:
  $ref:docs/archive/2026-06-src-dev/plans/grammar_iter_001_spec_v1.md§Phase-2
  $ref:tools/mcp/schemas/grammar_iterate_request_v1.schema.json
  $ref:tools/mcp/python/rust_engine_mcp/building_grammar.py
  COMMIT:WIT $ref:debug_runs/grammar_iter_001_massing_live.json
  BLANG:PY test_grammar_iter

@coder:
  $ref:src/construction/procedural/building_grammar.rs
  BLANG:S5 construction::procedural::building_grammar

🧊 modes roof/facade → $ref:docs/archive/2026-06-src-dev/plans/grammar_002_slice_001_v1.md
⏸ keyframe · parallel ⟨APS-BEVY-PREVIEW-002⟩ ⟨BUILD-WORKER-001⟩ 🟢
```

---

## Paste — @orchestrator (parent)

```text
AUTH: SNAP★ ⇢ grammar iter lane
$ref:docs/archive/2026-06-src-dev/plans/grammar_iter_agent_orders_v1.md
$ref:tools/orchestrator/queues/grammar_continuation_queue.json

ΔWF:
  @designer     ⟨GRAMMAR-ITER-001-UI⟩ 🟢
  @coder-mcp    ⟨GRAMMAR-ITER-001-APS1⟩ · ⟨GRAMMAR-ITER-001-API⟩
  @coder        ⟨GRAMMAR-ITER-001-RUST⟩

COMMIT:WIT $ref:debug_runs/grammar_iter_001_massing_live.json
⏸ Track B — $ref:tools/orchestrator/queues/defer_registry.json
```

---

## Verification (after API)

```powershell
cd C:\dev\github\Rust_engine_template_01\tools\mcp\python
python -m rust_engine_mcp.cli grammar-iterate path/to/request_massing.json --write-snapshot
python -m pytest tools/mcp/python/tests/test_grammar_iter.py -q
cd ..\..
cargo test -p proc_A_dine01 --lib building_grammar -- --nocapture
```

---

## Changelog

| Version | Date | Notes |
|:---|:---|:---|
| v1.0.0 | 2026-06-03 | Orchestrator dispatch after planner-mcp spec |
| v1.1.0 | 2026-06-03 | ⟨AGENT-LANG-002-REF⟩ $ref + ⟨⟩ delta |
