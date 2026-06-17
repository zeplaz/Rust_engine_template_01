# PLAN-OVR-P3-GUARD-SPEC-001 — P3 min-window layout guard `v1`

```text
⟦SYMLANG⟧⟐v1  ◈PLAN
⟨ID⟩ PLAN-OVR-P3-GUARD-SPEC-001
Date: 2026-06-17
Status: **SIGNED** (@planner-mcp)
Parent: PLAN-APS-UIUX-OVERHAUL-001 · OVR-P3-LAYOUT-001
Authority: $ref:src/dev/design_aps_uiux_layout_delta_v1.md §0
Test: `tools/mcp/python/tests/test_aps_min_window_layout.py` (coder-mcp ships with OVR-P3-LAYOUT-001)
```

**Goal:** Machine-verifiable acceptance for **960×600** min window — no tab h-scroll, footprint visible, chrome budget honored.

---

## Window matrix

| Mode | WxH | Role |
|:---|:---|:---|
| **MIN** | 960×600 | Guard target — all assertions |
| Default | 1280×800 | Smoke only (optional second test) |

---

## Assertions (`test_aps_min_window_layout.py`)

| ID | Assertion | Pass when |
|:---|:---|:---|
| **G-P3-01** | Window geometry | Root `minsize` or test harness sets 960×600 before layout |
| **G-P3-02** | No tab h-scroll | Buildings + Landscape notebook tabs: `ScrollableFrame.enable_horizontal == False` on form tabs |
| **G-P3-03** | Footprint visible | Assembly tab: `FootprintCanvas` bbox intersects notebook viewport (not fully clipped) |
| **G-P3-04** | Chrome budget | Combined chrome rows ≤ **72px** measured (2-row merge per layout delta SH-1) |
| **G-P3-05** | No off-screen primary | Catalog/Assembly primary CTA widgets have `winfo_viewable()` true at MIN |
| **G-P3-06** | Token literals | No new raw `padx=`/`pady=` in `app.py` touched by P3 — defer to `test_aps_style_tokens.py` if shared |

**Forbidden pass:** scaling window below 960×600 in test · skipping Assembly tab · mocking footprint away.

---

## pytest contract

```python
# Minimal shape — @coder-mcp implements in OVR-P3-LAYOUT-001
def test_aps_min_window_no_horizontal_scroll_at_960x600(tk_root): ...
def test_aps_assembly_footprint_visible_at_min(tk_root): ...
```

**Exit:** both pass + `pytest -k aps` baseline held (161+ at wave-3 open).

---

## Sign-off

| Role | Date | Action |
|:---|:---|:---|
| **@planner-mcp** | 2026-06-17 | **SIGNED** — guard table authoritative |
| **@coder-mcp** | — | Implements test + layout in **OVR-P3-LAYOUT-001** |
| **@designer-mcp** | — | Rubric cross-check **DMCP-OVR-P3-ACCEPT-RUBRIC-001** |

```text
⟦/PLAN-OVR-P3-GUARD-SPEC-001⟧  ΔWF→ OVR-P3-LAYOUT-001
```
