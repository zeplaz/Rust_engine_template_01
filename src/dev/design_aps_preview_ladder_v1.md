# APS preview fidelity ladder `v1` — G0→G4 × P0→P4

| Field | Value |
|:---|:---|
| **ID** | **DES-APS-PREVIEW-LADDER-001** |
| **Program** | PLAN-MULTI-PARALLEL-TRACKS-001 · T1 · grammar evolution |
| **Date** | 2026-06-20 |
| **Owner** | `@designer` |
| **Authority** | [`plan_aps_grammar_evolution_v1.md`](plan_aps_grammar_evolution_v1.md) · [`design_aps_preview_v2_spec_v1.md`](design_aps_preview_v2_spec_v1.md) |
| **Handoff** | OVR-P55-PREVIEW-002 · assembly_preview worker |
| **Verdict** | **PASS** |

```text
DES-APS-PREVIEW-LADDER-001 Q✓
Grammar tier G0–G4 gates preview levels P0–P4
```

---

## 0. Two axes

| Axis | Source | Purpose |
|:---|:---|:---|
| **Grammar tier** | `grammar_set_tier()` | Content maturity G0–G4 |
| **Preview level** | Panel renderer | Fidelity P0–P4 |

Preview level **never exceeds** tier cap.

---

## 1. Preview levels (P0–P4)

| Level | ID | Artist sees | Min tier |
|:---|:---|:---|:---:|
| **P0** | `PREVIEW-FOOTPRINT` | Token heatmap on footprint grid | G0 |
| **P1** | `PREVIEW-SLOT` | Isolated module + material thumb | G0 |
| **P2** | `PREVIEW-ASSEMBLY` | Combined assembly 2×2 strip | G0 |
| **P3** | `PREVIEW-GRAMMAR` | Inspector row → highlight placements + why tooltip | G2 |
| **P4** | `PREVIEW-SHIP` | Keyframe / ship render chip | G4 |

**Four visual states** (clean/night/damaged/burning) apply at **P1+** per preview v2 spec.

---

## 2. Tier unlock matrix

| Tier | Unlocked preview | Locked surfaces |
|:---|:---|:---|
| **G0** | P0–P2 (partial P2) | P3 hidden · P4 hidden |
| **G1** | P0–P2 full | P3 collapsed teaser |
| **G2** | P0–P3 | P4 banner only |
| **G3** | P0–P3 + iterate diff overlay default | P4 queued |
| **G4** | P0–P4 | — |

**Teaser copy when locked:** `Preview unlocks at G{n} — {bar from grammar_set_brief}`.

---

## 3. Surface mapping

| APS surface | Default level | State strip |
|:---|:---|:---:|
| Catalog module thumb | P1 | Optional (catalog browse) |
| Assembly slot row | P1 + P2 combined | **Required** |
| Variants panel | P2 | Required |
| Atlas pre-bake | P2 | Clean only |
| Ship check modal | P4 | Clean + night |

---

## 4. Async + debounce (all levels)

| Rule | Value |
|:---|:---|
| Select → render debounce | 300ms ([`design_aps_smoothness_charter_v1.md`](design_aps_smoothness_charter_v1.md)) |
| Stale job | Cancel on new select |
| Loading | `⟳ Rendering…` in preview pane — all thumbs |

---

## 5. Acceptance

| # | Check |
|:---:|:---|
| L1 | G0 never shows P3 row highlight |
| L2 | P4 chip only when tier G4 green in witness |
| L3 | State strip works at P1+ |
| L4 | Locked teaser names tier in human words |

---

## Sign-off

| Role | Verdict | Date |
|:---|:---|:---|
| `@designer` | **PASS** | 2026-06-20 |
