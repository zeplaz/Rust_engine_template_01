# APS Grammar Tier Exposure Contract `v1` — designer authority

| Field | Value |
|:---|:---|
| **ID** | **DES-APS-GRAM-TIER-EXPOSURE-001** |
| **Program** | PLAN-APS-GRAMMAR-EVOLUTION-001 |
| **Date** | 2026-06-17 |
| **Owner** | **`@designer`** — sole authority for APS exposure per tier |
| **Implements** | `AssemblyPanel.apply_grammar_tier()` · `APS-GRAM-TIER-002` · `APS-GRAM-TIER-004` |
| **Visual appendix** | [`design_aps_grammar_tier_wireframes_v1.md`](design_aps_grammar_tier_wireframes_v1.md) (APS-GRAM-TIER-003) |
| **Content tiers** | [`plan_aps_grammar_evolution_v1.md`](plan_aps_grammar_evolution_v1.md) § maturity model — **disk** bar; this doc is **UI** bar |
| **Verdict** | **PASS** |

```text
@designer owns APS exposure per tier
@coder-mcp implements §2 matrix only — no new surfaces without designer amend
```

---

## 0. Ownership rule

| Role | Owns | Does not own |
|:---|:---|:---|
| **@designer** | Which APS surfaces appear at G0–G4; collapsed vs visible; kit hint; spine copy; preview fidelity unlock | `grammar_set_tier()` disk math, RON files, evaluator |
| **@coder-mcp** | `apply_grammar_tier(tier)` mapping; guards; pipeline `grammar_tier` field | Changing exposure without designer sign-off |
| **@designer-mcp** | Grammar **content** (archetypes, districts) that **raises** tier on disk | APS chrome gates |

**Hard gate:** tooling row Q✓ forbidden if UI exposure disagrees with this matrix (witness scanner or `test_aps_grammar_tier_gates.py`).

---

## 1. Exposure modes (implementation enum)

| Mode | Meaning | `CollapsibleSection` |
|:---|:---|:---|
| `hidden` | Not packed — artist cannot open | `pack_forget()` |
| `collapsed` | Packed; header only | `_expanded = False` |
| `visible` | Packed; body expanded | `_expanded = True` |
| `promoted` | Always-visible strip (not collapsible) | build-set health @ G2+ |

---

## 2. Master matrix — Buildings · Assembly tab

**Always visible (all tiers G0–G4):** Generate row · footprint grid · placement list · material assign · grammar inspector (**collapsed** default) · pipeline spine.

| Surface | G0 | G1 | G2 | G3 | G4 |
|:---|:---:|:---:|:---:|:---:|:---:|
| **Tier chip** | `G0 — pilot kit` | `G1 — family seed` | `G2 — axis coverage` | `G3 — layer depth` | `G4 — production set` |
| **Archetype combo** | 1–2 + **kit hint** | ≥3 grouped | ≥3 + optional DNA badge on row | same | same |
| **District combo** | 1–2 | ≥2 per archetype | same | same | same |
| **Kit hint strip** | **visible** | **hidden** | hidden | hidden | hidden |
| **Shape bias / DNA panel** | hidden | collapsed | collapsed | **visible** (preset + β) | visible; save default on |
| **Iterate grammar panel** | hidden | collapsed | collapsed | **visible** | visible + diff overlay default |
| **Build-set / sweep** | collapsed | collapsed | **promoted** strip | promoted + sweep required | CI-linked |
| **Manual style fallback** | collapsed | collapsed | collapsed | collapsed | deprecated banner if grammar covers |
| **Inspector ↔ grid link** | — | — | **P3** on row click | same | same |
| **Assembly preview** | P1 slot + P2 quick | P2 | P2 + rule highlight | P3 grammar-aware | P4 ship fidelity |
| **Variants / Atlas spine** | visible; bake may warn | same | preview hints ↔ grammar tags | same | full ship path |
| **Max expanded grammar panels @ launch** | **≤2** | ≤2 | ≤3 | ≤4 | ≤4 |

