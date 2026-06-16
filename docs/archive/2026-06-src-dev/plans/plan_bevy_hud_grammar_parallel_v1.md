# PLAN-BEVY-HUD-GRAMMAR-PARALLEL-001 — Three Bevy surfaces + grammar iteration `v1`

| Field | Value |
|:---|:---|
| **Program ID** | **PLAN-BEVY-HUD-GRAMMAR-PARALLEL-001** |
| **Parent** | [`plan_three_track_execution_v1.md`](plan_three_track_execution_v1.md) |
| **Status** | **ACTIVE** |
| **Date** | 2026-06-03 |

---

## Planner decision (locked)

| Decision | Meaning |
|:---|:---|
| **Warehouse Track B keyframe / G4** | **PAUSED** — does **not** block other lanes ([`pilot_grammar_operator_runbook_v1.md`](pilot_grammar_operator_runbook_v1.md) remains for when art is ready) |
| **Broken ship art** | Gates **only** warehouse sign-off + production tile registry — **not** Bevy preview, Tk APS, grammar engine, or simulation HUD |
| **Three Bevy “HUD” surfaces** | **All three** are legitimate parallel work — they are **different products**, not one queue item |
| **Grammar rewrite** | Track C runs **parallel**; **after** P0–P2 APS preview/material authority unless planner reprioritizes |
| **Grammar UX** | New goal: **iterate** on semi-decent buildings (sliders / layer toggles / local overrides) — **not** seed-only reroll |

---

## Three Bevy surfaces (stop conflating)

| # | ID | What it is | Where | Owner | Blocked by broken art? | Status |
|:---:|:---|:---|:---|:---|:---:|:---|
| **1** | **APS-BEVY-PREVIEW-001** | Artist tool — load assembly snapshot, spawn GLBs in Bevy worker; APS “Preview assembly” | `bevy_preview_worker`, `assembly_preview.py`, APS Assembly tab | @coder-mcp · @coder | **No** | **shipped** — polish row open |
| **2** | **APS-BEVY-QC-HUD-001** | Optional **in-sim egui** QC: snapshot path, placements, `material_profile` | `src/gui/` egui tooling | @designer · @coder | **No** | **v1 done** — sign-off + v2 polish open |
| **3** | **SIM-HUD-PRODUCT-001** | **Player-facing** chrome — ops strip, build rail, minimap, PLAY-01 | Bevy native UI | @designer · @coder | **No** | **ready** — sliced in [`bevy_hud_lanes_agent_orders_v1.md`](bevy_hud_lanes_agent_orders_v1.md) |

**Wording fix:** “In-engine HUD / multiview — not APS” means **do not implement simulation HUD inside Tkinter**. It does **not** mean “never build Bevy HUD.”

---

## What broken art gates vs does not

| Question | Gated by bad warehouse PNGs? |
|:---|:---:|
| Does `bevy_preview_worker` load this snapshot? | No |
| Do Tk slot / material / atlas previews work? | No |
| Can egui QC show placements + material_profile? | No |
| Can grammar massing improve future buildings? | No |
| Does warehouse get `proceed_ship: yes`? | **Yes** |
| Does map show production warehouse atlas? | **Yes** |

---

## Parallel orchestrator assignment (now)

```text
Lane A  (APS Tk)       P0 BUILD-WORKER · P2 APS-MAT-002 · catalog/atlas UX     @coder-mcp · @designer
Lane A′ (Bevy tool)    APS-BEVY-PREVIEW polish · APS-BEVY-QC-HUD-001         @coder · @designer
Lane C  (grammar)      GRAMMAR-001/002 engine · GRAMMAR-ITER-001 UX           @coder · @planner-mcp
Lane product           SIM-HUD-PRODUCT-001 PLAY-01 / dock                     @designer · @coder
Lane B  (warehouse)    MCP-PILOT-GRAMMAR-001 keyframe                           PAUSED — @designer-mcp on-call
```

---

## GRAMMAR-ITER-001 — Iterative authoring

**Authoritative spec:** [`grammar_iter_001_spec_v1.md`](grammar_iter_001_spec_v1.md) (@planner-mcp)  
**Schema:** `tools/mcp/schemas/grammar_iterate_request_v1.schema.json`  
**Labels:** `assets/configs/buildings/grammars/grammar_labels_v1.json`  
**Designer brief:** [`grammar_iter_wireframe_brief_v1.md`](../../prompts/designer_questions/grammar_iter_wireframe_brief_v1.md)

