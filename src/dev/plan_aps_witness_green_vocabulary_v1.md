# PLAN-APS-WITNESS-GREEN-VOCABULARY-001 — three greens scope vocabulary `v1`

```text
⟦SYMLANG⟧⟐v1  ◈PLAN
⟨ID⟩ PLAN-THREE-GREENS-VOCAB-001
Date: 2026-06-16
Status: **SIGNED** (@planner-mcp)
Parent: $ref:src/dev/plan_aps_evolution_veg_capability_20260616_v1.md § Three greens
```

**Problem:** Witnesses and queue rows use **green** without scope — operators confuse schema sign-off with art-ship.

---

## Vocabulary (mandatory in new witnesses)

| Scope ID | Label | Certifies | Does NOT certify |
|:---|:---|:---|:---|
| **G-SCHEMA** | schema/sign green | JSON schema validate, planner sign | runtime, pixels, operator play |
| **G-BAKE** | bake green (G0–G3) | PNGs + atlas meta + honest bake gate | G4 ship, G5 registry production |
| **G-RUNTIME** | runtime-stamp green | ECS extract + registry/chunk stamp wired | art-ship, operator checklist |
| **G-ART-SHIP** | art-ship green (G4/G5) | Artist re-verdict + `ship: true` | — |
| **G-OPERATOR** | operator green | Human session checklist, no `--test` | lib harness alone |

---

## Witness header (required `_agent_meta` or top-level)

```json
{
  "green_scope": "G-RUNTIME",
  "green": true,
  "proof_grade": "live_sim | lib_harness | schema_only"
}
```

**BLANG rule:** `green: true` + `green_scope: G-OPERATOR` forbidden when `proof_grade: lib_harness`.

---

## APS program mapping

| Witness | Scope | Current honest grade |
|:---|:---|:---|
| `mcp_landscape_grammar_sign_live.json` | G-SCHEMA | schema_only |
| `tile_tile_landscape_lg5_pilot_v1_live.json` | G-BAKE | bake (pilot) |
| `landscape_grammar_lg5_live.json` | G-RUNTIME | runtime-stamp (ship:false) |
| `design_aps_artist_ship_review_*` | G-ART-SHIP | pending E0 re-verdict |
| `plan_g_play_close_001_checklist` | G-OPERATOR | OPEN |

---

## Sign-off

| Role | Date | Action |
|:---|:---|:---|
| **@planner-mcp** | 2026-06-16 | **SIGNED** |

```text
⟦/PLAN-THREE-GREENS-VOCAB-001⟧  use in all APS-E* witnesses going forward
```
