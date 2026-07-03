# APS Variants — reaction pull-up confront critique `v1`

| Field | Value |
|:---|:---|
| **Gate** | **DMCP-APS-VARIANTS-REACTION-PULLUP-001** |
| **Program** | PLAN-DESIGNER-WORK-202606 · APS reaction-territory |
| **Date** | 2026-07-03 |
| **Owner** | `@designer-mcp` |
| **Depends on** | **DES-REACTION-TERRITORY-EVENTS-001** PASS · `APS-P0-REACTION-TERRITORY-001` witness |
| **Authority** | [`variants_panel.py`](../tools/mcp/art_pipeline_suite/variants_panel.py) · [`variants_preview_panel.py`](../tools/mcp/art_pipeline_suite/variants_preview_panel.py) · [`design_aps_uiux_preview_spec_v1.md`](design_aps_uiux_preview_spec_v1.md) |
| **Verdict** | **PASS** |

```text
DMCP-APS-VARIANTS-REACTION-PULLUP-001 Q✓
Variants tab is a proper workflow UI — safe to pull up; reaction catalog honest at schema layer
```

---

## Order critique

```yaml
order_critique:
  request_summary: "What does the artist confront on APS Variants when reaction-territory catalog is live (11 events)?"
  rules_audit:
    no_ai_generated_images: pass
    spec_only_honest: pass          # catalog marked spec_only; preview uses layer merge not fake bespoke art
    no_black_preview: pass          # preview contract — labelled empty/loading/error
    no_jargon_primary_chrome: pass  # reaction filter uses human labels from catalog
    deterministic_catalog: pass     # 10/10 liquidation triggers mapped; witness green
    no_bpy_in_gate: pass
  blocked: false
  proceed: yes
  foresight_flags:
    - "Listbox keys remain machine-shaped — readability tail @coder-mcp (P2 polish)"
    - "29 mandate tag checkboxes — collapse-to-suggested tail @coder-mcp"
    - "Distinct reaction art ≠ four-state thumb until tile bake — expected at schema gate"
    - "Default filter All sessions shows ~25 rows after New from assembly — filter UX tail"
```

---

## Confront audit (what the user actually sees)

| Surface | Finding | Grade |
|:---|:---|:---:|
| **Shell** | Tk/ttk APS desktop — Catalog → Materials → Assembly → **Variants** → Atlas | OK |
| **Empty state** | Plain copy + CTA (`New from assembly` / `Load example`) | OK |
| **Cold start guard** | No assembly → labelled preview empty + status (not crash/black) | OK |
| **Reaction filter** | Readonly combobox — 11 human event labels + Base / All | OK |
| **Variant list** | Listbox; reaction rows append `· {event_id}`; keys include hash suffix | ACCEPT |
| **Preview strip** | Clean / Night / Damaged / Burning chips + 128px thumb + context line | OK |
| **Live draft** | Layer edits debounce preview before Apply ([`DES-APS-VARIANTS-LIVE-PREVIEW-001`](design_aps_variants_live_preview_v1.md)) | OK |
| **Mandate tags** | 3-column checkbox wall (Light / Fire / Heritage) | ACCEPT |
| **Advanced** | Agent patch collapsed — JSON textarea not default chrome | OK |
| **Honesty** | Reaction events change layers/tags/metadata; thumb is building state axes — **not** unique liquidation art yet | OK |

**Not a horror show:** structured tab, design tokens, preview contract, filters, empty states.  
**Not pro-grade consumer UI:** scroll density, cryptic list keys, tag wall — **8/10 internal tool** band (consistent with [`design_aps_artist_ship_review_uiux_v1.md`](design_aps_artist_ship_review_uiux_v1.md)).

---

## Catalog coverage (designer-mcp scope)

| Check | Status |
|:---|:---:|
| Events on disk | **11** |
| `cultural_liquidation_trigger` rows from doc | **10/10** |
| `liquidation_triggers_complete` witness | green |
| Resolver smoke all domains | green |
| `spec_only: true` on catalog | yes |

---

## Handoff (non-blocking tails)

| Owner | Task | Priority |
|:---|:---|:---:|
| `@coder-mcp` | Default post-create filter → **Base sessions** | P2 |
| `@coder-mcp` | List label = event label + short state (hide hash in primary) | P2 |
| `@coder-mcp` | Collapse mandate tags; show event `tag_anchors` first | P2 |
| `@coder-mcp` | CMCP-REACTION-TERRITORY-PREVIEW-001 APS wire completion | P1 |
| `@designer` | One-line honesty banner on reaction row select (schema vs art) | P3 |

---

## Sign-off

| Role | Verdict | Date |
|:---|:---|:---|
| `@designer-mcp` | **PASS** | 2026-07-03 |

**Witness:** `debug_runs/art_pipeline/dmcp_aps_variants_reaction_pullup_live.json`  
**CLI:** `python -m rust_engine_mcp.cli dmcp-aps-variants-reaction-pullup-witness`
