# APS Facility Needs strip `v1.1` — schema-aligned

| Field | Value |
|:---|:---|
| **ID** | **DES-APS-FACILITY-NEEDS-001** |
| **Program** | PLAN-INDUSTRIAL-FACILITY-GRAMMAR-001 · Track E3-A |
| **Date** | 2026-06-18 (amend **v1.1** — schema bind) |
| **Owner** | `@designer` |
| **Binding schema** | [`design_facility_binding_schema_v1.md`](design_facility_binding_schema_v1.md) · [`facility_binding_v1.schema.json`](../../tools/mcp/schemas/facility_binding_v1.schema.json) |
| **Brief authority** | `grammar_facility_brief()` → [`grammar_facility_brief_live.json`](../../debug_runs/grammar_facility_brief_live.json) |
| **Implementation** | [`facility_needs_strip.py`](../../tools/mcp/art_pipeline_suite/facility_needs_strip.py) (CMCP-FACILITY-NEEDS-PANEL-001) |
| **Depends** | [`design_power_tier_bands_v1.md`](design_power_tier_bands_v1.md) · [`design_aps_grammar_tier_exposure_v1.md`](design_aps_grammar_tier_exposure_v1.md) §12 |
| **Verdict** | **PASS** — signed against `facility_binding_v1` |

```text
DES-APS-FACILITY-NEEDS-001 Q✓
Strip renders grammar_facility_brief only — never grammar power_tier or catalog_id alone
```

---

## 0. Purpose

Read-only **Facility Needs** strip on Assembly tab: answers *what does this archetype need?* from the join:

```text
grammar.facility_binding  →  catalog JSON  →  industrial_supply_chains.json
         (Layer 2 ref)            (Layer 3 authority)
```

**Not:** editable fields · chain picker · site zone editor.

---

## 1. Placement

```text
┌─ Grammar tier chip ─────────────────────────────────────────────┐
│ G2 — axis coverage                                               │
├─ Facility Needs strip ───────────────────────────────────────────┤
│ ⚡ light power — Concrete batching plant · Concrete (Portland)    │
│ In: Cement, Water, Gravel  ·  Out: Concrete                      │
│ catalog: concrete_mixer_plant.json                               │
│ site: concrete_mixer_plant_site_v0 · storage med · loading high  │
├─ Archetype row ──────────────────────────────────────────────────┤
│ Building type [▼]   District [▼]   [Generate Assembly]           │
└──────────────────────────────────────────────────────────────────┘
```

**Pack order:** below `_grammar_tier_strip` · above kit hint / archetype row.

**Widget:** `FacilityNeedsStrip` (`ttk.Frame`) · reserved height 48–88px (no layout jump).

**Landscape lane:** hidden.

---

## 2. Tier exposure

| Grammar tier | Lines shown | Empty binding |
|:---|:---|:---|
| **G0** | Line 1 only | `○ Visual-only grammar — no process binding` |
| **G1** | Lines 1–2 | same |
| **G2+** | Lines 1–4 (site/axes when present) | same |

Matrix: [`design_aps_grammar_tier_exposure_v1.md`](design_aps_grammar_tier_exposure_v1.md) §12.

---

## 3. Schema → strip mapping (`facility_binding_v1`)

| Schema field | Required | Strip uses | Display rule |
|:---|:---:|:---|:---|
| `catalog_id` | yes | Line 3 · brief path | `catalog: {catalog_id}.json` — click opens folder |
| `chain_id` | yes | **never raw** | Human name via `brief.chain.display_name` only |
| `supply_chain_role` | yes | Line 1 detail | Step label table §4.1 — must match catalog + chain step |
| `power_tier` | yes | **not displayed from binding** | Display `brief.derived.power_tier_from_catalog` only |
| `site_template_id` | no | Line 4 | `site: {site_template_id}` when set (G2+) |
| `program_axes` | no | Line 4 suffix | `storage {lvl} · loading {lvl} · …` abbreviated (G3+) |

**Forbidden on grammar (schema §3):** `power_consumption`, `produces`, `consumes` — strip loads via brief `catalog` / `io_summary` only.

### 3.1 Validator alignment (brief join)

| Check | Brief field | Strip when fail |
|:---|:---|:---|
| Catalog exists | `errors[]` | `✗ Catalog missing — {catalog_id}` |
| Chain step match | `errors[]` | `✗ Chain mismatch — {first error}` |
| Role tripartite match | `errors[]` | `✗ Role mismatch — binding vs catalog` |
| Power tier match | `derived.power_tier_binding_match` | `◐ Tier drift — fix grammar power_tier` |
| Join green | `green` | normal render when `true` |

**Display tier authority:** always `derived.power_tier_from_catalog` — never `facility_binding.power_tier` in UI even when matching.

---

## 4. Brief JSON contract (`grammar_facility_brief`)

**Loader:** `grammar_facility_brief.grammar_facility_brief(grammar_id=…)` → `body["brief"]`.

### 4.1 Required brief paths for render

| Path | Type | Strip line |
|:---|:---|:---|
| `facility_binding` | object | gate — null → empty state |
| `catalog.catalog_id` | string | line 3 |
| `catalog.power_consumption` | number | tooltip only `(28 units)` |
| `derived.power_tier_from_catalog` | enum | line 1 glyph via `power_tier_atom` |
| `chain.display_name` | string | line 1 suffix (process buildings) |
| `io_summary.consumes_top3` | string[] | line 2 In |
| `io_summary.produces_top3` | string[] | line 2 Out |
| `derived.site_template_id` | string? | line 4 |
| `derived.program_axes` | object? | line 4 suffix |

### 4.2 Reference fixture (`factory_cluster_v1`)

Witness row in [`grammar_facility_brief_live.json`](../../debug_runs/grammar_facility_brief_live.json):

