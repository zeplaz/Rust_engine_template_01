# PLAN-MODULE-KIT-PRODUCTION-TIER-001 — Retire greybox-as-production `v1`

| Field | Value |
|:---|:---|
| **Program ID** | **PLAN-MODULE-KIT-PRODUCTION-001** |
| **Parent** | [`design_procedural_module_kit_v1.md`](design_procedural_module_kit_v1.md) · [`plan_mcp_pipeline_recovery_and_agent_fleet_v1.md`](plan_mcp_pipeline_recovery_and_agent_fleet_v1.md) |
| **Owner** | `@planner` (cross-lane) · art phases `@planner-mcp` |
| **Status** | **SIGNED — effective 2026-06-02** |
| **Date** | 2026-06-02 |
| **Trigger** | User directive: stop promoting pipeline smoke as permanent module inventory |

---

## Summary

`kit_greybox_001`–`003` were **MCP spine smoke** (literal `primitive_cube_add` bpy ops). They were promoted into `assets/configs/buildings/_module_index.ron` without a `development_tier` flag, without PBR, and with geometry cheats (`greybox:*`, pitched roof as flat slab). That created **permanent art debt** referenced by PROC-PG-2 and Bevy registry work.

This plan **separates smoke from production**, defines **minimum geometry + material bars**, migrates the index, and schedules **correct-first** module batches — not more greybox.

**Effective immediately:** **Stop `kit_greybox_004+`.** New batches use `kit_lod0_*` or `kit_production_*` only.

---

## Order critique

| Question | Verdict |
|:---|:---|
| Was greybox wrong for pipeline bring-up? | **No** — smoke proved G0–G5 |
| Was greybox wrong **in `_module_index.ron` as StylePack art**? | **Yes** |
| Can we texture cubes later and call it done? | **No** — silhouette lies (sawtooth ≠ slab); non-canonical IDs |
| Block PG-2 until 50 production modules? | **No** — PG-2 uses **lod0/production** or engine fallback; **never smoke** |
| Skip validation? | **No** — `validate_asset_report` tier rules + validation-first skill |

---

## Tier policy (authoritative)

| Tier | Purpose | Silhouette | PBR | In `_module_index.ron` | StylePack / PG-2 may load? |
|:---|:---|:---|:---|:---:|:---:|
| **`smoke`** | MCP pipeline harness only | Cube / cheat OK | None | Yes, tagged | **No** |
| **`lod0`** | PG-2 assembly, sim readability | Archetype profile required | Optional; `pbr_status: deferred` allowed | Yes | **Yes** (explicit only) |
| **`production`** | Player-visible StylePack art | Full kit contract | Required (tileable set id) | Yes | **Yes** (default StylePack) |

### Field contract

| Location | Field | Values | Writer |
|:---|:---|:---|:---|
| AssetSpec | `development_tier` | `smoke` \| `lod0` \| `production` | designer-mcp |
| AssetSpec | `pbr_status` | `none` \| `deferred` \| `shipped` | designer-mcp (required when tier = lod0/production) |
| geometry job | `development_tier` | same as spec | copied from spec |
| geometry job | `batch_id` | `kit_smoke_*` \| `kit_lod0_*` \| `kit_production_*` | designer-mcp |
| `_module_index.ron` | `development_tier` | same | library_register |
| `_module_index.ron` | `replaced_by` | optional canonical `module_id` | library_register on re-promote |
| `_module_index.ron` | `stylepack_visible` | bool (default by tier) | derived: `false` if smoke, else `true` |

**Batch naming (hard rule):**

| Prefix | Tier default | New work allowed? |
|:---|:---|:---:|
| `kit_greybox_*` | smoke (legacy) | **No** — frozen |
| `kit_smoke_*` | smoke | Harness only |
| `kit_lod0_*` | lod0 | Yes |
| `kit_production_*` | production | Yes |

**StylePack rule:** resolve modules with `development_tier != smoke` AND `stylepack_visible != false`. Smoke rows are **witness-only** inventory.

**validation-first:** `validate_asset_report` must fail promotion when tier rules violated — see [`.cursor/skills/validation-first/SKILL.md`](../.cursor/skills/validation-first/SKILL.md).

---

