# APS default presence audit `v1` — plan alignment + expansion corrections

| Field | Value |
|:---|:---|
| **ID** | **DES-APS-DEFAULT-PRESENCE-AUDIT-001** |
| **Date** | 2026-06-02 |
| **Owner** | `@designer` |
| **For** | `@planner` → `@coder-mcp` (witness refresh) · `@operator` (session dump) |
| **Authority** | [`plan_aps_grammar_evolution_v1.md`](plan_aps_grammar_evolution_v1.md) · [`design_aps_grammar_tier_exposure_v1.md`](design_aps_grammar_tier_exposure_v1.md) · [`design_landscape_lg5_expansion_matrix_v1.md`](design_landscape_lg5_expansion_matrix_v1.md) |
| **Verdict** | **PASS (qualified)** — content tier **G3** on disk; guard/witness layer stale |

```text
DES-APS-DEFAULT-PRESENCE-AUDIT-001 Q✓
Live APS cold start = G3 family · not G0 pilot — plans + witnesses must catch up
```

---

## 0. Executive summary (for planner)

**What the artist actually gets today** when opening APS with repo defaults:

| Signal | Live value (2026-06-02) | Stale docs still say |
|:---|:---|:---|
| **Grammar tier (`grammar_set_tier()`)** | **G3** — layer depth | G0 pilot singleton |
| **Archetypes in combo** | 4 — CivicBlock, FactoryCluster, IndustrialWarehouse, RailEdge | 1 — IndustrialWarehouse |
| **District count** | 5 | 1 |
| **Kit hint strip** | **hidden** (correct for G3) | visible in `aps_grammar_tier_gates_live.json` |
| **DNA + Iterate panels** | **visible** (G3 matrix) | hidden in tier-gates witness |
| **Set health strip** | **promoted** (G2+) | not in G0 witness |
| **G4 ship path** | **blocked** — coverage/parity guards red | brief alone says “gaps: none” |

**Designer ruling:** Tier **exposure** specs in [`design_aps_grammar_tier_exposure_v1.md`](design_aps_grammar_tier_exposure_v1.md) are still valid — implement **G3**, not G0, on cold start. **Content** tables in [`plan_aps_grammar_evolution_v1.md`](plan_aps_grammar_evolution_v1.md) § maturity model and witness examples are **wrong** and must be amended (planner-owned plan edit).

---

## 1. Cold-start presence map (what shows up)

### 1.1 Always (all tiers) — unchanged

| Surface | Artist sees | OK? |
|:---|:---|:---:|
| First-run onboarding | 5-step Catalog→Atlas card ([`aps_uiux_onboard.py`](../../tools/mcp/python/rust_engine_mcp/aps_uiux_onboard.py)) | ✓ |
| Pipeline spine | Assembly step active on Buildings tab | ✓ |
| Generate row | Building type + district (human labels) | ✓ |
| Footprint grid + placements | Empty until Generate | ✓ |
| Material library | Populated from repo profiles | ✓ |
| Grammar inspector | Collapsed | ✓ |
| Manual fallback | Collapsed | ✓ |

### 1.2 G3-specific (current default) — must match exposure matrix

| Surface | Expected @ G3 | Coder check |
|:---|:---|:---|
| **Tier chip** | `G3 — layer depth` | `_grammar_set_tier_var` |
| **Archetype combo** | 4 human labels (not engineer ids) | `archetype_combo_count == 4` |
| **Kit hint** | **hidden** | `kit_hint_visible == false` |
| **Shape bias / DNA** | **visible** (expanded) | `dna_panel_visible == true` |
| **Iterate grammar** | **visible** (expanded) | `iterate_panel_visible == true` |
| **Set health strip** | **promoted** — brief line or gap | strip packed before next-step |
| **Build-set / sweep** | Collapsible **expanded** default @ G2+ | `build_set_expanded_default == true` |
| **Next-step line** | `Tune shape bias; inspect the rule chain after Generate.` | spine G2+ copy |
| **Pack atlas spine step** | May gray — tier `< G4` | not silent fail |

### 1.3 Onboarding copy vs G3 reality — correction

