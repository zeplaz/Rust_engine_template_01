# APS-UX-AUDIT-001 — Designer-MCP artist workflow audit (Phase 0)

| Field | Value |
|:---|:---|
| **Program** | `APS-UX-AUDIT-001` |
| **Track** | A — APS Product |
| **Agent** | `@designer-mcp` (artist workflow + QC lens) |
| **Pairs with** | `@designer` (lead sign-off) |
| **Brief** | [`prompts/designer_questions/aps_ux_audit_brief_v1.md`](../../prompts/designer_questions/aps_ux_audit_brief_v1.md) |
| **Exec** | [`plan_aps_artist_tool_exec_v1.md`](plan_aps_artist_tool_exec_v1.md) Phase 0 |
| **Date** | 2026-06-03 |
| **Verdict (designer-mcp)** | **PASS WITH NOTES** |
| **Witness** | [`debug_runs/aps_ux_audit_001_live.json`](../debug_runs/aps_ux_audit_001_live.json) |

---

## Production lens (Phase 9 preview)

**“Would an artist ship this workflow today?”** → **5 / 10**

Foundation is real (previews, metadata flow panels, pipeline bar, grammar inspector, materials studio). A first-time artist can *complete* the no-Blender path with documentation, but daily ship is blocked by discoverability gaps (material assign split across tabs), accessibility (glyph-only status), grammar readability (raw `rule_id`s), catalog lack of in-list thumbs, and atlas QC without inline plain-language validator.

**Target after Phases 2–5:** **7 / 10** (Phase 9 E2E gate).

---

## Scores (1–10)

| Dimension | Score | Rationale |
|:---|:---:|:---|
| **Clarity** | **6** | Slot previews + metadata flow panels answer “what am I looking at?” Grammar inspector and P0 gate still expose snake_case ids without human labels. |
| **Discoverability** | **5** | Pipeline bar helps; material assignment requires Assembly tab after Materials browse — no persistent “next step” callout. Catalog has no row thumb (Phase 2). |
| **Error recovery** | **5** | GLB validate surfaces green/red text; P0 gate opens report dialog. Atlas has no **Validate atlas meta** button with artist-readable sentences (Phase 3). |
| **Accessibility** | **4** | Materials map status is ●/◐/○ (+ color) without adjacent text. Pipeline ✓/○ only. Consolas 8 used for primary meta in atlas preview + material maps line. |
| **Workflow efficiency** | **6** | Generate → slot preview → tags → variants → atlas path exists. Assembly panel density + collapsed metadata flow slow first run. |

---

## Artist journey (no Blender daily path)

```text
Catalog          → validate GLB · edit sidecar (hints only)
     ↓ flow / manual
Assembly         → grammar generate · footprint · slot previews · semantic_tags · Save snapshot
     ↓ mat_apply / footprint cell
Materials        → browse profile · preview modes · Apply to selected placement (returns to Assembly authority)
     ↓ flow_bake_variants
Variants         → layer types · tile_batch JSON
     ↓ flow_pack_atlas
Atlas            → cell strip + packed PNG · pack · (register deferred)
```

**QC checkpoints per tab:**

| Tab | Pass | Gap |
|:---|:---|:---|
| **Catalog** | Validate GLB, sidecar editor, metadata flow (collapsed by default) | No in-list thumb; sidecar vs assembly authority not on-screen until expand flow panel |
| **Assembly** | Footprint grid, slot previews, grammar inspector summary, P0 gate | Grammar **why** = raw ids; dense inspector stack at 960×600 |
| **Materials** | 300+ profiles searchable, preview modes, Apply wires to snapshot | Map readiness glyph-only; category flat slash paths |
| **Variants** | Load/save variant set, agent patch area | Bake hint tooltip-only; layer semantics need TOOLTIPS-002 |
| **Atlas** | Packed atlas + cell strip + meta line (v1) | No UV grid overlay, no validate-report panel, errors would be JSON if added ad hoc |

---

## Top 10 issues (ranked)

