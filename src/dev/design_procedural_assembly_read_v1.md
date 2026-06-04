# DESIGN-PROC-ASSEMBLY-READ-001 — Procedural assembly player read `v1`

| Field | Value |
|:---|:---|
| **Queue ID** | **MCP-DUX-PG2-001** |
| **Fleet** | [`mcp_fleet_wave3_engine_orders_v1.md`](mcp_fleet_wave3_engine_orders_v1.md) |
| **Parent** | [`design_procedural_module_kit_v1.md`](design_procedural_module_kit_v1.md) · [`design_construction_site_stage_read_v1.md`](design_construction_site_stage_read_v1.md) (CON-P2) |
| **Coder exec** | [`plan_procedural_build_gen_exec_001_v1.md`](plan_procedural_build_gen_exec_001_v1.md) § PG-2 · witness `debug_runs/procedural_assembly_live.json` |
| **Version** | `1.0.0` |
| **Date** | 2026-06-02 |
| **Owner** | `@designer` |
| **Verdict** | **PASS** |
| **Unblocks** | MCP-PG-2-WIT designer sign-off · PG-2 UX acceptance |
| **No Rust · No MCP tools** | Charter + rubric only |

---

## Purpose

Players must **read district character from assembled buildings** before production PBR lands. PG-2 wires footprint **W/D/C** grammar → **StylePack** slot → **lod0 GLB**. This doc defines:

1. What players **can** distinguish at lod0 today (and what they cannot).
2. How **StylePack swap** reads on the **same footprint** (Victorian vs industrial west).
3. Failure presentation when a slot has no lod0 module (`hide_slot` policy).
4. **Sign-off rubric** for post-`procedural_assembly_live.json` review.
5. **Quality ladder** — why current modules feel simple, greybox is retired from player view, and how materials/style variation improve.

**Inputs (Wave 2 closed):** 7 style pack RON files · 50 canonical lod0 modules · W/D/C grammar · registry tier filter (smoke excluded).

---

## Player mental model

```text
Footprint (W/D/C grid)  →  StylePack picks module_id per slot  →  lod0 mesh + material_profile
         ↑                           ↑                                    ↑
   "how big / where door"    "which district family"           "silhouette + tint family"
```

At **lod0**, players answer: *What kind of building is this?* (residential brick vs factory steel vs bunker concrete) — **not** *What exact brick texture is this?*

At **production**, players answer: *Does this feel built and worn?* (tileable PBR, variation, clutter density).

Construction **phase** read (Planned → Operational) stays separate — see CON-P2. PG-2 assembly appears during **UnderConstruction** onward when `RepresentationResult.procedural_module_meshes` is active.

---

## lod0 vs production — player read ladder

| Signal | lod0 (PG-2 now) | production (later) |
|:---|:---|:---|
| **Wall family** | Brick vs steel vs wood vs glass **silhouette** + flat/grey tint from `material_profile` when embedded | Tileable albedo/normal/roughness; hue/wear variation |
| **Roof profile** | Gable vs hip vs sawtooth vs shed vs flat **shape** readable at tactical zoom | Same profile + roof material sets (tile, metal, membrane) |
| **Door scale** | Residential 1u vs warehouse/gate **width** + frame opening (not solid cube) | Panel detail, hardware, glass reflectance |
| **Window type** | Single / double / strip / arched / slit **bay grammar** | Mullion depth, curtain wall transparency |
| **Props / corners** | L-corner, vent, tank, chimney **placement** on roof/edge | Higher poly clutter; emissive on `prop_light` |
| **District swap** | **Different module_ids** per StylePack on same W/D/C grid | Same swap + richer per-pack material instances |

**Honest expectation:** Most lod0 rows today use `pbr_status: deferred` — players get **correct archetype shape** and **material family hint** (engine grey tint or partial pilot PBR), not final art. That is sufficient for PG-2 proof (*Victorian brick wall* vs *industrial steel wall* on the same footprint) but **not** sufficient for long-term player delight. Production tier + Material Maker lane closes the gap.

