# PLAN-MULTI-PARALLEL-TRACKS-001 — Wave 0 dispatch orders `v1`

```text
⟨PLAN-MULTI-PARALLEL-TRACKS-001⟩  🟡 ACTIVE  wave=0  issued=2026-06-20
AUTH: MAT★ ⇢ APS★ ⇢ SNAP★ ⇢ WRK★ ⇢ ATL★ ⇢ RT★
Board: $ref:tools/orchestrator/queues/multi_parallel_tracks_dispatch_v1.json
Plan:  $ref:src/dev/plan_multi_parallel_tracks_v1.md
Gate:  BLANG:WIT-HON before every BLANG:Q✓ · dual Q✓ dispatch + home queue
```

---

## Orchestrator action

**No global primary.** Issue wave-0 to **all owner lanes in parallel**. Each agent filters `owner=<me> status=ready wave=0` and picks **one row per session** (or parallel spec rows for @designer).

When blocked on track A → **cross-drain** to track B (same owner) — do not idle.

---

## @designer — T1 wave-0 (four specs in parallel)

```text
PICK ALL wave-0 (any order):
  DES-APS-INTERACTION-001      → src/dev/design_aps_interaction_v1.md
  DES-APS-ONBOARD-SPEC-002     → src/dev/design_aps_onboard_spec_v2.md
  DES-APS-PREVIEW-LADDER-001   → src/dev/design_aps_preview_ladder_v1.md
  DES-APS-MANUAL-FALLBACK-001  → src/dev/design_aps_manual_fallback_v1.md
ALSO ready: DES-POWER-NODE-HOVER-001 (T6) if APS specs waiting on review
Q✓: designer_active_queue.json + dispatch row
```

---

## @designer-mcp — T3 P0

```text
PRIMARY: DMCP-VEG-ATLAS-SHIP-001
  Deliverable: src/dev/dmcp_veg_atlas_ship_v1.md
  Unblocks: VEG-F01 · VEG-F02 · honest ship path
ON-CALL: MCP-PILOT-GRAMMAR-001 when operator clears LOCK-G4-OPERATOR
```

---

## @coder-mcp — pick ONE territory per session

```text
Option A (T1 — LOCK-APP-PY):
  OVR-P5-TAIL-001
  VERIFY: pytest tools/mcp/python/tests -k aps -q

Option B (T3 — no app.py):
  APS-EVO-E4-ATLAS-EXPAND-001
  VEG-CATALOG-BURN-ROWS-001
  DO NOT: ship:true (G0 rules)

Option C (T2 — CLI only):
  CMCP-GRAMMAR-FACILITY-BRIEF-001
  CMCP-SITE-ZONE-VALIDATE-001
  CMCP-GRAM-SWEEP-PROCESS-001
  GRAM-CONTENT-005
```

---

## @coder — T5 + T2 + T6 + T4 (cross-drain)

```text
wave-0 parallel picks (one per session):
  COD-SIM-HUD-EGUI-THEME-001      (T5)
  COD-SIM-HUD-BUILD-PICKER-001    (T5)
  COD-SIM-HUD-TRAY-BUILD-001      (T5)
  CODER-PILOT-REFACTOR-001        (T2)
wave-1+ when ready:
  COD-POWER-ISLAND-HIGHLIGHT-001  (T6)
  COD-UTILITY-ACTIVATION-LINK-001 (T6)
REGRESSION: cargo test -p proc_A_dine01 --lib construction stage5
```

---

## @coder A — T4 + T3

```text
wave-0:
  FIRE-F2-FUEL-SPREAD-001
  WSS-SMOKE-BRIDGE-001
  FIRE-F2-READINESS-ALIGN-001
wave-1:
  CDR-A-VISUAL-SMOKE-ECO-001  (LOCK-WITNESS-STAGE5 — solo writer)
  VEG-F01-ATLAS-SHIP-001      (after DMCP-VEG-ATLAS-SHIP)
  VM-10-MINIMAP-LOCKSTEP      (T8)
```

---

## @coder B — T5 + T6 + T3 + T8

```text
wave-0 parallel:
  COD-SIM-HUD-POPUP-MIGRATE-001     (T5)
  COD-POWER-OVERLAY-RENDER-001      (T6)
  CDR-B-VEG-MINIMAP-LEGEND-UI-001   (T3)
  TRIAGE-PERF-SHELL                 (T8)
```

---

## @operator — T7 (rollup leverage)

```text
ORDER:
  1. G-PLAY-01
  2. G-PLAY-OPERATOR-01   ← unblocks veg honest close
  3. PERF-SHELL-001
  4. OPS-VT5-OPERATOR-001 (T4 visual confirm)
NEEDS-DISPLAY — no agent self-certifies pixels
```

---

## @sim-steward — T8 + T4 close

```text
PICK when routing or acting:
  VM-09-V2-INVERT-BRIDGE (triage → @coder)
  SIM-STEWARD-FIRE-REGRESS-001 (after fuel spread)
  VEG-SHIP-CLOSE-001 witness honesty audit
```

---

## @orchestrator-mcp

```text
DO NOT issue single-lane wave.
Track close only when wave-2 rows ready:
  APS-STUDIO-CLOSE-001 · GRAMMAR-SHIP-CLOSE-001 · VEG-SHIP-CLOSE-001
```

---

## Cross-drain quick reference

| If blocked on… | Same owner picks… |
|:---|:---|
| Power overlay (T6) | Sim HUD popup (T5) or minimap legend (T3) |
| OVR-P5 app.py lock | E4 landscape batch (T3) or CMCP-GRAM tools (T2) |
| G4 warehouse paused (T2) | APS specs (T1) or veg atlas sign-off (T3) |
| G-PLAY waiting (operator) | Coders continue all non-operator tracks |

```text
⟦wave-0⟧  ΔWF→ ALL agents: filter owner · wave=0 · parallel_ok default · WIT-HON · dual Q✓
```
