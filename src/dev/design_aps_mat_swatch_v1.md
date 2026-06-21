# APS materials swatch grid `v1` — glyph + word status

| Field | Value |
|:---|:---|
| **ID** | **DES-APS-MAT-SWATCH-001** |
| **Program** | PLAN-DESIGNER-WORK-202606-001 · Track B3 |
| **Date** | 2026-06-20 |
| **Owner** | `@designer` |
| **Depends** | [`design_aps_mat_browse_v1.md`](design_aps_mat_browse_v1.md) · [`design_aps_color_a11y_audit_v1.md`](design_aps_color_a11y_audit_v1.md) |
| **Verdict** | **PASS** |

```text
DES-APS-MAT-SWATCH-001 Q✓
Swatch grid — status glyph+word · not color-alone
```

---

## 1. Grid cell

```text
┌────────────┐
│  [thumb]   │
│ Brick red  │
│ ● Assigned │
└────────────┘
```

| Status | Glyph | Word |
|:---|:---:|:---|
| Assigned | `●` | `Assigned` |
| Draft | `○` | `Draft` |
| Missing ref | `✗` | `No texture` |
| Unsorted | `?` | `Unsorted` |

---

## 2. Rules

| Rule | Spec |
|:---|:---|
| Selection | gold border 2px |
| Hover | caption shows profile id (mono) |
| A11y | status always word + glyph |
| Grid | min cell 96px · max 8 columns |

---

## Sign-off

| Role | Verdict | Date |
|:---|:---|:---|
| `@designer` | **PASS** | 2026-06-20 |
