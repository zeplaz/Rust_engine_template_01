# MCP fleet Wave 3 — ENGINE + UX orders `v1`

| Field | Value |
|:---|:---|
| **Program ID** | **MCP-FLEET-WAVE3-ENGINE-001** |
| **Parent** | Wave 2 **CLOSED** — [`mcp_fleet_wave2_gate_checklist.json`](../../debug_runs/art_pipeline/mcp_fleet_wave2_gate_checklist.json) |
| **Owner** | `@orchestrator-mcp` |
| **Date** | 2026-06-02 |
| **Status** | **ACTIVE** |
| **Snap** | [`mcp_orchestrator_snap_CURRENT.md`](../../tools/orchestrator/queues/mcp_orchestrator_snap_CURRENT.md) |
| **Queue** | [`mcp_active_queue.json`](../../tools/orchestrator/queues/mcp_active_queue.json) |

**Prerequisite met:** 50/50 lod0 GLBs + 7 style pack RONs + registry tier filter + PG exec v1.1 signed.

**Wave 2 lanes drained:** `@planner-mcp`, `@designer-mcp`, `@coder-mcp` — on-call only.

---

## What Wave 3 delivers

PG-2 procedural assembly in Bevy: footprint W/D/C grid → StylePack slot → lod0 GLB via existing registry — **no smoke**, no new MCP art batches.

---

## Task packets

### MCP-PG-1-001 — `@coder` · **P0 · DONE**

**Goal:** StylePack + archetype types and RON loaders.

**Status:** Shipped — ran parallel with MCP-DUX-PG2-001; no further PG-1 order changes.

**Read first:** [`plan_procedural_build_gen_exec_001_v1.md`](plan_procedural_build_gen_exec_001_v1.md) § PG-1 · [`plan_style_pack_ron_v1.md`](plan_style_pack_ron_v1.md)

**Acceptance:** [x] 7 packs load · [x] slots resolve lod0 · [x] `cargo test -p proc_A_dine01 --lib procedural` green

---

### MCP-PG-2-001 — `@coder` · **P1 · DONE**

**Goal:** Footprint grid + build extract instances.

**Read:** exec plan § PG-2, [`design_procedural_module_kit_v1.md`](design_procedural_module_kit_v1.md) § W/D/C grammar

**Implement (≤3 files):**

| File | Content |
|:---|:---|
| `src/construction/procedural/footprint_grid.rs` | W/D/C tokens from width×depth, door on floor 0 |
| `src/render/extraction/procedural_build_extract.rs` | Grid cell → StylePack slot → `resolve_module_id` → scene handle |
| `src/construction/procedural/mod.rs` | export + register systems |

**Tests:**

- `footprint_grid_door_on_floor_zero`
- `footprint_grid_corner_token_consumes_c`
- `procedural_build_extract_resolves_lod0_glb`
- `procedural_build_extract_skips_smoke_row`
- `procedural_build_extract_hide_slot_when_module_missing`
- `style_pack_victorian_vs_industrial_different_wall_ids`

**Rules:**

- Missing module → hide slot (`fallback_policy: hide_slot`) — **never smoke GLB**
- Single extract path via `RepresentationResult` — Stage 5 convergence

---

### MCP-PG-2-002 — `@coder` · **P1 · DONE**

**Goal:** Wire existing procedural module scene catalog to assembly extract.

**Files:** `src/render/extraction/procedural_module_extract.rs`, extract graph registration, `RepresentationResult.procedural_module_meshes`

**Acceptance:** Scene handles load only for `modules_for_stylepack()` entries used by active StylePack.

---

### MCP-PG-2-WIT — `@coder` · **P1 · DONE**

**Goal:** Closure witness.

**File:** `debug_runs/procedural_assembly_live.json`

| Key | Pass when |
|:---|:---|
| `pg2_wired` | true |
| `style_pack_id` | e.g. `style_victorian` |
| `module_ids_used` | all lod0+, no `kit_greybox` job ids |
| `smoke_fallback_used` | **false** |
| `footprint_cells` | > 0 |
| `green` | true |

**Witness landed:** `green: true` — UX sign-off gate **open** (YAML pending).

---

### MCP-DUX-PG2-001 — `@designer` · **P0 · DONE**

**Goal:** Player read charter for lod0 assembly — unblocks PG-2 UX sign-off.

**Output:** [`design_procedural_assembly_read_v1.md`](design_procedural_assembly_read_v1.md)

**Must cover:**

- What players distinguish at lod0 (wall material family, roof profile, door width) vs production tier later
- StylePack swap read (Victorian brick vs industrial steel on same footprint)
- Failure modes: hidden slot vs primitive fallback — align with `hide_slot` policy
- Sign-off rubric YAML template for post-PG-2-WIT review

**Inputs:** 7 style packs, 50 lod0 modules, construction stage read docs (CON-P2)

**Does not:** author AssetSpecs, run MCP tools

---

### MCP-DUX-PG2-002 — `@designer` · **P0 · READY**

**Goal:** Fill PG-2 witness sign-off YAML after tactical review.

**Witness:** [`debug_runs/procedural_assembly_live.json`](../../debug_runs/procedural_assembly_live.json) (`green: true`)

**Template:** [`debug_runs/art_pipeline/procedural_assembly_pg2_signoff.yaml`](../../debug_runs/art_pipeline/procedural_assembly_pg2_signoff.yaml) · charter § PG-2 witness sign-off rubric

**Acceptance:**

- Copy witness keys (`pg2_wired`, `smoke_fallback_used`, `footprint_cells`, `module_ids_used`, …) → rubric `pass`
- W2 style-pack swap check (Victorian vs `style_industrial_west` on same footprint)
- W3 tactical review → set `proceed_player_visible: yes` \| `no`
- **Does not** replace or satisfy **ART-APS-USE** / **TILE-REAL-001** — separate gates

**After pass:** `@designer-mcp` may pivot to **MCP-APS-PILOT-001** (variant_set + tile batch JSON) if on art lane.

---

### MCP-T0-003 — `@designer-mcp` · **P3 · OPTIONAL**

**Goal:** Validate-only on tile batch draft — no bake.

```powershell
python -m rust_engine_mcp.cli validate-report tile_batch tools/mcp/schemas/examples/tile_batch_factory_floor_v1.json
```

**Blocked by:** nothing — optional polish after ENGINE starts.

---

## Drained lanes — do not re-dispatch

| Agent | Wave 2 completed |
|:---|:---|
| @planner-mcp | PLN-SP-001, T0-001 |
| @planner | PLN-PG2-001 |
| @designer-mcp | D0-003…010, D0-SP-001/002, all G5 |
| @coder-mcp | C0-004…011, C0-012, T0-002 |

---

## Verification

```powershell
cargo test -p proc_A_dine01 --lib procedural construction
# After PG-2-WIT:
# debug_runs/procedural_assembly_live.json green: true
```

---

## Changelog

| Version | Date | Notes |
|:---|:---|:---|
| v1.0.0 | 2026-06-02 | Wave 3 ENGINE only — post Wave 2 closure |
| v1.1.0 | 2026-06-02 | PG-1/PG-2/PG-2-WIT done; MCP-DUX-PG2-002 READY (YAML sign-off) |