### Kit hint copy (G0 only)

```text
One building type in the kit for now — add grammar files under assets/configs/buildings/grammars/ to grow this list.
```

(G1+:** strip hidden** — not downgraded caption; use tier chip for maturity story.)

---

## 3. `apply_grammar_tier()` mapping (coder contract)

```text
tier == G0:
  kit_hint → visible
  dna → hidden; iterate → hidden; build_set → collapsed

tier == G1:
  kit_hint → hidden
  dna → collapsed; iterate → collapsed; build_set → collapsed
  archetype_combo_count ≥ 3 when disk tier is G1

tier in G2, G3, G4:
  kit_hint → hidden
  dna → collapsed (G2) | visible (G3, G4)
  iterate → collapsed (G2) | visible (G3, G4)
  build_set → promoted expanded (G2+) | sweep gate (G3+)
```

**Scanner fields** (`grammar_tier_gate_snapshot`): `tier`, `dna_panel_visible`, `iterate_panel_visible`, `build_set_expanded_default`, `kit_hint_visible`, `archetype_combo_count`.

---

## 4. Pipeline spine copy (Buildings · Assembly step)

| Tier | Assembly pill / next-action emphasis |
|:---|:---|
| **G0, G1** | `Generate from building type` |
| **G2+** | `Tune shape bias; inspect rule chain` |

**Next-action line:**

| Tier | Copy |
|:---|:---|
| G0, G1 | `Generate from your building type and district.` |
| G2+ | `Tune shape bias; inspect the rule chain after Generate.` |

Other spine steps: unchanged per [`design_aps_uiux_spine_spec_v1.md`](design_aps_uiux_spine_spec_v1.md). **Pack atlas** may gray when `tier < G4` and ship check failed — not silent.

---

## 5. Preview fidelity ladder (tier-coupled)

| Level | ID | Unlocks at | Artist sees |
|:---|:---|:---:|:---|
| P0 | Footprint heatmap | G0 | W/D/C/R/Y tokens |
| P1 | Slot thumb | G0 | Isolated module + material |
| P2 | Assembly quick | G0 | Combined / browser |
| P3 | Grammar highlight | **G2** | Inspector row → grid + why tooltip |
| P4 | Ship render chip | **G4** | Keyframe / ship fidelity |

Spec: [`design_aps_uiux_preview_spec_v1.md`](design_aps_uiux_preview_spec_v1.md) · why copy: [`design_aps_grammar_why_copy_v1.md`](design_aps_grammar_why_copy_v1.md).

---

## 6. Content tier bars (disk — informs chip only)

| Tier | Content bar (not UI) | APS exposure unlock |
|:---|:---|:---|
| G0 | 1 archetype | G0 matrix |
| G1 | ≥3 archetypes | G1 matrix — kit hint off |
| G2 | DNA axes across set | G2 matrix — DNA promoted |
| G3 | facade + detail + age in chain | G3 matrix — iterate visible |
| G4 | diversity + module audit green | G4 matrix — ship path |

Disk authority: `grammar_set_tier()` in `grammar_build_set.py`. UI **must not** claim G1 while exposing G0 kit hint.

---

## 7. Anti-patterns (forbid)

- Five grammar panels full-width before **G1**
- Engineer ids in combos (`IndustrialWarehouse`)
- Duplicate walkthrough in Generate **and** spine
- Footprint canvas on Landscape Grammar tab
- Fake green: `tier=G1` witness with `kit_hint_visible=true`

---

## 8. Amend process

Changes to §2 require **@designer** amend + registry bump. Coder drive-by visibility changes are **rejected** at review.

---

## 9. Sign-off

| Role | Verdict | Date |
|:---|:---|:---|
| `@designer` | **PASS** — owns APS exposure per tier | 2026-06-17 |

