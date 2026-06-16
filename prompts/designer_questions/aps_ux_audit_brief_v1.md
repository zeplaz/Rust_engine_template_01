# APS-UX-AUDIT-001 — Designer brief (Art Pipeline Suite)

**Program:** [`plan_aps_artist_tool_exec_v1.md`](../../src/dev/plan_aps_artist_tool_exec_v1.md)  
**Owners:** `@designer` (lead) · `@designer-mcp` (artist workflow acceptance)  
**Launch:** `python -m art_pipeline_suite.run` from repo root (MCP Python env)

---

## Your mission

Review the **Art Pipeline Suite** as an **artist authoring tool** — not a data editor. Score whether a new artist can:

1. Understand **what they are looking at** (module, material, assembly, tiles)
2. **Tag and assign metadata** correctly knowing it becomes **engine truth** via `assembly_snapshot`
3. Complete daily work **without Blender** (preview, QC, atlas review)
4. Recover from errors (validation, missing GLB, missing material)

**Keyframe bake in Blender is out of scope** — that is a separate ship lane behind Atlas.

---

## Tabs to review

| Tab | Focus questions |
|:---|:---|
| **Catalog** | Can I find modules, validate GLB, edit sidecar without breaking index? |
| **Assembly** | Is grammar + footprint + slot previews + tags a coherent story? |
| **Materials** | Can I browse 300 profiles, preview, and know how to assign on Assembly? |
| **Variants** | Do variant layers make sense before tile batch? |
| **Atlas** | Can I QC individual PNGs and packed atlas before register? |

---

## Deliverables (required)

1. **Scores (1–10):** clarity · discoverability · error recovery · accessibility · workflow efficiency  
2. **Top 10 issues** — ranked; tag each `P0` / `P1` / `P2`  
3. **Top 5 fixes for coder-mcp** — specific, implementable (not “make it prettier”)  
4. **Tooltip copy review** — edit or approve strings in `tools/mcp/art_pipeline_suite/aps_tooltips.py`  
5. **Optional:** one-page IA diagram (which tab owns which authority)  
6. **Sign-off:** add `APS-UX-AUDIT-001` to `tools/orchestrator/queues/designer_signoff_registry.json`

---

## Accessibility & readability checklist

- [ ] Status not conveyed by color/glyph alone  
- [ ] Minimum readable font sizes (avoid Consolas 8 for primary labels)  
- [ ] Critical actions not tooltip-only  
- [ ] Scroll regions obvious (footprint grid, material list, atlas cells)  
- [ ] Paned layouts usable at 960×600  
- [ ] Metadata → engine panels understandable without reading ARCH-MAT doc  
- [ ] Pipeline status bar matches mental model  

---

## Paste back to orchestrator

```text
APS-UX-AUDIT-001 complete
Scores: clarity _ discoverability _ error_recovery _ accessibility _ efficiency _
Top 5 coder-mcp fixes:
1.
2.
3.
4.
5.
Sign-off: PASS | PASS WITH NOTES | FAIL
```