| Onboarding / empty state | Current copy | Designer correction |
|:---|:---|:---|
| Assembly empty | `No assembly yet — Generate one to begin.` | **Add tier-aware tail @ G2+:** `…then tune shape bias in the panels below.` |
| Onboarding step 3 | `Combine module + materials into one saved building.` | **Keep** — still true; do not mention DNA in first-run card (onboarding stays tier-agnostic). |
| Kit hint (G0 only) | Code: `Only one building type in the kit…` | Align to §10.1 spec: `One building type in the kit for now — add grammar files under assets/configs/buildings/grammars/ to unlock more building types.` (G0 only — never shown @ G3) |

---

## 2. Plan corrections (grammar evolution)

### 2.1 Amend [`plan_aps_grammar_evolution_v1.md`](plan_aps_grammar_evolution_v1.md)

| Section | Was | **Correct to** |
|:---|:---|:---|
| § maturity “Example today” | G0 — IndustrialWarehouse only | **G3** — 4 archetypes · 4 RON files · DNA/iterate visible |
| § APS exposure intro | “one building grammar” | **Four** building grammars; UI tier from `grammar_set_tier()` not file count |
| Witness APS-GRAM-TIER-001 example JSON | `tier: G0`, `archetype_count: 1` | **`tier: G3`, `archetype_count: 4`** + `reasons` for G4 blockers |

### 2.2 Amend [`plan_aps_grammar_evolution_witness_v1.md`](plan_aps_grammar_evolution_witness_v1.md)

| Rule | Correction |
|:---|:---|
| Anti-fake-green #3 | “G1 forbidden until ≥3 RON” → **met**; refresh `grammar_archetype_g1_live.json` |
| Anti-fake-green #2 | Tier gates witness must use **`refresh_grammar_tier_from_registry()` tier**, not hardcoded G0 test fixture |
| APS-GRAM-TIER-002 JSON | Must match live tier snapshot — add `live_tier` vs `fixture_tier_g0` split (see §4) |

### 2.3 Do **not** change

| Doc | Why |
|:---|:---|
| [`design_aps_grammar_tier_exposure_v1.md`](design_aps_grammar_tier_exposure_v1.md) §2 matrix | G0–G4 columns still authoritative |
| [`design_aps_onboard_spec_v2.md`](design_aps_onboard_spec_v2.md) | 5-tab spine still correct |
| Preview ladder P0–P4 | Tier coupling unchanged |

---

## 3. Expansion corrections

### 3.1 Building grammar G3 → G4 (not “expand archetypes”)

**Artist-visible story @ G3:**

```text
You have a family of building types and shape controls.
Ship check / atlas pack stays cautious until set coverage guards go green.
```

**G4 blockers (disk — inform Set health / atlas gray, not kit hint):**

| Guard | Live symptom | Artist-facing copy (Set health / toast) |
|:---|:---|:---|
| `building_set_coverage_report` | `grammar_pilots: 0` per set · hardcode lint violations | `Set health: building-set manifests need pilot links — ask coder-mcp.` |
| `grammar_pilot_parity` | `grammar_pilots=0 need ≥4` | `Set health: grammar pilot registry out of sync with brief.` |

**Designer correction:** [`grammar_set_brief`](../../tools/mcp/python/rust_engine_mcp/grammar_build_set.py) reports **green / 4 pilots** while coverage/parity report **0 pilots** — **contradictory**. UI must **not** show “Set health: OK” when G4 guards fail. Prefer **brief gaps line** when `grammar_set_tier().reasons` non-empty:

```text
Set health: not ready for ship — building-set coverage and pilot parity must match brief.
```

**Planner route:** single coder-mcp slice **APS-GUARD-BRIEF-PARITY-001** — one pilot-count authority; then refresh G4 witnesses.

### 3.2 Landscape LG-5 expansion matrix — still valid, separate lane

[`design_landscape_lg5_expansion_matrix_v1.md`](design_landscape_lg5_expansion_matrix_v1.md) **unchanged** (16 cells · 5 topologies · scar/burn/regrowth rows).

| Correction | Detail |
|:---|:---|
| **Do not merge** with building G3 tier chip | Landscape Variants tab uses **state axis** banner — keep separate from Buildings grammar tier |
| **Atlas pack @ G4 building** | Building atlas ship ≠ landscape LG-5 bake — spine labels must say **domain** on Pack step |
| **Pilot reuse row** | First 3 clean cells reuse pilot PNGs — designer-mcp bake order unchanged |

### 3.3 APS “expansion” UX (tier unlocks) — what artist should **not** see @ G3

