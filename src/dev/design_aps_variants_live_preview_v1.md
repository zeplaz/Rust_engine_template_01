# APS Variants — live draft preview + context lines `v1`

| Field | Value |
|:---|:---|
| **ID** | **DES-APS-VARIANTS-LIVE-PREVIEW-001** |
| **Date** | 2026-06-02 |
| **Owner** | `@designer` |
| **Implements** | `APS-VAR-LAYER-WIRE-001` (coder-mcp) |
| **Verdict** | **PASS** — shipped in `variants_panel.py` + `variants_layer_context.py` |

```text
DES-APS-VARIANTS-LIVE-PREVIEW-001 Q✓
Layer dropdowns drive debounced preview before Apply — draft strip when form ≠ row
```

---

## Problem

Artists changed Lighting / Damage / Fill comboboxes but preview only updated after **Apply layers to selected**, making controls feel disconnected from tags and thumbs.

## Solution

| Behavior | Rule |
|:---|:---|
| **Live preview** | Any layer control change → debounced preview using **draft merge** into selected row |
| **Context line** | One plain-language sentence under preview header — focus follows last control |
| **Draft strip** | `Draft — not saved on row. Apply layers to commit.` when form ≠ saved row |
| **Apply** | Unchanged semantics — commits row for Save / tile batch |

## Control → context (canonical)

See [`variants_layer_context.py`](../../tools/mcp/art_pipeline_suite/variants_layer_context.py) hint tables.

## Acceptance

| # | Check |
|:---:|:---|
| V1 | Change lighting combobox → preview refreshes without Apply |
| V2 | Draft strip visible until Apply |
| V3 | Apply clears draft strip and writes row |
| V4 | Tooltips wired: `var_lighting`, `var_power`, `var_damage`, `var_fill`, `var_apply_layers` |

## Exit predicate

`pytest tools/mcp/python/tests/test_variants_layer_context.py -q` green + manual Variants tab scrub.

---

## Sign-off

| Role | Verdict | Date |
|:---|:---|:---|
| `@designer` | **PASS** | 2026-06-02 |
| `@coder-mcp` | **SHIPPED** | 2026-06-02 |