**@coder-mcp:** cite `DES-APS-GRAM-TIER-EXPOSURE-001` in APS-GRAM-TIER-002 / TIER-004 commits.

---

## 10. G0 / G1 empty states — grammar content catch-up (DES-APS-GRAM-TIER-004)

When disk tier lags or jumps (content program GRAM-CONTENT-*), APS must **never** show a broken sparse UI. `@designer` owns copy; `@coder-mcp` wires from `grammar_tier_gate_snapshot()`.

### 10.1 G0 — pilot singleton (`archetype_combo_count < 3`)

| Surface | Condition | Copy |
|:---|:---|:---|
| **Kit hint strip** | `tier == G0` | `One building type in the kit for now — add grammar files under assets/configs/buildings/grammars/ to unlock more building types.` |
| **Archetype combo** | 1 value | Single label — no empty dropdown |
| **District combo** | 1–2 values | Same |
| **Generate row** | always | Building type + district enabled |
| **Empty assembly** | no snapshot | `No Assembly yet. Generate one from your building type.` |
| **Tier chip** | G0 | `G0 — pilot kit` |

**Kit hint:** visible · single line @ MIN · full text in tooltip.

### 10.2 G1 unlock (`archetype_combo_count >= 3`)

| Transition | UI behavior | Copy |
|:---|:---|:---|
| Tier flips G0→G1 | `kit_hint` **hidden** (`pack_forget`) | no replacement banner — tier chip tells story |
| **Archetype combo** | ≥3 values, grouped | See §2 dropdown grouping |
| **First open after unlock** | optional one-time toast | `✓ More building types available — pick a type and district, then Generate.` |
| **Tier chip** | G1 | `G1 — family seed` |
| **District empty for archetype** | 0 districts | `○ No districts for this type yet — check grammar files.` |

### 10.3 Mismatch guard (fake green prevention)

| Witness field | Rule |
|:---|:---|
| `tier` | must match `grammar_set_tier()` |
| `kit_hint_visible` | **true** only when `tier == G0` |
| `archetype_combo_count` | must equal `len(list_archetype_ids())` |
| `tier == G1` | implies `archetype_combo_count >= 3` and `kit_hint_visible == false` |

### 10.4 Spine copy at G0 vs G1

Both use **Generate-from-type** emphasis (unchanged from §4) until **G2** shape-bias copy activates.

| Tier | Assembly step hint |
|:---|:---|
| G0, G1 | `Generate from your building type and district.` |

### 10.5 Implementation hook

```python
# assembly_panel.apply_grammar_tier — after combo refresh:
if tier == "G0":
    show_kit_hint(KIT_HINT_G0)
elif len(archetypes) >= 3:
    hide_kit_hint()
    # optional: show_g1_unlock_toast() once per prefs
```

**Prefs key:** `grammar_g1_unlock_toast_seen_v1` (bool).

---

## 11. Sign-off amendment (DES-APS-GRAM-TIER-004)

| Role | Verdict | Date |
|:---|:---|:---|
| `@designer` | **PASS** — §10 empty states | 2026-06-17 |

---

## 12. Industrial facility surfaces (DES-APS-FACILITY-NEEDS-001 / DES-APS-SITE-PREVIEW-001)

Add to §2 master matrix — Buildings · Assembly tab:

| Surface | G0 | G1 | G2 | G3 | G4 |
|:---|:---:|:---:|:---:|:---:|:---:|
| **Facility Needs strip** | collapsed | visible | **promoted** | promoted | promoted |
| **Site layout preview** | **hidden** | collapsed (placeholder) | collapsed (body on expand) | collapsed | collapsed |

Specs: [`design_aps_facility_needs_v1.md`](design_aps_facility_needs_v1.md) · [`design_aps_site_preview_v1.md`](design_aps_site_preview_v1.md).

---

## 13. Sign-off amendment (facility IA)

| Role | Verdict | Date |
|:---|:---|:---|
| `@designer` | **PASS** — §12 facility surfaces | 2026-06-18 |