## Resolved open questions

| # | Question | Decision | Rationale |
|:---:|:---|:---|:---|
| 1 | **Migrate vs delete** smoke GLBs | **Migrate in index only** — keep files under `assets/models/modules/` | Preserves MCP witnesses; zero disk churn |
| 2 | **PG-2 fallback** | **Dual path:** (a) engine primitive / hide when no lod0+ row; (b) parallel **`kit_lod0_001`** (5 modules, real profiles) | Unblocks PG-2 without waiting for 50 production modules |
| 3 | **PBR deferral** | **`lod0`:** `pbr_status: deferred` OK. **`production`:** promote blocked until `pbr_status: shipped` OR Material Maker lane SHIPPED with witness | Matches kit § textures; no fake production |
| 4 | **ID rename** | **Re-promote under canonical ids only** — no alias in StylePack. Optional `replaced_by` on smoke row → canonical id. Non-canonical smoke ids remain until replaced | Inventory §191–211 is sole catalog; e.g. `win_industrial_3u` not `window_industrial_1u` |

---

## Current state (problem)

| Artifact | Issue |
|:---|:---|
| `module_*.py` bpy | Single axis-aligned cube per op |
| `kit_greybox_001`–`003` | In index **without** tier — treated as StylePack art |
| `validate_glb.py` | Header + verts only — cubes pass |
| Kit § validation | PBR + silhouette — **not enforced** |
| PG-2 index doc | Implied greybox extract OK — **revoked** |

---

## Target architecture

```text
AssetSpec (development_tier + canonical module_id)
  → geometry_run_job (profile matches archetype)
  → validate_asset_report (tier-aware)
  → G4 designer-mcp (silhouette + sim read)
  → promote (blocked if smoke masquerading as production)
  → library_register (tier + stylepack_visible)
  → Bevy: StylePack filters smoke; PG-2 uses lod0+ only
```

---

## Phase schedule (A–E)

### Phase A — Policy + index hygiene (`@planner`, **DONE this doc**)

| ID | Task | Owner | Acceptance |
|:---|:---|:---|:---|
| A1 | Sign production tier plan | planner | This file **SIGNED** |
| A2 | Amend module kit LOD + validation tier language | planner | `design_procedural_module_kit_v1.md` v1.3 |
| A3 | PG-2 mesh authority in growth index | planner | `construction_procedural_growth_index_v1.md` v1.4 |
| A4 | Stop `kit_greybox_004+` in MCP runbook | orchestrator-mcp | `mcp_art_pipeline_agent.md` |
| A5 | Index migration spec | planner-mcp | § Schema annex below; coder-mcp implements B1 |

**Migration command (after B1):** `library-register --rebuild-all` with tier inference: `batch_id` starts with `kit_greybox` → `development_tier: smoke`, `stylepack_visible: false`.

### Phase B — Validator + schema (`@coder-mcp`, 3–5 days)

| ID | Task | Acceptance |
|:---|:---|:---|
| B1 | `development_tier`, `pbr_status`, `stylepack_visible`, `replaced_by` in index + AssetSpec + `geometry_job_v1` | schema + RON emit |
| B2 | `validate_asset_report` tier rules | 24-vert cube fails pitched/sawtooth/arched; `greybox:*` → smoke only |
| B3 | `promote_staging_module` rejects `tier=smoke` unless `--allow-smoke` + witness | |
| B4 | `validate-report` blocks `batch_id` matching `kit_greybox_*` for new jobs | |
| B5 | `error_signatures.json` tier entries | validation-first |

### Phase C — bpy geometry profiles (`@coder-mcp`, 2–3 weeks)

| Op | Profile param | Minimum |
|:---|:---|:---|
| `module_wall` | `panel: flat \| recess` | Not single scale cube |
| `module_roof` | `profile: flat \| pitched \| shed \| sawtooth` | Extrude; **no** `greybox:pitched_via_slab` |
| `module_door` | frame + leaf | |
| `module_window` | frame + mullion + glass | |
| `module_prop` | `prop_kind` | Kind-specific mesh |

≤2k tris LOD0; seed required.

### Phase D — Designer batches (`@designer-mcp`)