---

## Greybox retirement (player-facing)

| Tier | Player sees? | Status |
|:---|:---:|:---|
| **`smoke`** (`kit_greybox_*`, cheat cubes) | **Never** | Index rows kept for MCP witnesses only; `stylepack_visible: false`; `replaced_by` → canonical lod0 id |
| **`lod0`** (`kit_lod0_*`, 50/50) | **Yes** (PG-2) | Archetype-correct silhouettes; PBR optional |
| **`production`** (`kit_production_*`) | **Yes** (default target) | Full kit contract + shipped PBR |

**Player rule:** If a mesh looks like a flat cube with wrong proportions, it is **a bug** — not lod0 policy. PG-2 must never fall back to smoke GLBs (`smoke_fallback_used: false` in witness).

**Conversion policy (not player-visible):** Legacy smoke ids (`door_shop_1u`, `window_industrial_1u`, …) remain on disk but are **superseded** by canonical lod0 rows. StylePack always resolves canonical ids. Remaining smoke rows should shrink over time; they must not re-enter StylePack.

---

## StylePack swap — same footprint, different read

**Reference footprint:** 4×3 cells, floor 0 = `W W D W`, floor 1 = `W W W W`, roof = 4×3 **R** plane, one **C** on front-right corner.

| Slot | Victorian (`style_victorian`) | Industrial West (`style_industrial_west`) |
|:---|:---|:---|
| `wall_1u` | `wall_brick_1u` | `wall_steel_1u` |
| `wall_2u` | `wall_brick_2u` | `wall_concrete_2u` |
| `door_default` | `door_residential` | `door_shop` |
| `door_wide` | — | `door_warehouse` |
| `window_1u` | `win_single_1u` | `win_double_1u` |
| `window_2u` | `win_double_1u` | — |
| `window_industrial` | — | `win_industrial_3u` |
| `roof_default` | `roof_pitched_gable` | `roof_sawtooth` |
| `roof_flat` | `roof_flat` | `roof_metal_low` |
| `roof_industrial` | — | `roof_shed` |
| `corner_outer` | `corner_L` | `corner_L` |
| `prop_clutter` | `prop_chimney` | `prop_vent` |

### Player read (tactical camera, lod0)

| Question | Victorian | Industrial West |
|:---|:---|:---|
| **Wall read** | Warm masonry bay; shorter 1u/2u brick modules | Cool steel panel + wide concrete 2u bay |
| **Roof read** | Pitched gable — residential skyline | Sawtooth / shed — factory roofline |
| **Door read** | Narrow residential entry | Shop front + optional wide warehouse leaf |
| **Window read** | Single + double domestic bays | Double + long industrial strip glazing |
| **Clutter read** | Chimney — hearth/residential cue | Roof vent — mechanical/industrial cue |
| **One-line player takeaway** | *"Brick house with a chimney"* | *"Metal factory with sawtooth roof and vents"* |

**Pass condition for PG-2-WIT:** Side-by-side screenshot or witness with **different `module_ids_used`** for wall + roof + clutter between packs; **same** footprint token count.

---

## W/D/C grammar → visible cues

| Token | Player sees | StylePack slot keys |
|:---|:---|:---|
| **W** | Facade bay (wall + optional window) | `wall_1u` / `wall_2u` + window slot by bay width |
| **D** | Ground-floor opening (door) | `door_default` or `door_wide` when bay ≥ 2u |
| **C** | Corner mass / turn | `corner_outer`, `corner_inner`, or `corner_parapet` |
| **R** | Roof silhouette over footprint | `roof_default`, `roof_flat`, `roof_industrial` by usage |
| **.** | Setback / yard | Nothing — intentional void |

**Door on floor 0 only** — upper floors show windows in **W** bays; players infer height from repeating window rhythm.

**Min read footprint:** 2×2. Below that, prefer **hide_slot** over stretching modules.

---

## All seven packs — district one-liners