```json
{
  "facility_binding": {
    "catalog_id": "concrete_mixer_plant",
    "chain_id": "concrete_portland",
    "supply_chain_role": "concrete_mixer",
    "power_tier": "light",
    "site_template_id": "concrete_mixer_plant_site_v0",
    "program_axes": { "storage": "medium", "loading": "high", "office": "low" }
  },
  "derived": { "power_tier_from_catalog": "light", "power_tier_binding_match": true }
}
```

**Expected strip (G2+):**

```text
⚡ light power — Concrete batching plant · Concrete (Portland)
In: Cement, Water, Gravel  ·  Out: Concrete
catalog: concrete_mixer_plant.json
site: concrete_mixer_plant_site_v0 · storage med · loading high · office low
```

### 4.3 Step labels (`supply_chain_role` → detail)

| Role | Line 1 detail |
|:---|:---|
| `aggregate_mine` | Aggregate quarry |
| `cement_kiln` | Cement kiln |
| `concrete_mixer` | Concrete batching plant |
| `integrated_plant` | Integrated cement plant (legacy) |
| `bauxite_mine` | Bauxite mine |
| `alumina_refinery` | Alumina refinery |
| `aluminum_smelter` | Aluminum smelter |
| `aluminum_fabrication` | Aluminum fabrication |

**Utility** (`catalog.utility_role`): `Substation yard` · `Transformer pad` · `Coal power plant` — **omit** `chain.display_name` on line 1.

### 4.4 Line 1 format (canonical)

```python
tier = brief["derived"]["power_tier_from_catalog"]
detail = step_label(brief)  # §4.3
glyph, word, fg, _bg = power_tier_atom(tier, detail=detail)
# Process:  f"{glyph} {word} · {chain.display_name}"
# Utility:  f"{glyph} {word}"
```

**Example:** `⚡⚡ medium power — Cement kiln · Concrete (Portland)`

### 4.5 Line 2 — I/O

```text
In: {consumes_top3 joined}  ·  Out: {produces_top3 joined}
```

Source: `brief.io_summary` only (catalog-truncated top 3). Full lists in tooltip.

### 4.6 Line 3 — catalog link

```text
catalog: {catalog_id}.json
```

Click → OS folder · status `✓ opened catalog folder`.

### 4.7 Line 4 — site + program axes (G2+ binding optional fields)

| Part | When | Format |
|:---|:---|:---|
| Site | `derived.site_template_id` set | `site: {id}` — links site preview ([`design_aps_site_preview_v1.md`](design_aps_site_preview_v1.md)) |
| Axes | `derived.program_axes` set (G3+) | `storage med · loading high · office low` — abbrev `low/med/high` |

Omit line 4 entirely when both absent.

---

## 5. Empty & loading states

| Condition | Copy | Atom |
|:---|:---|:---:|
| No `facility_binding` on grammar | `○ Visual-only grammar — no process binding` | ○ |
| `gaps` contains `no facility_binding` | same | ○ |
| `errors` non-empty | `✗ {errors[0]}` | ✗ |
| `power_tier_binding_match == false` | `◐ Tier drift — update grammar power_tier to {derived}` | ◐ |
| Loading | `⟳ Loading facility brief…` | ⟳ |
| `catalog` null | `✗ Catalog missing — {catalog_id}` | ✗ |

Use `apply_status_atom` / `power_tier_atom` fg — not ad-hoc hex.

---

## 6. Refresh & debounce

| Event | Action |
|:---|:---|
| Archetype combo change | `grammar_facility_brief(grammar_id)` |
| District change | no reload |
| Assembly tab focus | reload if grammar file mtime changed |
| Lane ≠ Buildings | `pack_forget` strip |

Debounce: **150ms** on rapid archetype scroll.

---

## 7. Tokens

| Element | Token |
|:---|:---|
| Line 1 | `FONT_UI` · fg from `power_tier_atom` |
| Line 2 | `FONT_SMALL` · `COLOR_TEXT_SUBTLE` |
| Lines 3–4 | `FONT_SMALL` · `COLOR_MUTED` / `COLOR_ACCENT` on link |
| Background | `COLOR_EXPLAINER_BG` |

---

## 8. Witness / scanner fields

```json
{
  "facility_needs_visible": true,
  "facility_needs_mode": "promoted",
  "binding_present": true,
  "brief_green": true,
  "catalog_id": "concrete_mixer_plant",
  "chain_id": "concrete_portland",
  "power_tier_displayed": "light",
  "power_tier_binding_match": true,
  "site_template_id": "concrete_mixer_plant_site_v0",
  "binding_schema": "facility_binding_v1.schema.json"
}
```

**Gate:** CMCP strip Q✓ requires `brief_green` for bound archetypes in quality loop.

---

## 9. Implementation checklist (CMCP)

| # | Requirement | File |
|:---:|:---|:---|
| 1 | Load brief not raw grammar I/O | `facility_needs_strip.py` |
| 2 | `power_tier_from_catalog` only | `aps_inline_feedback.power_tier_atom` |
| 3 | Lines 1–3 per tier matrix §2 | `FacilityNeedsStrip.refresh` |
| 4 | Line 4 site + axes G2/G3 | *wire if missing* |
| 5 | Show `errors[0]` / tier drift | *wire if missing* |
| 6 | Wire into `assembly_panel` below tier chip | `assembly_panel.py` |

---

## 10. Sign-off

| Role | Verdict | Date |
|:---|:---|:---|
| `@designer` | **PASS** — aligned to `facility_binding_v1` + brief witness | 2026-06-18 |

**Unblocks:** CMCP-FACILITY-NEEDS-PANEL-001 completion (lines 4 + error rows) · operator facility rubric
