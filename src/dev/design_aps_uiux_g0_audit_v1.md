# APS UI/UX G0 audit — §2 ban-list + voice rules `v1` (DMCP-OVR-G0-AUDIT-001)

| Field | Value |
|:---|:---|
| **Program** | **PLAN-APS-UIUX-OVERHAUL-001** |
| **Gate** | **DMCP-OVR-G0-AUDIT-001** |
| **Owner** | `@designer-mcp` |
| **Date** | 2026-06-02 |
| **Charter** | [`plan_aps_uiux_overhaul_20260616_v1.md`](plan_aps_uiux_overhaul_20260616_v1.md) §2 |
| **Design lock** | [`aps_design_system_v1.md`](aps_design_system_v1.md) |
| **Parallel** | `OVR-P1-TOKENS-001` (tokens — does not clear text debt) |
| **Verdict** | **PASS WITH NOTES** (audit complete · UI **not** clean) |
| **UI ban-list** | **FAIL** — 54 hits / 29 P0-class (automated scan) |

---

## Order critique

```yaml
order_critique:
  request_summary: "G0 ban-list audit vs §2 before P2 text implementation"
  rules_audit:
    ban_list_enforced: fail   # expected pre-OVR-P2-TEXT-001
    voice_rules_documented: pass
    glossary_locked_in_p0: pass  # aps_design_system_v1.md §1
  blocked: false              # audit slice closes; P2 blocked ON violations
  proceed: yes_with_notes
  reroute: "OVR-P2-TEXT-001 after OVR-DES-P2-COPY-PACK-001 + P1 token guards"
```

**Headline:** Engineering vocabulary is still on screen. Gate IDs in LabelFrame titles, schema names in tooltips, and `Ship truth:` authority strips violate §2b. This is **expected** before `OVR-P2-TEXT-001` — the audit is honest, not a dishonest green.

---

## Automated scan (2026-06-02)

| Metric | Count |
|:---|:---:|
| Total §2b hits | **54** |
| P0-class hits | **29** |
| Files touched | **15+** |
| Guard | `tests/test_aps_no_jargon.py` |

**Witness:** `debug_runs/art_pipeline/dmcp_ovr_g0_audit_live.json`  
**CLI:** `python -m rust_engine_mcp.cli dmcp-ovr-g0-audit-witness`

### Top rules violated

| Rule | Hits | Example |
|:---|:---:|:---|
| `gate_id_aps` | 11 | `(APS-MAT-AUTH-UI-001)`, `(APS-PREVIEW-001)` |
| `gate_id_arch` | 6 | `(ARCH-MAT-001)`, `ARCH-DNA` |
| `gate_p0` | 4 | Button `P0 gate` |
| `schema_material_profile` | 4 | tooltips + metadata diagram |
| `schema_land_dna` | 4 | Grammar banner + metadata |
| `schema_assembly_snapshot` | 4 | lane banners, tooltips |
| `gate_lg5` | 3 | flow verbs, pipeline hints |
| `voice_ship_truth` | 1 | `domain_router` authority strip |

### Worst files (fix first in P2)

| File | Hits | Priority |
|:---|:---:|:---:|
| `metadata_flow_panel.py` | 14 | P0 — diagram + title `(ARCH-MAT-001)` |
| `assembly_panel.py` | 6 | P0 — `P0 gate`, `(APS-MAT-AUTH-UI-001)` |
| `atlas_panel.py` | 6 | P1 — `tile_batch_v1`, `tilemapgen` |
| `grammar_dna_panel.py` | 4 | P0 — `ARCH-DNA + β` |
| `aps_tooltips.py` | (in scan) | P1 — schema names in hover text |

Full finding table + rewrites: [`aps_sweep_text_20260616_v1.md`](aps_sweep_text_20260616_v1.md).

---

## §2 voice rules — audit checklist

| Rule | Status | Notes |
|:---|:---:|:---|
| Sentence case body / Title tabs | partial | Mixed |
| Buttons = imperative + canonical noun | partial | `P0 gate` vs `Validate` ambiguous |
| Status word-first | pass | Pills use words |
| FAIL states fix in artist verbs | partial | P0 dialog still says "P0 gate" |
| No code/types/env in body | **fail** | metadata diagram, engine path |
| No gate/program IDs in chrome | **fail** | ~12 LabelFrame titles |
| One **"What ships:"** phrasing | **fail** | `Ship truth:` variant remains |

---

## Handoff

| Slice | Owner | Action |
|:---|:---|:---|
| **OVR-DES-P2-COPY-PACK-001** | @designer | Signed rewrites for worst offenders |
| **OVR-P2-TEXT-001** | @coder-mcp | Implement copy + `test_aps_no_jargon.py` → 0 hits |
| **OVR-P1-TOKENS-001** | @coder-mcp | Parallel — no text scope |

**Exit for P2:** `pytest tools/mcp/python/tests/test_aps_no_jargon.py` green + operator eyeball per `aps_design_system_v1.md` §6 DoD.

---

## Sign-off

| Role | Verdict | Date |
|:---|:---|:---|
| `@designer-mcp` | **PASS WITH NOTES** (G0 audit closed) | 2026-06-02 |

```text
DMCP-OVR-G0-AUDIT-001 complete
54 ban-list hits documented · guard test wired · P2 must drive to 0
```
