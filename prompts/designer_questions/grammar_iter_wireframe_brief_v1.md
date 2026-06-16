# GRAMMAR-ITER-001 — Designer wireframe brief

**Spec:** [`grammar_iter_001_spec_v1.md`](../../docs/archive/2026-06-src-dev/plans/grammar_iter_001_spec_v1.md)  
**Owner:** `@designer` (after @planner-mcp spec)  
**APS surface:** Assembly tab — **Iterate grammar** panel

---

## Design problem

Artists lose good buildings when they change one grammar layer because **Generate** rerolls everything from seed.

**Target:** Republic-style iteration — tune massing, then facade, then materials — with **visible diff** and **preview refresh** after each step.

---

## Required controls (minimum)

| Control | Artist intent |
|:---|:---|
| **Mode** dropdown | Which layer am I editing? |
| **Apply iteration** | Run partial regen (not full Generate) |
| **Pin checkboxes** | Keep district / age while changing massing |
| **Diff summary** | "+12 −4 ~6 cells" — plain language |
| **Preview assembly** | Same as today — must stay one click away |
| **Save branch** | New assembly_id with parent lineage |
| **Revert to parent** | Undo iteration chain |

## Mode-specific fields

| Mode | Show |
|:---|:---|
| Massing | Strategy radios, W×D, floors, footprint_mode read-only label |
| Roof | Roof slot dropdown |
| Facade | Wall/door/window slot dropdowns |
| Material strategy | Link to Materials tab + district profile summary |
| Placement | Defer to existing slot editor (no duplicate) |

---

## Visual feedback

1. **Footprint grid** — color added/removed/changed cells after Apply  
2. **Grammar inspector** — pinned overrides in bold; lineage line  
3. **Slot preview** — refresh selected cell automatically on Apply  

---

## Deliverables

1. Annotated wireframe (ASCII or screenshot markup) for Iterate panel placement  
2. Label glossary review — edit [`grammar_labels_v1.json`](../../assets/configs/buildings/grammars/grammar_labels_v1.json)  
3. Accessibility: mode + Apply reachable without hover-only hints  
4. Sign-off row: `GRAMMAR-ITER-001-UI` in designer registry  

---

## Paste back to orchestrator

```text
GRAMMAR-ITER-001-UI complete
Wireframe: prompts/designer_questions/grammar_iter_wireframe_v1.md (or image path)
Top 3 UX risks:
1.
2.
3.
Sign-off: PASS | PASS WITH NOTES
```