| Batch | Tier | Modules | Notes |
|:---|:---|:---|:---|
| `kit_lod0_001` | lod0 | 5 | walls×2, `roof_pitched_gable`, `door_residential`, `win_single_1u` — canonical IDs |
| `kit_lod0_002` | lod0 | 5 | next category slice — see [`plan_kit_lod0_roadmap_v1.md`](plan_kit_lod0_roadmap_v1.md) |
| `kit_lod0_003`–`010` | lod0 | 35 | wave roadmap **SIGNED** — same doc |
| `kit_production_001` | production | 5 | walls only; PBR set id when lane ships — **DEFER** |

**Forbidden:** `greybox:*` references as final geometry excuse; non-canonical `asset_id`.

### Phase E — Bevy registry (`@coder`, after first `kit_lod0_001` G5)

| ID | Task | Acceptance |
|:---|:---|:---|
| E1 | `ProceduralModuleRegistry` filters by tier | API `modules_for_stylepack()` excludes smoke |
| E2 | PG-2 extract uses lod0+ only | test + witness |
| E3 | Missing mesh → primitive/hide — **not** smoke row | documented in proc plan |

---

## Gate realignment (G0–G5)

| Gate | Production tier addition |
|:---|:---|
| G0 | Rules audit rejects greybox-as-final |
| G1 | `module_id` ∈ kit inventory §191–211; `development_tier` set |
| G2 | `batch_id` not `kit_greybox_*` for new work |
| G3 | `validate_asset_report` tier green |
| G4 | Designer confirms silhouette + sim read |
| G5 | Index row has tier; smoke not StylePack-visible |

---

## Planner-MCP annex — schema + validator rules (art-only)

**For `@planner-mcp` implementation detail** — coder-mcp executes in Phase B.

### `asset_spec_v1` additions (v1.1)

```json
{
  "development_tier": "lod0",
  "pbr_status": "deferred",
  "module_id_alias_of": null
}
```

- `asset_id` must equal canonical `module_id` when `development_tier` is `lod0` or `production`
- `references` containing `greybox:` → forces tier `smoke` at validate time

### `geometry_job_v1` additions (v1.2)

```json
{
  "development_tier": "lod0",
  "batch_id": "kit_lod0_001",
  "params": {
    "seed": 42,
    "profile": "pitched"
  }
}
```

### Validator rules (`validate_asset_report` extension)

| Rule ID | Condition | Result |
|:---|:---|:---|
| TIER-001 | `development_tier` missing on batch `kit_production_*` | error |
| TIER-002 | `vertex_count <= 32` AND archetype in roof/window AND `profile` expects non-box | error `SilhouetteInsufficient` |
| TIER-003 | `references` has `greybox:` AND tier != smoke | error |
| TIER-004 | tier=production AND `pbr_status` not shipped | error (when MM lane closed) |
| TIER-005 | `module_id` not in kit inventory whitelist | warning → error for production |
| TIER-006 | `batch_id` matches `kit_greybox_*` AND job created after 2026-06-02 | error `BatchRetired` |

### Promotion gate

- Default: `promote_staging_module` reads sidecar tier; refuses smoke→index unless `--allow-smoke`
- `library_register` sets `stylepack_visible: false` for smoke

---

## Delegation — active sprint

| Order | Agent | Goal | Gate |
|:---:|:---|:---|:---|
| 1 | `@coder-mcp` | Phase B1–B5 tier schema + validators | G3 |
| 2 | `@designer-mcp` | `kit_lod0_001` specs (5 canonical modules) | G0–G1 |
| 3 | `@coder-mcp` | Phase C profiles for wall + roof pitched | G2 |
| 4 | `@coder` | Phase E1 registry filter (after lod0 promote) | G5 engine |
| — | `@orchestrator-mcp` | Block `kit_greybox_004+`; queue `kit_lod0_001` | HANDOFF |

**Do not assign:** `kit_greybox_004`, `kit_greybox_005`, or any batch expanding smoke inventory.

---

## Changelog

| Version | Date | Notes |
|:---|:---|:---|
| v1.0.0 | 2026-06-02 | Draft |
| v1.1.0 | 2026-06-02 | **SIGNED** — tier policy, OQs resolved, planner-mcp annex, phase A doc updates |
