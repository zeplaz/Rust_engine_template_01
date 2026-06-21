# APS manual fallback deprecation `v1` — footprint lane banner

| Field | Value |
|:---|:---|
| **ID** | **DES-APS-MANUAL-FALLBACK-001** |
| **Program** | PLAN-MULTI-PARALLEL-TRACKS-001 · T1 |
| **Date** | 2026-06-20 |
| **Owner** | `@designer` |
| **Depends** | [`design_aps_assembly_workflow_realign_v1.md`](design_aps_assembly_workflow_realign_v1.md) · [`design_aps_grammar_tier_exposure_v1.md`](design_aps_grammar_tier_exposure_v1.md) |
| **Handoff** | assembly_panel.py · Setup collapse section |
| **Verdict** | **PASS** |

```text
DES-APS-MANUAL-FALLBACK-001 Q✓
Manual style/footprint lane — collapsed default · deprecation banner at G3+
```

---

## 0. Scope

**Setup & manual fallback** collapse — style pack picker + manual footprint grid beside grammar generate path.

**Not removed in v1** — deprecated with clear copy as grammar set matures.

---

## 1. Placement

```text
┌ Generate (primary) ─────────────────────────┐
│ Type · District · [Generate]                │
└─────────────────────────────────────────────┘
┌ ▸ Setup & manual fallback (collapsed) ──────┐  ← default collapsed all tiers
└─────────────────────────────────────────────┘
```

---

## 2. Banner rules

| Tier | Banner inside collapse header |
|:---|:---|
| G0–G2 | *(none)* — subtitle: `Advanced — use when generate path insufficient` |
| **G3** | `◆ Grammar covers most styles — manual fallback for edge cases only` |
| **G4** | `⚠ Manual footprint deprecated — prefer generate + tweak layer` |

Banner uses `warn` token @ 15% wash — not modal.

---

## 3. Expanded body (when opened)

| Control | Label | Note |
|:---|:---|:---|
| Style pack combo | `Manual style pack` | Engineer ids hidden — human labels |
| Footprint grid | `Manual footprint` | Same grid widget as today |
| Apply | `Apply manual layout` | Does not auto-run ship |

**G4:** `Apply manual layout` shows confirm: `Manual layout bypasses grammar tags — continue?`

---

## 4. Interaction

| Rule | Spec |
|:---|:---|
| Default | Collapsed |
| Open state | Persist per session in `aps_ui_prefs.manual_fallback_open` |
| Generate success | Does not auto-collapse if artist had it open |
| Tier drop G4→G3 | Banner text updates — no data loss |

---

## 5. Acceptance

| # | Check |
|:---:|:---|
| M1 | Collapsed on first open all tiers |
| M2 | G4 banner verbatim |
| M3 | Primary generate row never inside collapse |
| M4 | No engineer schema keys in banner |

---

## Sign-off

| Role | Verdict | Date |
|:---|:---|:---|
| `@designer` | **PASS** | 2026-06-20 |