| Forbid | Reason |
|:---|:---|
| Kit hint “one building type” | Fake G0 @ G3 |
| Five grammar panels full-width @ launch | Anti-pattern — max **4** expanded @ G3 |
| Engineer ids in combos | RailEdge ok as label mapping only via `human_label()` |
| “Gaps: none” in Set health when tier reasons cite G4 | Contradiction |

---

## 4. Session debug dump (operator + coder-mcp)

**Problem:** Planner and designer cannot reconcile “what APS showed” vs “what guards say” without a **single boot snapshot**.

### 4.1 When to dump

| Trigger | Who |
|:---|:---|
| After `pytest -k aps` (CI) | coder-mcp — **split fixture vs live** |
| On APS launch (after `refresh_grammar_tier_from_registry`) | coder-mcp hook — optional `APS_DUMP_PRESENCE=1` |
| Before operator pixel walk / planner review | **operator** — manual CLI |
| On APS exit (File→Quit or window close) | coder-mcp — write-if-changed |

### 4.2 One-shot operator command (today — no new code)

From repo root:

```powershell
cd C:\dev\github\Rust_engine_template_01
$env:APS_TEST_HEADLESS = "1"
python -m rust_engine_mcp.cli grammar-set-brief --write-witness
python -m rust_engine_mcp.cli grammar-set-tier --write-witness
python -m rust_engine_mcp.cli building-set-coverage --write-witness
python -c "from rust_engine_mcp import grammar_build_set; import json; print(json.dumps(grammar_build_set.grammar_pilot_parity(), indent=2))" | Out-File debug_runs/_grammar_pilot_parity_scratch.json
pytest tools/mcp/python/tests/test_aps_grammar_tier_gates.py::test_refresh_grammar_tier_from_registry_matches_api -q
```

**Attach to planner:** `grammar_set_tier_live.json`, `grammar_set_brief_live.json`, `aps_grammar_tier_gates_live.json` (after test fix), coverage witness if present.

### 4.3 Target bundled witness (coder-mcp — **DES-APS-SESSION-DUMP-001**)

**Path:** `debug_runs/aps_session_presence_live.json`

```json
{
  "gate": "DES-APS-SESSION-DUMP-001",
  "grammar_set_tier": { "tier": "G3", "archetype_count": 4, "reasons": ["…"] },
  "grammar_set_brief": { "green": true, "gaps": [] },
  "g4_guards": {
    "building_set_coverage_green": false,
    "grammar_pilot_parity_green": false
  },
  "ui_presence": {
    "tier_chip": "G3 — layer depth",
    "kit_hint_visible": false,
    "dna_panel_visible": true,
    "iterate_panel_visible": true,
    "set_health_visible": true,
    "archetype_combo_count": 4,
    "default_archetype_label": "…",
    "default_district_label": "…"
  },
  "onboarding_seen": false,
  "expansion": {
    "building_g4_blocked": true,
    "landscape_lg5_matrix_cells": 16,
    "landscape_lane_active": false
  },
  "sources": ["grammar_set_tier()", "grammar_tier_gate_snapshot()", "grammar_set_brief()"]
}
```

**CLI sketch (planner seeds):** `python -m rust_engine_mcp.cli aps-session-presence-dump --write-witness`

**Test rule:** `ui_presence.tier` **must equal** `grammar_set_tier.tier` — else **WIT-HON fail**.

### 4.4 Split fixture vs live in CI

| Witness | Purpose |
|:---|:---|
| `aps_grammar_tier_gates_g0_fixture_live.json` | Proves G0 matrix wiring (`apply_grammar_tier("G0")`) |
| `aps_grammar_tier_gates_live.json` | **Live** — after `refresh_grammar_tier_from_registry()` |

Stop overwriting live file with G0 fixture in `test_write_aps_grammar_tier_gates_witness`.

---

## 5. Planner routing package

See [`planner_routing_aps_presence_v1.md`](planner_routing_aps_presence_v1.md).

---

## 6. Sign-off

| Role | Verdict | Date |
|:---|:---|:---|
| `@designer` | **PASS (qualified)** — exposure matrix holds; plan/witness/content guard drift documented | 2026-06-02 |

**Qualified until:** `aps_session_presence_live.json` green · tier-gates live witness matches G3 · brief/coverage pilot-count unified.
