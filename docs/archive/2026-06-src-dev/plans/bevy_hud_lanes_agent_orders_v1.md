# Lane A′ + Product HUD — Agent orders (orchestrator dispatch) `v1`

| Field | Value |
|:---|:---|
| **Program** | PLAN-BEVY-HUD-GRAMMAR-PARALLEL-001 (lanes 4–5) |
| **Lang** | [`agent_lang_v1.md`](agent_lang_v1.md) — `⟨ID⟩` · `$ref:` · BLANG |
| **Parent** | $ref:docs/archive/2026-06-src-dev/plans/plan_bevy_hud_grammar_parallel_v1.md |
| **Owners** | **@designer + @coder** — **not** @designer-mcp · **not** @coder-mcp · **not** Tk APS |
| **Queue** | $ref:tools/orchestrator/queues/grammar_continuation_queue.json |
| **Blocked by warehouse keyframe?** | **No** — ⏸ $ref:tools/orchestrator/queues/defer_registry.json |

---

## Two surfaces — do not merge

| # | ⟨ID⟩ | UI system | Code home | Agent |
|:---:|:---|:---|:---|:---|
| **4** | ⟨APS-BEVY-QC-HUD-001⟩ | **egui** dev tooling | $ref:src/gui/assembly_snapshot_qc_ui.rs | @designer + @coder |
| **5** | ⟨SIM-HUD-PRODUCT-001⟩ | **Bevy native** player chrome | in_game_hud · simulation_session | @designer + @coder |

**Boundary:** $ref:prompts/guides/ui_boundary_guide_v1.md

---

## Lane A′ — APS-BEVY-QC-HUD-001

### v1 status (@coder — **shipped**)

| Item | Detail |
|:---|:---|
| Panel | `assembly_snapshot_qc_ui.rs` — path load, summary, placement table, spawn preview hint |
| Toggle | **Ctrl+Shift+Q** |
| Diagnostics | F3 → Assembly snapshot QC |
| Witness | $ref:debug_runs/aps_bevy_qc_hud_001_live.json — 🟢 |
| Tests | `cargo test -p proc_A_dine01 --lib aps_bevy_qc_hud` |

### Remaining work (parallel)

| ⟨ID⟩ | Agent | Task | φ |
|:---|:---|:---|:---:|
| ⟨APS-BEVY-QC-HUD-001-DESIGN⟩ | @designer | UX sign-off | 🟢 $ref:docs/archive/2026-06-src-dev/plans/design_aps_bevy_qc_hud_v1.md |
| ⟨APS-BEVY-QC-HUD-001-V2⟩ | @coder | row select + P0 read-only strip | 🟢 $ref:debug_runs/aps_bevy_qc_hud_001_v2_live.json |

---

## Lane Product — SIM-HUD-PRODUCT-001

**Mission:** PLAY-01 simulation chrome — collapsed editor, readable ops strip, dock/minimap/build rail.

**Brief:** $ref:prompts/designer_questions/sim_hud_product_brief_v1.md

### Sliced delivery (one slice per cycle)

| ⟨ID⟩ | Agent | Slice | COMMIT:WIT / spec |
|:---|:---|:---|:---|
| ⟨SIM-HUD-SLICE-PLAY01⟩ | @coder | Session entry/exit | $ref:debug_runs/sim_hud_play01_live.json |
| ⟨SIM-HUD-SLICE-OPS⟩ | @designer → @coder | Ops strip | $ref:docs/archive/2026-06-src-dev/plans/design_sim_hud_ops_v1.md 🟡 |
| ⟨SIM-HUD-SLICE-DOCK⟩ | @designer → @coder | Command tray | $ref:docs/archive/2026-06-src-dev/plans/design_sim_hud_dock_v1.md 🟡 |
| ⟨SIM-HUD-SLICE-MINIMAP⟩ | @designer → @coder | Minimap overlays | $ref:docs/archive/2026-06-src-dev/plans/design_sim_hud_minimap_v1.md 🟡 |
| ⟨SIM-HUD-SLICE-BUILD⟩ | @designer → @coder | Build rail | $ref:docs/archive/2026-06-src-dev/plans/design_sim_hud_build_v1.md 🟡 |

**Program close:** ⟨SIM-HUD-PRODUCT-001⟩ 🟢 — $ref:debug_runs/sim_hud_product_close_001_live.json · Phase 3 **SIM-HUD-PRODUCT-CLOSE-001** done.

---

