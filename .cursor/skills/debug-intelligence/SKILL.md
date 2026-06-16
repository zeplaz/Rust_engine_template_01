---
name: debug-intelligence
description: >-
  Triage diagnostics into compressed, routed reports instead of fixing in place or
  dumping logs. Use when interpreting witness JSON, viewport/authority drift, render
  contract mismatches, construction placement projection deltas, multi-writer ECS
  resources, schedule hazards, or stale scaffolds. Produces a YAML routing packet for
  @planner / @coder / @designer. Triggers: witness, drift, dual writer, render contract,
  viewport, placement debug, Pick Δ, ghost misalign, diagnostics, panic, regression triage.
disable-model-invocation: true
---

`⟦SYM⟧ lang⊳ $ref:prompts/SYMBOLIC_LANGUAGE.meta.md`

# debug-intelligence — compress evidence, route the fix

## The pattern

A triage agent does **not** fix and does **not** paste raw logs.

```text
◎evidence ◂⊳ smallest sufficient source (witness DIGEST · code REGION · debug probe fields)
  ▷⊳ ▢compress ─⬡[severity · root_cause · affected · migration · conf]▶
  ▷⊳ ◆route ▶ ⦿owner + single next step
  ⛔ log-walls — surface the DECISION not the dump
```

Output = routing packet, not a patch:

```yaml
issue: <one line>
root_cause: <mechanism, not symptom>
affected: [<system/resource>, ...]
migration_status: <tag or n/a>
recommendation: <single next action>
owner: "@planner | @coder | @designer | @orchestrator"
confidence: 0.0–1.0
```

## In this repo (scope)

```text
⊚own  ECS / viewport / render / placement-projection DRIFT
¬own  pipeline DSM / Q-C-E / three-track → operations-intelligence skill
```

**Watch surfaces:**

```text
src/gui/view_authority.rs · view_projection_authority.rs · map_camera.rs
src/render/viewport_pipeline.rs · fire_visual_extract.rs · map_view/
src/construction/placement_debug.rs · map_egui_projection.rs
```

**Detects:** multi-writer resources · hidden authority mutation · camera bleed · scissor/heal mismatch · schedule hazards · stale scaffolds · projection span confusion (`fixed_w` vs `visible_w`)

Authority map: [bevy-simulation-grade/07-repo-authority-map.md](../bevy-simulation-grade/07-repo-authority-map.md)  
Placement projection: [bevy-simulation-grade/09-sim-map-projection-placement.md](../bevy-simulation-grade/09-sim-map-projection-placement.md)

## Construction placement debug (live overlay)

When user reports cursor/ghost misalignment, read **probe fields** first (not perf logs):

| Field | Drift signal |
|-------|----------------|
| `pick_delta_world` | cam vs manual egui inverse — want **< 1** |
| `ghost_delta_camera_vs_egui_px` | draw path — want **< 4** |
| `cursor_reproject_delta_px` | camera self-consistency |
| `latch_using_hole` vs `camera_viewport_phys` | hole latch true but viewport full window → ortho/view_px bug |
| `ortho_fixed_wh` vs `projection_visible_wh` | fixed = view/zoom; visible = manual span |
| `camera_authoritative` | which pick path is live |

**Routing examples:**

```yaml
# span bug
issue: Pick Δ world large while camera roundtrip ok
root_cause: sim_map_*_in_frame used fixed_w/h instead of visible_w/h
owner: "@coder"
confidence: 0.9

# scissor heal bug
issue: Ghost detached, latch_hole=true, viewport full window
root_cause: view_px sized from sim hole while GPU draws window_px
owner: "@coder"
confidence: 0.85
```

Enable: `--test vfx` / `--test visual` or `CONSTRUCTION_PLACEMENT_DEBUG=1`.

## Evidence commands

Prefer validation-first and witness briefs over raw cargo:

```bash
cargo validate-report cargo --cached
# witness: debug_runs/unified_witness_index.json · construction_stage_live.json
```

Escalate to `raw_log_path` only when `confidence < 0.7` or validator status failed with empty errors.

## Gotchas

- Lone ✅ is not a verdict — close with 🧪 measured, 📜 witnessed, or ⊚ authority-valid
- Do not read full witness JSON when a digest or probe struct suffices
- Stale binary after `map_camera.rs` edit — confirm file non-empty and rebuild before deep triage

## Source

Full detection rules: [reference.md](reference.md)
