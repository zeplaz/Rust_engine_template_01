# APS UI/UX P3 layout acceptance rubric `v1` — DMCP-OVR-P3-ACCEPT-RUBRIC-001

| Field | Value |
|:---|:---|
| **Gate** | **DMCP-OVR-P3-ACCEPT-RUBRIC-001** |
| **Owner** | `@designer-mcp` |
| **Date** | 2026-06-02 |
| **Program** | PLAN-APS-UIUX-OVERHAUL-001 |
| **Authority** | [`design_aps_uiux_layout_delta_v1.md`](design_aps_uiux_layout_delta_v1.md) · [`plan_aps_uiux_p3_layout_guard_v1.md`](plan_aps_uiux_p3_layout_guard_v1.md) |
| **Implements** | Operator **NEEDS-DISPLAY** gate for `OVR-P3-LAYOUT-001` |
| **Verdict** | **PASS** — rubric signed |

```text
DMCP-OVR-P3-ACCEPT-RUBRIC-001 Q✓
Cross-check for @coder-mcp OVR-P3-LAYOUT-001 + @operator eyeball
```

---

## 0. How to use

| Layer | Who | When |
|:---|:---|:---|
| **Machine** | `@coder-mcp` | `test_aps_min_window_layout.py` green before P3 Q✓ |
| **Rubric** | `@operator` + `@designer-mcp` | After P3 lands — walk both lanes @ 960×600 and 1280×800 |
| **Fail** | Any **P0** row red → P3 not accepted; file issue against `OVR-P3-LAYOUT-001` |

Scale: **pass** / **fail** / **n/a** (lane not applicable).

---

## 1. Window & shell (P0)

| ID | Check | Pass when |
|:---|:---|:---|
| **R-P3-01** | Min window | App opens usable at **960×600** — no clipped window chrome |
| **R-P3-02** | Default window | **1280×800** — Assembly footprint grid visible without vertical scroll |
| **R-P3-03** | Chrome rows | **≤2 rows** above notebook (lane+flow merged; authority+pipeline on row 2) |
| **R-P3-04** | Vertical budget | Notebook work area ≥ **70%** of client height @ MIN (no log expanded) |
| **R-P3-05** | Left gutter | Lane, flow, authority, pipeline share **same left inset** (`GAP_MD`) |
| **R-P3-06** | Status log | Log collapsed or capped — does not push notebook off-screen @ MIN |

Maps to guards **G-P3-01**, **G-P3-04**, **G-P3-05**.

---

## 2. Scroll & overflow (P0)

| ID | Check | Pass when |
|:---|:---|:---|
| **R-P3-07** | Tab h-scroll | No horizontal scrollbar on Catalog, Materials, Assembly, Variants, Atlas form tabs @ MIN |
| **R-P3-08** | Footprint canvas | Grid **intersects** notebook viewport on Assembly @ MIN — not fully clipped |
| **R-P3-09** | Canvas internal scroll | Footprint may scroll **inside** canvas; tab pane itself must not h-scroll |
| **R-P3-10** | Atlas preview | Tile preview grid scrolls inside preview widget only |

Maps to **G-P3-02**, **G-P3-03**.

---

## 3. Assembly tab density (P0 — worst offender)

| ID | Check | Pass when |
|:---|:---|:---|
| **R-P3-11** | Setup strip | Generate / Load / Save / checks in **one collapsible Setup** — default **collapsed** after first Assembly |
| **R-P3-12** | Metadata flow | `Where this data goes` **collapsed** on first paint |
| **R-P3-13** | 3-pane priority | Footprint + material library + inspector visible without scrolling past setup |
| **R-P3-14** | Footprint legends | Legends **below** canvas (not beside) — no pane overflow |
| **R-P3-15** | Iteration diff | Grammar iteration diff **hidden** until after iterate op |

Per layout delta **AS-1…AS-4**.

---

## 4. Other tabs (P1)

| ID | Check | Pass when |
|:---|:---|:---|
| **R-P3-16** | Catalog | Module list + detail visible @ MIN; primary actions not off-screen |
| **R-P3-17** | Materials | Category tree + profile list usable @ MIN |
| **R-P3-18** | Variants | Layer controls reachable without h-scroll |
| **R-P3-19** | Landscape Grammar | Node list + inspector in split; no orphan h-scroll |
| **R-P3-20** | Landscape States | Matrix columns fit or vertical scroll only |

---

## 5. Lane switch (P1)

| ID | Check | Pass when |
|:---|:---|:---|
| **R-P3-21** | Lane repaint | Switching Buildings ↔ Landscape repaints chrome (authority + pipeline + flow) |
| **R-P3-22** | Selection isolation | No module selection bleeding into Landscape presets (and vice versa) |
| **R-P3-23** | Tab memory | Each lane remembers last tab index independently |

---

## 6. P2 copy spot-check @ layout (P1)

After P3 merges `app.py`:

| ID | Check | Pass when |
|:---|:---|:---|
| **R-P3-24** | Flow caveat | Row 1 shows artist copy — **not** `rust_engine_mcp` / `MCP` (CP-04) |
| **R-P3-25** | Authority voice | `What ships:` phrasing on row 2 — not `Ship truth:` |

Cross-ref [`design_aps_uiux_p2_impl_audit_v1.md`](design_aps_uiux_p2_impl_audit_v1.md).

---

## 7. Regression bundle (machine — must be green)

```powershell
cd tools/mcp/python
python -m pytest tests/test_aps_min_window_layout.py -q
python -m pytest tests/test_aps_imports.py tests/test_aps_runtime_callbacks.py -q
python -m pytest -k aps -q
```

**Forbidden:** accepting P3 on pytest skip · window smaller than 960×600 in guard tests.

---

## 8. Acceptance summary

| Tier | Rule |
|:---|:---|
| **P0** | R-P3-01…15 + machine guards G-P3-01…05 all pass |
| **P1** | R-P3-16…25 — at least 8/10 pass; failures documented for P4/P5 |
| **Exit** | `@designer-mcp` stamps witness `dmcp_ovr_p3_accept_rubric_live.json` **after** operator walk |

---

## 9. Operator walk script (~5 min)

```text
1. Launch: python tools/mcp/art_pipeline_suite/run.py
2. Resize to 960×600 — walk Buildings tabs 0→4, note h-scroll / clipped CTAs
3. Assembly — confirm footprint grid visible; collapse Setup + metadata
4. Switch Landscape — Presets → Grammar → States → Atlas
5. Resize 1280×800 — confirm footprint without scroll
6. Expand status log — notebook must still show primary work object
```

Record pass/fail per §1–6 in operator notes; attach screenshot only on **fail**.

---

## 10. Sign-off

| Role | Verdict | Date |
|:---|:---|:---|
| `@designer-mcp` | **PASS** (rubric locked) | 2026-06-02 |
| `@operator` | — | After P3 lands |

```text
DMCP-OVR-P3-ACCEPT-RUBRIC-001 Q✓ — 25 rubric rows · maps to G-P3-01…06
```