| # | Pri | Issue |
|:---:|:---:|:---|
| 1 | **P0** | Grammar inspector **Rule ID** column shows snake_case only — artist cannot read “why” without glossary (Phase 6 `APS-UX-GRAMMAR-WHY`). |
| 2 | **P0** | Material map status **●/◐/○** without text labels — fails accessibility checklist; color-only status. |
| 3 | **P0** | **Catalog:** no module list thumbnail — artist opens browser preview or Assembly blind (Phase 2). |
| 4 | **P1** | **Atlas:** no inline `validate_atlas_meta_v2` with plain-language errors before register (Phase 3). |
| 5 | **P1** | **Materials → Assembly** assignment path not obvious on-screen (“select cell first” tooltip-only). |
| 6 | **P1** | Pipeline status bar **✓/○** without text (“complete” / “pending”) — same a11y class as materials glyphs. |
| 7 | **P1** | Metadata flow panel **collapsed by default** — ARCH-MAT-001 sidecar vs snapshot authority easy to miss. |
| 8 | **P2** | **Consolas 8** on atlas cell meta, material maps line, slot mesh path — below readable minimum for primary labels. |
| 9 | **P2** | **P0 gate** report dialog is technical — needs code→sentence map (Phase 7 `APS-MAT-AUTH-UI-001`). |
| 10 | **P2** | **Variants** tab lacks on-screen “next: Atlas pack” callout matching pipeline bar. |

---

## Top 5 fixes for @coder-mcp

1. **APS-UX-GRAMMAR-WHY** — Add `detail` column labels from [`aps_ux_grammar_why_glossary_v1.md`](aps_ux_grammar_why_glossary_v1.md); show human label beside `rule_id` in grammar tree.
2. **APS-UX-POLISH-001 (status text)** — Materials: `Ready` / `Partial` / `Missing` beside ●◐○; pipeline bar: `Catalog ✓ complete` vs `○ pending`.
3. **APS-PREVIEW-CATALOG-001** — List-row GLB thumb + one-line sidecar≠ship truth under module summary.
4. **APS-ATLAS-PREVIEW-002** — Validate button → plain-language panel; UV grid overlay on packed atlas.
5. **APS-UX-TOOLTIPS-002 + next-step callout** — Assembly post-generate banner: “Select footprint cell → Materials tab → Apply → Save snapshot”.

---

## Accessibility checklist (brief)

| Item | Status |
|:---|:---|
| Status not color/glyph alone | **FAIL** — materials + pipeline bar |
| Min readable font sizes | **PARTIAL** — Consolas 8 in atlas/materials/slot meta |
| Critical actions not tooltip-only | **PARTIAL** — mat apply, P0, sidecar authority |
| Scroll regions obvious | **PASS** — footprint list, material grid, atlas cells |
| Paned layouts at 960×600 | **PARTIAL** — assembly inspector tight; materials pane min width hack exists |
| Metadata → engine without ARCH doc | **PARTIAL** — panel exists but collapsed |
| Pipeline bar mental model | **PASS** — step order matches journey |

---

## Information architecture (authority)

```text
Catalog sidecar     → hints · module resolver input
Assembly snapshot   → AUTHORITY (material_profile, semantic_tags, placements)
Materials registry  → profile source; assign writes into snapshot only
Variant set         → derived from snapshot → tile_batch
Atlas meta/PNG      → tile lookup; independent of per-slot material_profile
```

---

## Tooltip copy review (`aps_tooltips.py`)

**Status:** APS-UX-TOOLTIPS-002 strings largely on disk (designer-reviewed pass). **APPROVE** current registry; bind `mat_status` + `pipeline_step` to UI in Phase 5 polish.

| Key | Verdict |
|:---|:---|
| `tab_catalog`, `cat_metadata`, `cat_sidecar_truth` | **APPROVE** — sidecar ≠ ship truth clear |
| `asm_material_lib`, `mat_apply`, `mat_status` | **APPROVE** — assignment order + map status text in tooltip |
| `asm_p0`, `asm_save_reminder` | **APPROVE** |
| `atl_qc`, `atl_preview` | **APPROVE** — v2 grid called out |
| `pipeline_step` | **APPROVE** copy · **wire** to pipeline bar labels (P1) |

---

## Verification run

| Check | Result |
|:---|:---|
| `pytest test_aps_preview_001 + test_aps_atlas_preview` | **8 passed** |
| `validate-report asset_glb` | **Skipped** — no indexed `model.glb` on disk in worktree |
| Code review | `art_pipeline_suite/*` tabs + grammar inspector + pipeline bar |

---

## Sign-off

| Role | Verdict |
|:---|:---|
| **@designer-mcp** | **PASS WITH NOTES** — unblocks coder-mcp Phases 2–4 draft work |
| **@designer** (lead) | **PASS** — upgraded 2026-06-03 after APS-UX-POLISH-001-SIGNOFF |

**Co-sign registry:** `tools/orchestrator/queues/designer_signoff_registry.json` → `APS-UX-AUDIT-001`

**Deferred (Track B):** MCP-PILOT-GRAMMAR-001 manual keyframe — not blocking APS 2–7.
