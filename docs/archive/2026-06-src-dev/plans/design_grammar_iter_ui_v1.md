# GRAMMAR-ITER-001-UI — Designer sign-off record `v1`

| Field | Value |
|:---|:---|
| **Program** | `GRAMMAR-ITER-001-UI` |
| **Owner** | `@designer` |
| **Verdict** | **PASS WITH NOTES** |
| **Date** | 2026-06-03 |
| **Wireframe** | [`prompts/designer_questions/grammar_iter_wireframe_v1.md`](../prompts/designer_questions/grammar_iter_wireframe_v1.md) |
| **Labels** | [`assets/configs/buildings/grammars/grammar_labels_v1.json`](../assets/configs/buildings/grammars/grammar_labels_v1.json) v1.1.0 |

---

## Deliverables

| # | Item | Status |
|:---:|:---|:---:|
| 1 | Annotated wireframe (Iterate panel + placement + mode variants) | done |
| 2 | `grammar_labels_v1.json` review + edits | done |
| 3 | Accessibility: mode + Apply on-panel | done |
| 4 | Registry row `GRAMMAR-ITER-001-UI` | done |

---

## Labels review summary

| Change | Rationale |
|:---|:---|
| Added `detail.prop_clutter` | Matches grammar RON; inspector + Detail mode dropdown |
| Added `footprint_mode.l_shape` | Same id used in massing L-strategy path |
| Renamed `facade.facade_v1` → **Industrial Facade Pack** | Matches live snapshots (`facade_v1` rule_id) |
| Added `ui.*` strings | Iterate panel copy single source for coder-mcp |
| Title Case in JSON | UI dropdowns; Python `aps_grammar_labels.py` should load JSON in Phase 2 (parity note) |

**Note:** `aps_grammar_labels.py` still uses inline dict with sentence-case — **@coder-mcp** should load `grammar_labels_v1.json` to avoid drift (does not block Phase 1 wireframe).

---

## Top 3 UX risks

1. Generate vs Apply — separate frames + status copy required  
2. Material strategy mode — link only, no duplicate picker  
3. Small footprint diff noise — text summary fallback  

---

## Notes (PASS WITH NOTES)

- Phase 1 must show **disabled Apply** with `ui.phase2_disabled` string visible on-panel  
- `placement` mode defers to slot editor — no duplicate controls  
- Pin roof slot optional checkbox deferred to Phase 2 when roof mode ships  

**Unblocks:** `GRAMMAR-ITER-001-APS1` · `GRAMMAR-ITER-001-API`

---

## Paste back

```text
GRAMMAR-ITER-001-UI complete
Wireframe: prompts/designer_questions/grammar_iter_wireframe_v1.md
Top 3 UX risks:
1. Generate vs Apply confusion
2. Material strategy scope creep
3. Small footprint diff legend noise
Sign-off: PASS WITH NOTES
```
