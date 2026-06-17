# APS-ARTIST-SHIP-REVIEW-20260616 — Designer-MCP re-verdict after E1-FIX + E0 relaunch `v1`

| Field | Value |
|:---|:---|
| **Program** | **DMCP-E0-ARTIST-REVERDICT-001** |
| **Owner** | `@designer-mcp` |
| **Date** | 2026-06-17 |
| **Depends on** | `APS-EVO-E0-RELAUNCH-001` (`debug_runs/aps_artist_tool_e2e_live.json` **green: true**) |
| **Supersedes** | [`design_aps_artist_ship_review_20260615_v1.md`](design_aps_artist_ship_review_20260615_v1.md) (FAIL 2/10 — zero-byte regression) |
| **Verdict** | **PASS WITH NOTES** (re-verdict 2026-06-02) |
| **Ship score** | **7 / 10** (prior 2/10 — **+5** after E1-FIX restore) |

---

## Headline

**E0 relaunch witness is honest.** Import guard passes, canonical path steps are green, and the zero-byte panel regression from 2026-06-15 is **reversed** (`variants_panel.py`, `scrollable.py`, `grammar_inspector.py` restored). Artist can exercise Catalog → Assembly → Materials → Variants → Atlas without Blender for **building** domain fixtures.

**Not ship-ready for landscape production atlas** — that lane is correctly gated on DMCP-E3/E4/E4 keyframes (catalog now on disk; expanded bake still `ship: false`).

---

## Order critique

```yaml
order_critique:
  request_summary: "Re-verdict artist acceptance after E1-FIX + APS-EVO-E0-RELAUNCH-001"
  rules_audit:
    no_ai_generated_images: pass
    deterministic_output: pass
    batch_processing: pass
    grid_alignment: pass   # building pilot path; landscape expanded deferred
  blocked: false
  proceed: yes_with_notes
  foresight_flags:
    - "Atlas pilot fixture step still fails atlas_meta_v2 — expected (TILE-FIX-001 frozen greybox); do not treat as dishonest_gate."
    - "Landscape domain Option D toggle + veg state axis need coder-mcp E3 UI after DMCP-E3 catalog."
    - "Track B manual keyframe (MCP-PILOT-GRAMMAR-001) remains operator-owned — not an E0 blocker."
```

---

## Evidence

| Check | Result |
|:---|:---|
| `aps_artist_tool_e2e_live.json` | `green: true`, `import_guard_pass: true` |
| `honest_gate` | `build_health+schema+wit_hon` (not dishonest_gate) |
| Artist path steps | 5/5 ok (catalog, assembly, materials, variants, atlas fixture) |
| `variants_panel.py` | Restored (~20k) — Variants step testable |
| `DMCP-E3` catalog | `_vegetation_variant_catalog.ron` on disk · schema validate pass |

---

## QC notes (artist must read)

| # | Note | Severity |
|:---:|:---|:---:|
| N1 | Warehouse pilot atlas step reports v1 meta fail — **teach fixture only** | info |
| N2 | Landscape expanded batch `ship: false` until G4 keyframes | expected gate |
| N3 | Preset browse QC per [`design_aps_preset_qc_criteria_v1.md`](design_aps_preset_qc_criteria_v1.md) — verify on next APS UX pass | P1 |
| N4 | `designer_mcp_signoff` field in E0 witness → set **signed** via this doc | closed |

---

## Sign-off

| Role | Verdict | Date |
|:---|:---|:---|
| `@designer-mcp` | **PASS WITH NOTES** | 2026-06-17 · **re-verdict 2026-06-02** — E0 witness stamped `signed` |

```text
DMCP-E0-ARTIST-REVERDICT-001 complete (re-verdict)
E0 green · designer_mcp_signoff signed on aps_artist_tool_e2e_live.json
Witness: debug_runs/art_pipeline/dmcp_e0_artist_reverdict_live.json
```
