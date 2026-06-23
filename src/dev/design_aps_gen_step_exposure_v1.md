# APS generation step exposure + artist approve `v1`

| Field | Value |
|:---|:---|
| **ID** | **DES-APS-GEN-STEP-EXPOSURE-001** |
| **Date** | 2026-06-02 |
| **Owner** | `@designer` |
| **Implements** | `generation_trace_strip.py` on Assembly + Variants |
| **Verdict** | **PASS (qualified)** — trace strip shipped; full step editor deferred |

```text
DES-APS-GEN-STEP-EXPOSURE-001 Q✓
Generation trace + approve checkbox — grammar steps visible, not hidden in inspector only
```

---

## Problem

Grammar **Generate** runs on Assembly but artists on **Variants** could not see what produced the parent snapshot or sign off before baking tile states.

## Solution — Generation trace strip

Collapsible-adjacent strip (always visible on Assembly + Variants):

```text
┌─ Generation trace ─────────────────────────────────────────┐
│ Industrial warehouse · Industrial west · seed 39 · id…   │
│ ✓ Footprint / grammar massing                            │
│ ✓ Module resolve (N placements)                          │
│ ✓ Rule chain (M chain tags)                              │
│ ○ P0 ship check not run                                  │
│ [☐ Approve snapshot for variant / bake parent]             │
│ [Edit on Assembly]                                       │
└──────────────────────────────────────────────────────────┘
```

| Control | Behavior |
|:---|:---|
| **Summary line** | Human labels from snapshot archetype · district · seed · assembly_id |
| **Step rows** | Read-only checks from snapshot + `assembly_p0_passed` |
| **Approve** | Sets `SuiteState.assembly_generation_approved` — artist gate before variant bake (future hard gate optional) |
| **Edit on Assembly** | Switches notebook to Assembly tab |
| **Reset approve** | Clears on generate / load snapshot |

## Preview window groups (workflow alignment)

| Group | Tabs | Role |
|:---|:---|:---|
| **Piece** | Assembly inspector | Slot thumbs |
| **Whole** | Assembly preview | Snapshot 3D |
| **State** | Variants preview | Draft layer merge + 4-state strip |
| **Ship** | Atlas preview | UV / cells |

Variants group now includes **generation trace** above **variant preview** above **layer editor**.

## Deferred (qualified)

| Item | Why |
|:---|:---|
| Inline grammar step edit | Use Assembly iterate / DNA panels — not duplicated on Variants |
| Hard block bake without approve | Wire in `variant_bake` after operator sign-off |

## Acceptance

| # | Check |
|:---:|:---|
| G1 | Trace visible on Assembly after Generate |
| G2 | Trace visible on Variants when snapshot loaded |
| G3 | Approve resets on new Generate / Load |
| G4 | Edit on Assembly switches tab |

## Exit predicate

Strip renders with example warehouse snapshot · approve toggles `state.assembly_generation_approved`.

---

## Sign-off

| Role | Verdict | Date |
|:---|:---|:---|
| `@designer` | **PASS (qualified)** | 2026-06-02 |