| StylePackId | Player district read (lod0) |
|:---|:---|
| `style_victorian` | Brick & pitched roof — dense urban residential |
| `style_modern` | Glass curtain + flat roof — office/commercial |
| `style_industrial_west` | Steel, sawtooth, warehouse doors — western factory |
| `style_industrial_soviet` | Heavy concrete panels, gate doors — soviet industrial |
| `style_military` | Bunker walls, slit windows, parapet — hardened gov/military |
| `style_rural` | Wood siding, hip/tile roof, fence props — farm/low density |
| `style_colonial` | Brick civic doors, arched windows, canopy roof — town hall/commercial |

Pack files: `assets/configs/buildings/style_packs/style_*.ron` · manifest: `debug_runs/art_pipeline/style_packs_manifest_live.json`.

---

## Failure modes — player presentation

Policy: **`fallback_policy: hide_slot`** (all 7 packs). **Never smoke.**

| Engine state | Player sees | Accept? |
|:---|:---|:---:|
| Slot resolves to lod0 GLB | Module mesh at snap | Yes |
| Slot module missing / not promoted | **Gap** at that snap (wall hole, no roof cap) | Yes — honest incomplete |
| `smoke_fallback_used` | Cube cheat mesh | **No — witness fail** |
| `primitive_footprint` (future opt-in) | Flat shaded footprint prism | Only if explicitly enabled per pack; not default |

**UX copy (F3 / debug):** `Procedural slot hidden — {slot_key} unresolved for {style_pack_id}`

**Player-facing:** Prefer a **missing bay** over a **wrong-style cube**. Incomplete districts read as *under construction art* until production batch fills gaps.

---

## Quality gap — simple geometry & materials (design stance)

Current lod0 modules are **intentionally minimal** for pipeline proof: low vertex counts (often 24–192 verts), bpy primitives with profile params, **`pbr_status: deferred`** on most rows. Players will notice:

| Issue | Cause | Remediation lane |
|:---|:---|:---|
| Walls feel like flat slabs | lod0 flat profile + deferred PBR | `kit_production_*` + tileable families (`brick_red_01`, `concrete_grey_01`, …) |
| Roofs readable but plain | Correct profile, grey metal tint | Production roof materials + normal detail |
| Props feel like boxes | lod0 prop_kind placeholders | Production prop meshes (tank, transformer, chimney) |
| Styles blur together at distance | Tint-only differentiation | Per-pack **MaterialVariation** (hue/roughness/wear) + distinct silhouettes already in packs |
| "Greybox" vibe | **Smoke** cubes if wrongly loaded — **bug** | Registry tier filter; canonical lod0 only |

**Design priority for art waves after PG-2:**

1. **`kit_production_001`** — ship PBR on high-traffic slots (wall_1u, door_default, roof_default) across Victorian + industrial west first.
2. **Material profile expansion** — 5–8 tileable families with deterministic variation seeds (kit § MaterialVariation).
3. **Silhouette polish** — keep archetype profiles; add recess/panel depth without breaking tier validators.
4. **Smoke index cleanup** — mark all superseded smoke rows `stylepack_visible: false`; do not delete GLBs (witness preservation).

PG-2 **proves assembly**. Production tier **delivers vibe**.

---

## Style variation — what must read without textures

Even at lod0, packs must differ on **shape vocabulary**, not tint alone:

| Vocabulary | Victorian / Rural | Industrial | Military |
|:---|:---|:---|
| Roof | pitched gable / hip / tile | sawtooth / shed / metal low | bunker / parapet |
| Wall | brick / wood | steel / concrete panel | bunker / thick concrete |
| Opening | residential / arched | shop / warehouse / strip | slit / heavy frame |
| Clutter | chimney / fence | vent / tank / transformer | tank / parapet corner |

If two packs map the **same** module_id for **wall + roof + door**, district swap fails player read — designer must remap slots before PG-2 sign-off.

---

## PG-2 witness sign-off rubric

Use after `@coder` writes `debug_runs/procedural_assembly_live.json` (MCP-PG-2-WIT).

