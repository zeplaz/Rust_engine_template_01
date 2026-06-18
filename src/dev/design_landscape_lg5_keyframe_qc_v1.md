# LG-5 expanded keyframe QC `v1` — DMCP-LG5-KEYFRAME-QC-001

| Field | Value |
|:---|:---|
| **Gate** | **DMCP-LG5-KEYFRAME-QC-001** |
| **Program** | APS-E4 · landscape production |
| **Owner** | `@designer-mcp` |
| **Date** | 2026-06-02 |
| **Authority** | [`design_landscape_keyframe_burn_reqs_v1.md`](design_landscape_keyframe_burn_reqs_v1.md) · [`tile_batch_landscape_expanded_v1.json`](../assets/staging/specs/tile_batch_landscape_expanded_v1.json) |
| **Keyframes** | `assets/staging/tiles/keyframe_stills/tile_landscape_expanded_v1/` |
| **Verdict** | **PASS WITH NOTES** — teach bake 16/16 · G4 manual still required |

```yaml
order_critique:
  request_summary: "QC procedural teach keyframes vs burn/scar reqs before ship flip"
  rules_audit:
    deterministic_output: pass
    batch_alignment: pass
    g4_manual_stills: open
  proceed: yes_with_notes
```

---

## 1. Scope

| Layer | Status |
|:---|:---|
| **Phase A teach** | 16 PNG @ 64×64 · seed **550005** · procedural overlay |
| **Phase B G4** | Manual stills in iso rig — **not** authorized for `ship: true` |
| **Matrix** | 16-cell expanded batch (not full 22-row reqs superset) |

---

## 2. Batch ↔ reqs alignment (16 cells)

All `variant_key` rows in the expanded batch have a matching §2 reqs row. Cells **not** in the 16-cell matrix (e.g. `topology_patch_burn_07`, `topology_corridor_regrowth_grass`) remain **deferred** to G4 expansion — not a QC fail for this pilot.

| G4 operator minimum (§3) | In 16-cell batch? | Teach PNG |
|:---|:---:|:---:|
| `topology_patch_burn_04` | yes | yes |
| `topology_patch_scar` | yes | yes |
| `topology_corridor_regrowth_grass` | **no** | deferred |

---

## 3. Per-variant QC rubric (teach tier)

| Check | Pass rule |
|:---|:---|
| File exists | `{variant_key}.png` in keyframe folder |
| Dimensions | 64×64 px |
| Determinism | Same seed 550005 · no unseeded export |
| Burn read | `_burn_*` keys show orange overlay (teach) |
| Scar read | `_scar` keys show ash wash |
| Regrowth read | `_regrowth_*` keys show green/brown shift |

**Honest label:** teach PNGs prove **topology + state lanes** — not art-ship quality.

---

## 4. Forbidden (re-affirm §4 reqs)

- Headless ortho-only for `ship: true`
- Unseeded per-export randomness
- Ad-hoc exports outside batch folder

---

## 5. Handoff

| Owner | Next |
|:---|:---|
| **Operator** | G4 manual stills for minimum review set + corridor regrowth |
| **@coder-mcp** | `landscape_lg5_expanded_batch` after manual folder drop |
| **@designer-mcp** | `DMCP-VEG-ATLAS-SHIP-001` after G4 green |

DMCP-LG5-KEYFRAME-QC-001 Q✓ — teach tier PASS WITH NOTES