Summary: layer-scoped iteration (massing → facade → roof → materials) with snapshot lineage, APS Iterate panel, and `iterate_grammar` API — **not** seed-only reroll. Does not wait on warehouse keyframe.

---

## Queue rows (orchestrator)

| ID | Agent | Priority | Status |
|:---|:---|:---:|:---|
| **WH-TRACK-B-PAUSE** | @orchestrator | — | **paused** — warehouse keyframe/G4 until artist QC + BUILD-WORKER green |
| **APS-BEVY-PREVIEW-002** | @coder-mcp | P1′ | ready — context thumb from assembly PNG; worker stability |
| **APS-BEVY-QC-HUD-001** | @designer + @coder | P1′ | **v1 done** — DESIGN sign-off + V2 polish |
| **SIM-HUD-PRODUCT-001** | @designer + @coder | product | **ready** — [`bevy_hud_lanes_agent_orders_v1.md`](bevy_hud_lanes_agent_orders_v1.md) |
| **SIM-HUD-PRODUCT-001** | @designer + @coder | product | ongoing — PLAY-01 / dock (not APS) |
| **GRAMMAR-ITER-001** | @designer + @coder-mcp + @coder | P4 | **SPEC done** — [`grammar_iter_agent_orders_v1.md`](grammar_iter_agent_orders_v1.md) |
| **GRAMMAR-001** | @coder | P4 | ready — massing maturity |
| **GRAMMAR-002** | @coder | P5 | blocked on GRAMMAR-001 slice plan |

---

## Paste — @orchestrator

```text
Parallel programs — warehouse Track B PAUSED (WH-TRACK-B-PAUSE).

Assign now (no keyframe wait):
- Lane A: plan_aps_artist_tool_exec_v1.md P0 BUILD-WORKER + P2 APS-MAT-002
- Lane A′: plan_bevy_hud_grammar_parallel_v1.md — APS-BEVY-PREVIEW-002 + APS-BEVY-QC-HUD-001
- Lane C: GRAMMAR-001/002 + GRAMMAR-ITER-001 (iterative sliders/live preview — not seed-only)
- Lane product: SIM-HUD-PRODUCT-001 (in_game_hud / PLAY-01)

Broken warehouse art gates ONLY proceed_ship + registry — not Bevy preview, Tk APS, or grammar engine work.
```

---

## Paste — @designer (Bevy QC + product HUD)

```text
Two Bevy UI lanes — do not merge:

1) APS-BEVY-QC-HUD-001 — egui dev tool: load assembly_snapshot path, list placements (module_id, material_profile, tags), optional spawn preview entities. Read-only QC; ui_boundary_guide_v1.md (egui for dev tooling).

2) SIM-HUD-PRODUCT-001 — player simulation chrome (in_game_hud.rs, dock, PLAY-01). Product UX — not MCP art pipeline.

Grammar iteration input: GRAMMAR-ITER-001 — which sliders/toggles artists need for massing/facade/roof without full seed reroll.
```

---

## Paste — @coder / @coder-mcp

```text
Lane A′ (Bevy artist tools):
- APS-BEVY-PREVIEW-002: polish bevy_preview_worker; pipe assembly preview PNG into APS context thumb (APS-PREVIEW-001 follow-up).
- APS-BEVY-QC-HUD-001: egui panel in editor/sim — snapshot path, placement table, material_profile display; optional link to preview worker spawn.

Lane C (after P0–P2 unless reprioritized):
- GRAMMAR-001/002 per plan_building_grammar_evolution_v1.md
- GRAMMAR-ITER-001: partial regen API + APS live refresh (snapshot authority preserved)

Do NOT block on warehouse keyframe (WH-TRACK-B-PAUSE).
```

---

## References

- [`plan_aps_artist_tool_exec_v1.md`](plan_aps_artist_tool_exec_v1.md) § Parallel lanes  
- [`aps_preview_004_bevy_worker_v1.md`](aps_preview_004_bevy_worker_v1.md)  
- [`pilot_grammar_operator_runbook_v1.md`](pilot_grammar_operator_runbook_v1.md) (when Track B resumes)  
- [`plan_building_grammar_evolution_v1.md`](plan_building_grammar_evolution_v1.md)

---

## Changelog

| Version | Date | Notes |
|:---|:---|:---|
| v1.0.0 | 2026-06-03 | Three Bevy lanes + GRAMMAR-ITER + warehouse pause |