**Template path:** `debug_runs/art_pipeline/procedural_assembly_pg2_signoff.yaml`

```yaml
# procedural_assembly_pg2_signoff.yaml — MCP-DUX-PG2-001 / post MCP-PG-2-WIT
program_id: MCP-FLEET-WAVE3-ENGINE-001
task_id: MCP-DUX-PG2-002
witness: debug_runs/procedural_assembly_live.json
design_charter: src/dev/design_procedural_assembly_read_v1.md
reviewer: designer
proceed_player_visible: yes  # yes | no — blocks PG-3 if no

witness_keys:
  pg2_wired: pass          # must be true
  smoke_fallback_used: pass  # must be false
  footprint_cells: pass      # must be > 0
  green: pass                # rollup true

style_pack_swap:
  footprint: "4x3_reference"
  pack_a: style_victorian
  pack_b: style_industrial_west
  module_ids_must_differ:
    - wall slot (wall_1u or wall_2u)
    - roof slot (roof_default)
    - prop_clutter
  same_footprint_token_count: true
  result: pass  # pass | fail

player_read_lod0:
  wall_family_distinguishable: pass    # brick vs steel silhouette at tactical zoom
  roof_profile_distinguishable: pass   # gable vs sawtooth
  door_width_distinguishable: pass     # residential vs shop/warehouse
  district_one_liner_a: "Brick house with chimney"
  district_one_liner_b: "Steel factory with sawtooth roof"
  notes: ""

failure_modes:
  hide_slot_when_missing: pass         # no smoke cube fallback
  gap_preferred_over_wrong_mesh: pass
  f3_copy_present: optional

quality_debt_acknowledged:
  lod0_pbr_mostly_deferred: true       # expected — not a PG-2 fail
  production_tier_scheduled: true      # kit_production_* lane open
  smoke_never_player_visible: pass

modules_used:
  all_lod0_tier: pass
  no_kit_greybox_job_ids: pass
  canonical_module_ids_only: pass

blocked_by: []
next: "@coder MCP-PG-3-001 parametric commit bridge"
```

### Rubric gate summary

| Gate | Pass when |
|:---|:---|
| **W1** | Witness `green: true`, `smoke_fallback_used: false` |
| **W2** | Victorian vs industrial west → different wall + roof + clutter module_ids, same footprint |
| **W3** | Tactical screenshot: roof profile + wall family readable without labels |
| **W4** | Missing slot → gap, not cube |
| **W5** | All `module_ids_used` are lod0 tier, canonical 50-set |
| **W6** | Quality debt logged — production PBR tracked separately (not blocking PG-2) |

---

## Integration with CON-P2 (construction read)

| Construction phase | Procedural assembly visible? | Player copy |
|:---|:---:|:---|
| `Planned` … `Foundation` | Optional preview ghost only | No final district read |
| `UnderConstruction` | **Yes** — lod0 modules assemble | `{site_name} — Building ({progress_pct}%)` |
| `Operational` | **Yes** — same assembly until production swap | Full district read applies |

Do not promise *"finished architecture"* during Clearing/Surveying — only scaffold + phase labels.

---

## Acceptance (this doc)

| # | Criterion | Status |
|:---:|:---|:---:|
| R1 | lod0 vs production player read defined | ☑ |
| R2 | Victorian vs industrial swap table on same footprint | ☑ |
| R3 | W/D/C → slot → player cue mapped | ☑ |
| R4 | `hide_slot` failure modes aligned with pack RON | ☑ |
| R5 | PG-2 witness YAML rubric template | ☑ |
| R6 | Greybox/smoke excluded from player read | ☑ |
| R7 | Quality/material debt + production path documented | ☑ |

---

## Sign-off

| Role | Verdict | Date |
|:---|:---|:---|
| `@designer` | **PASS** | 2026-06-02 |

---

## Changelog

| Version | Date | Notes |
|:---|:---|:---|
| v1.0.0 | 2026-06-02 | MCP-DUX-PG2-001 — player read charter + PG-2 sign-off rubric |