## Paste — @designer

```text
@designer · Chain G · NOT Tk APS
$ref:docs/archive/2026-06-src-dev/plans/bevy_hud_lanes_agent_orders_v1.md

⟨APS-BEVY-QC-HUD-001-DESIGN⟩ 🟢
$ref:prompts/designer_questions/aps_bevy_qc_hud_brief_v1.md
- Review shipped panel (Ctrl+Shift+Q in sim/editor)
- Deliver: 1-page wireframe/sign-off OR PASS WITH NOTES on column layout, empty states, spawn-preview copy
- Paste back per brief § Paste back

TASK B (primary): SIM-HUD-PRODUCT-001 — pick ONE slice this session:
- SIM-HUD-SLICE-OPS | DOCK | MINIMAP | BUILD (see orders doc)
- Write src/dev/design_sim_hud_<slice>_v1.md with before/after + PLAY-01 checklist
- Hand off acceptance criteria to @coder — do not implement Rust

OUT OF SCOPE: MCP batches, warehouse keyframe, Tk APS, src/construction/ edits
```

---

## Paste — @coder

```text
BLANG:Q+("coder") · src/gui/ · NOT coder-mcp
$ref:docs/archive/2026-06-src-dev/plans/bevy_hud_lanes_agent_orders_v1.md
$ref:prompts/guides/ui_boundary_guide_v1.md

⟨APS-BEVY-QC-HUD-001-V2⟩ optional:
$ref:src/gui/assembly_snapshot_qc_ui.rs
- Row select highlights footprint cell when preview spawn active
- Optional: read-only strip from validate-report assembly_p0 (text only, no MCP in hot path)

ΔWF product HUD (one slice per cycle):
  1. ⟨SIM-HUD-SLICE-PLAY01⟩  → $ref:debug_runs/sim_hud_play01_live.json
  2. ⟨SIM-HUD-SLICE-DOCK⟩     → $ref:debug_runs/sim_hud_slice_dock_live.json
  3. ⟨SIM-HUD-SLICE-OPS⟩      → $ref:debug_runs/sim_hud_slice_ops_live.json
  4. ⟨SIM-HUD-SLICE-MINIMAP⟩  → $ref:debug_runs/sim_hud_slice_minimap_live.json
  5. ⟨SIM-HUD-SLICE-BUILD⟩    → $ref:debug_runs/sim_hud_slice_build_live.json

Specs: $ref:docs/archive/2026-06-src-dev/plans/design_sim_hud_ops_v1.md · dock · minimap · build

RULES:
- egui QC panel = dev tooling (Assembly snapshot QC)
- Player HUD = Bevy native UI (in_game_hud) — do not merge QC table into product shell
- No warehouse keyframe / tile ship work

VALIDATION: cargo test -p proc_A_dine01 --lib gui::assembly_snapshot_qc_ui simulation_session (as applicable)
Attach bevy-simulation-grade for viewport/session authority if touching ViewManager gates
```

---

## Paste — @orchestrator

```text
Chain G — $ref:src/dev/master_chain_board_4d_v1.md§1
$ref:docs/archive/2026-06-src-dev/plans/bevy_hud_lanes_agent_orders_v1.md

Lane A′: ⟨APS-BEVY-QC-HUD-001⟩ 🟢 · ⟨APS-BEVY-QC-HUD-001-V2⟩ ○
Lane Product: ⟨SIM-HUD-PRODUCT-001⟩ 🟡 — PLAY01 → DOCK → OPS → MINIMAP → BUILD
$ref:prompts/designer_questions/sim_hud_product_brief_v1.md

NOT: @coder-mcp · @designer-mcp · ⏸ keyframe
```

---

## Verification

```powershell
cd C:\dev\github\Rust_engine_template_01

# Lane A′ witness (regression)
cargo test -p proc_A_dine01 aps_bevy_qc_hud --lib

# Manual: run app → Simulation or Editor → Ctrl+Shift+Q → Load example warehouse snapshot

# PLAY-01 (after SIM-HUD-SLICE-PLAY01)
cargo test -p proc_A_dine01 simulation_session --lib
```

---

## Changelog

| Version | Date | Notes |
|:---|:---|:---|
| v1.0.0 | 2026-06-03 | Orchestrator dispatch — QC v1 done, SIM-HUD sliced |
| v1.1.0 | 2026-06-03 | ⟨AGENT-LANG-002-REF⟩ $ref + ⟨⟩ delta |
