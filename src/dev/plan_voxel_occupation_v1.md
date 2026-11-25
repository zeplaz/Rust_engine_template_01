# PLAN-VOXEL-OCCUPATION-v1 — sim authority charter (X1)

| Field | Value |
|:---|:---|
| **⟨ID⟩** | **PLAN-VOXEL-OCCUPATION-v1** |
| **Slice** | **PLAN-REF-BOOKS-UPGRADE-v3** wave V3 **X1** (charter only — **no X2 impl**) |
| **Owner** | `@planner` (charter) → **X2** `@coder` + `@sim-steward` (EV/Cx gate) |
| **Status** | **OPEN 2026-11-24** — review fixes done (save/load wired, deterministic site pick, witness code deleted: CQ-07/20/30, `efe818b8`); flags default **off** until OCC-P4/P5 re-score for default-on. Was: **REOPENED 2026-11-16** — commit authority defaults ON but voxel occupation is not saved (overlay is an unwired stub); non-deterministic site pick; OCC-P6 "live witness" is banned test-shaped code; P2 test fails. **DR-VOXEL-OCCUPATION reopened** · cleanup **CQ-07, CQ-20, CQ-30** |
| **Parents** | [`plan_ref_books_upgrade_v3.md`](plan_ref_books_upgrade_v3.md) · [`plan_voxel_massing_presentation_v1.md`](plan_voxel_massing_presentation_v1.md) (**VOXEL-MASSING-PRES-001**) · [`plan_artist_studio_expansion_v1.md`](plan_artist_studio_expansion_v1.md) · [`construction_invariants.md`](construction_invariants.md) |
| **Book rows (dual-cite)** | **NEW:** **PCG-06** (aig3:8.5–8.5.2) · **ECS-15** (gea4v2:16.2.2) · **LEGACY:** `shape_grammars` (aig_e3 p748) · `asset_pipeline` (gea_v1_e4 p68) |
| **Deferral** | **DR-VOXEL-OCCUPATION** — **CLOSED** 2026-11-15 (`voxel_occupation_authority_live.json`) |
| **Steward EV/Cx** | 2026-11-15 review said PASS; its JSON record was a gitignored debug file and is not evidence (CQ-84). Evidence now: tests in `src/strategic/site/voxel_occupation.rs` (`cq07_default_config_is_off_until_save_proven`, `voxel_occupation_snapshot_ron_round_trip`, `apply_site_commit_appends_on_recommit_and_picks_lowest_site_id`, `voxel_occupation_commit_authority_writes_and_blocks_reserve`). |

**EV/Cx (steward):** Q↑ Ct▮▮▮ Cx▮▮▮▮▮▮ EV/Cx=**MED-HIGH** — bounded authority migration inside existing construction execute funnel; risk = dual occupation during phased cutover, save slice ordering, and `RepresentationResult` accidentally becoming a writer. Complexity budget: **one** new sim resource family + **one** RON hydrate slice; no parallel nav/PHYS/voxel gameplay in X2 scope without separate charter. **Recorded:** PASS unlocks X2-P0 only (flag default off); P1+ retains phased dual-book guards.

---

## Review 2026-11-16 — reopened

> Operator code review of the Cursor wave ([`plan_cleanup_cursor_wave_v1.md`](plan_cleanup_cursor_wave_v1.md)). Until the save path is wired, the safe default is commit authority **OFF** (CQ-07).
>
> **CQ-07/CQ-30 progress 2026-11-16:** save overlay write/hydrate wired; flags default off; `by_site` BTreeMap + append; witness JSON API removed. Close again only after OCC-P4/P5 re-score for product default-on.

| Finding | Evidence | Fix |
|:---|:---|:---|
| Not persisted | `io/save/voxel_occupation_overlay.rs` is a "P0 stub — not wired"; `VoxelOccupationHydrateState` never `init_resource` | Wire into manifest save/load + round-trip test; until then default flags OFF (CQ-07) |
| Non-deterministic | `primary_site_at` uses `find_map` over `HashMap by_site` (`voxel_occupation.rs:246`); `apply_site_commit` overwrites on re-commit (`:202`) | `BTreeMap`, lowest-id tie-break, append (CQ-07) |
| Witness code in production | `VoxelOccupationAuthorityWitness`, `authority_witness_stub`, `write_authority_witness_json`, hardcoded `issued`, always-true `dual_writer_guard_ok` (`:423-470, 541-552, 607-665`) | Delete; counts only in `sim_run_dump` (CQ-30) |
| Failing test | `voxel_occupation_p2_sole_writer_tile_derived_cache` asserts `"X2-P2"`, code returns `"OCC-P5"` | Fix with CQ-30 (CQ-20) |
| Docs contradict code | module docs say flags default off; `Default` sets both on (`:1-15` vs `:30-44`) | Align |
| Evidence untracked | cited `debug_runs/construction/*.json` are gitignored | **Done (CQ-84, 2026-11-24):** plan cites tests instead |

**Close again only when:** voxel occupation survives save → load in a test, the site pick is deterministic, no witness code remains, and lib tests are green. Every ADOPTED claim cites a production caller `file:line` (a test alone does not count).

## Summary

Voxels are **in scope** for the ref-books v3 program. **DR-VOXEL-OCCUPATION** gates **only** the moment sim truth stops being “catalog `FootprintMatrix` + weighted `TileOccupationBook` XY” and becomes a **committed voxel column occupancy model** that construction, logistics overlap checks, and save/load treat as authoritative. Presentation (`massing_columns`, Studio ParamFrame, PixelWorld silhouette) stays on the **ECS-15** tool/runtime split: tools condition assets; sim commits through the construction execute funnel only.

This charter names the **sole writer**, a **conflict matrix** vs roads, zones, and `RepresentationResult`, and **save/hydrate** rules so **X2** can migrate without a second hidden occupation graph.

---

## Current problems (honest baseline)

| Issue | Why it blocks naïve voxel promotion |
|:---|:---|
| **Split occupation surfaces** | Gameplay validity uses catalog `FootprintMatrix` booleans + commit-time `TileOccupationBook` Σw per tile (`src/strategic/site/tile_occupation.rs`). Massing stacks are **presentation-only** (`src/construction/voxel_massing.rs`). DR wording “replace FootprintMatrix XY” is correct direction but **today** weighted book is the commit aggregate. |
| **Roads are graph-first, tiles-second** | Committed road/rail paths rasterize to world cells and hydrate `TransportTopology` (`construction_pipeline` + R8 snapshot). Voxel columns must not silently punch through reserved corridor cells without the demolish / plan queue funnel. |
| **Zones ≠ buildings** | `Zone` entities and `ActiveZonePaint` are district metadata (`construction_invariants` §12). Voxel occupation must not collapse zoning into footprint occupation or bypass `PendingEntryKind::ZonePaint`. |
| **`RepresentationResult` is band policy, not sim** | Sole frame writer: `build_representation_result` (`WorldRepresentationSystemSet::ComputeFrame`). Tactical occupation **tokens** are drawn from construction visuals + strategic icons — not from PG-2 policy fields. A voxel migration must not make repr bake a shadow occupation writer. |
| **Studio / APS lane** | ParamFrame and `massing_preview.json` carry `sim_occupation: false` (S0/S2 ADOPTED). Tool-side grammars (**PCG-06**) inform height intent only until X2 defines an explicit conditioned → commit bridge. |

---

## Target architecture (X2 — chartered, not implemented here)

### Authority graph

```text
⊚ConstructionExecuteFunnel ═▶ ⊨committed world mutation
  execute_construction_plans_system
  commit_construction_site_system (CommitConstructionSiteEvent)
  zone confirm · demolish/blast after approval
  (unchanged invariant: preview never mutates gameplay)

⊚VoxelOccupationAuthority ═▶ ⊨sim XY(+Z) occupancy truth   [X2 introduces]
  ⊰ ⊚ConstructionExecuteFunnel          — only execute paths write
  ⊰ ⊚FootprintMatrix (catalog)            — shape template RO at preview; not world truth post-cutover
  ⊰ ⊚TileOccupationBook                 — either retired as sole aggregate OR demoted to derived cache

⊚TileOccupationBook (interim) ═▶ ⊨weighted Σw per tile   [today]
  writer: commit/demolish paths only

⊚ConstructionVisualRequests.massing_columns ═▶ ⊨presentation stacks
  writer: sync_voxel_massing_visual_requests (X0 ADOPTED)
  ⊰ ⊚VoxelOccupationAuthority (future) OR ghost/footprint derive until cutover flag

⊚RepresentationResult ═▶ ⊨per-frame LOD / extract / overlay policy
  writer: build_representation_result
  ◂⊳[RepresentationInputs + WorldRepresentationFrame] ⊚Sim + view snaps
  ⛔ must not write VoxelOccupationAuthority or TileOccupationBook

⊚TransportTopology + CorridorConstructionBook ═▶ ⊨network + corridor phase
  roads/rail: ConstructionPlanQueue → execute → tile markers → R8 hydrate
```

### Proposed sim resource (X2 name frozen here)

```text
VoxelOccupationBook {
  // Deterministic sparse map: world cell → occupancy record
  // Minimum viable record:
  //   xy: IVec2
  //   layers: OccupancyLayerStack   // capped Z columns per product rule (e.g. 1..=K)
  //   site_id: Option<SiteId>
  //   corridor_reserve: bool       // true when edge-owned strip locks cell
}
```

**Sole writer (production):** the same set as `TileOccupationBook` today — **only** construction execute + approved demolish/blast + **one** save-game hydrate path. Overlap validation reads RO snapshots during preview (`build_interaction`, staged panel).

**Consumers (RO after commit phase):** logistics overlap, fire exposure (`TileOccupationBook` call sites today), `RepresentationInputs` builders, massing derive, world_map / pixel_world paint, sim_review dump counters (counts only).

---

## Conflict matrix (roads · zones · RepresentationResult)

| Peer | Authority | Relation to voxel occupation | Rule |
|:---|:---|:---|:---|
| **Road / rail tile strips** | `ConstructionPlanQueue` → execute → `SimRoadSegmentMarker` / markers → `bake_snapshot_from_ordered_tile_markers` → `TransportTopology` | **COORDINATE — SERIAL on conflict** | Committed corridor cells export a **reserve mask** into occupation authority. Voxel place preview may not propose commit on reserved cells. Changing a road strip goes through plan execute + transport dirty mask — not massing derive. |
| **`CorridorConstructionBook`** | Phase overlay per `TransportEdgeId` | **PARALLEL** | Phase polylines are **visual** (`sync_corridor_phase_visual_requests`). Occupation records carry `corridor_reserve` when tile is part of an active edge footprint; book rows do not dual-write occupation. |
| **`Zone` / zone paint** | `Zone` components on confirm; `ActiveZonePaint` preview | **COORDINATE — ORTHOGONAL** | Zones classify districts; they do **not** allocate Σw. Voxel commit still requires `allows_commit` / plan `Validated`. Zone paint never creates `CivilHousing` site rows (invariant §5). Post-X2: optional **read** of zone kind for AI/economy — not a writer. |
| **`FootprintMatrix` (catalog)** | Building definitions / ghost template | **DERIVES-FOR-PREVIEW → REPLACED-FOR-TRUTH** | Until cutover flag: ghost still uses catalog matrix for pick UX. After cutover: matrix seeds **candidate voxels** only; commit writes `VoxelOccupationBook`. |
| **`TileOccupationBook`** | Weighted Σw (today) | **SERIAL / MERGE** | X2 phase A: occupation book becomes **derived mirror** refreshed from voxel authority each commit (compat). Phase B: remove book writes; single voxel writer. No frame where both are independently mutable. |
| **`RepresentationResult`** | LOD band, extract plan, overlay matrix, `building_visual_simplified`, procedural mesh gate | **CONSUMES SIM — NEVER WRITES** | Tactical band may show occupation **tokens** from construction visuals; PG-2 bake may simplify **meshes** for committed sites. Voxel stacks for play are **not** authored by repr policy. Static audit: `world_presentation_mode` must keep zero `ResMut<RepresentationResult>`. |
| **`massing_columns` / Studio override** | `ConstructionVisualRequests` | **DERIVES-FROM sim or ghost** | **ECS-15:** APS/Studio never commits occupation. Engine derive may read voxel authority RO after X2; until then X0 path stands. |
| **Blast / demolish** | `execute_demolish_at_tile`, `BlastCraterBook` | **SERIAL** | Demolish removes site occupancy through same writer as commit; craters may clear voxel layers per charter rule in X2 tests. |
| **Nav / transport motion** | `TransportTopology`, solvers | **DERIVES-FROM topology** | No “nav-on-stacks” in X2 — path legality stays graph-first (**REJECT** per presentation plans). |

---

## Save / hydrate (X2 contract — specified here)

| Artifact | Format | Owner slice | Hydrate order |
|:---|:---|:---|:---|
| **Transport R8** | Existing `TransportNetworkSnapshot` + optional `construction[]` phase records | transport | Unchanged — voxel slice is **sibling**, not embedded in edge geometry. |
| **`VoxelOccupationSnapshot` v1** (new) | **RON** preferred (engine serde policy); JSON only for external editor interchange | construction / strategic save bundle | Load **before** overlap-sensitive sim tick **after** transport topology hydrate **or** in same loader pass with explicit dependency: `topology → occupation → logistics GraphSync`. |
| **Studio massing override** | `massing_preview.json` / env `RUST_ENGINE_STUDIO_MASSING` | presentation | **Never** hydrates sim occupation — optional one-shot visual override only (X0). |

**Determinism:** snapshot schema includes `schema_version`, stable sort of sparse cells, seed-independent cell keys (world `IVec2` + layer index). Replay must not depend on frame order of visual sync.

**Hydrate writers:** exactly **one** loader function (named in X2 impl plan) — no secondary “fixup” systems in Update.

**Dump / review:** extend `sim_run_dump` with `occupation.voxel_*` **counts only** (D0 pattern) — no `*_green` predicates in production.

---

## ECS schedule placement (X2)

```text
PreUpdate / Update (sim)
  SimControlSystemSet::AdvanceSimTick
    … transport / strategic (unchanged)

PostUpdate (construction — after ApplyCameraScissor per invariants)
  ConstructionExecuteSet::DrainPlans          // existing funnel
    → sole mutate VoxelOccupationAuthority
    → refresh derived TileOccupationBook (phase A only)
  ConstructionVisualSet::SyncRequests
    → sync_voxel_massing_visual_requests      // RO authority

Update (view — unchanged)
  WorldRepresentationSystemSet::ComputeFrame
    → build_representation_result             // RO occupation
```

**Extraction boundary:** massing sync stays **after** occupation commit for the same sim step when cutover enabled; never write occupation from render extract or PG-2 bake.

---

## DR-VOXEL-OCCUPATION — unblock predicates

| Predicate | X1 (this doc) | Remaining for DR **close** |
|:---|:---|:---|
| Signed `plan_voxel_occupation_v1.md` | **PASS** (SIGNED) | — |
| Conflict matrix vs roads / zones / `RepresentationResult` | **PASS** (§ Conflict matrix) | — |
| Steward **EV/Cx PASS** | **PASS** 2026-11-15 (`@sim-steward` · X2-P0 scope) | — |
| User world sign-off | N/A at charter | Required only if X2 changes default save semantics or visible gameplay rules (`plan_ref_books_upgrade_v3` §0.1 hard gate) |

**DR status:** **X2 implemented** — **DR-VOXEL-OCCUPATION** stays `product_blocked` until default-on is decided (OCC-P4/P5). Evidence: tests in `src/strategic/site/voxel_occupation.rs` (`cq07_default_config_is_off_until_save_proven`, `voxel_occupation_snapshot_ron_round_trip`, `apply_site_commit_appends_on_recommit_and_picks_lowest_site_id`, `voxel_occupation_commit_authority_writes_and_blocks_reserve`); counts in the `sim_run_dump` occupation section.

---

## Implementation phases (X2 DAG — reference for @coder; do not execute in X1)

| Phase | Goal | Gate |
|:---|:---|:---|
| **X2-P0** | `VoxelOccupationBook` resource + schema + loader stub; derived mirror from `TileOccupationBook`; feature flag default off | charter + EV/Cx · **ADOPTED 2026-11-15** — Accept `voxel_occupation` → 6 passed |
| **X2-P1** | Commit path writes voxel authority; overlap tests; road reserve mask | P0 green · **ADOPTED 2026-11-15** — `commit_authority_enabled` default off; commit dual-write; road/rail reserve in execute funnel; Accept `voxel_occupation` 7 passed |
| **X2-P2** | Remove dual writer on `TileOccupationBook`; migrate call sites | P1 witness diff · **ADOPTED 2026-11-15** — under `commit_authority_enabled`, voxel sole writer + tile `replace_from_voxel`; flag default off; Accept `voxel_occupation` + `tile_occupation` |
| **X2-P3** | Massing derive reads voxel RO; dump counters; dual-cite in commit | **ADOPTED 2026-11-15** — optional `Res<VoxelOccupationBook>` RO in `sync_voxel_massing_visual_requests` when commit/mirror on; `occupation.*` dump counts; dual-cite **PCG-06** · **ECS-15**; Accept `voxel_massing` + `voxel_occupation` |

### X3 — productize (flag-on / EV-Cx / DR close)

Cross-link: [`plan_ref_books_defer_unlock_v1.md`](plan_ref_books_defer_unlock_v1.md) §8. Flags stay **default off** until **OCC-P4** PASS.

#### OCC-P4 — steward EV/Cx for product default-on
| Field | Value |
|:---|:---|
| **Needs** | X2-P3 ADOPTED · product wants default-on |
| **Files** | this plan amend (decision recorded here, not in a debug file) |
| **Authority** | `@sim-steward` EV/Cx; **no** code flag flip yet |
| **Do** | Re-score EV/Cx for **default-on** (save semantics, dual-book absence, overlap call sites, demolish/blast). Complexity: no new resource family. |
| **Don't** | Flip default in same slice as first EV/Cx · Studio write path |
| **Accept** | Steward JSON `verdict: PASS` · dual-writer guard tests still green under flag-on in harness |
| **Unlocks** | Clears OCC-P5 |

#### OCC-P5 — product default-on + save hydrate
| Field | Value |
|:---|:---|
| **Needs** | OCC-P4 PASS |
| **Files** | `src/strategic/site/voxel_occupation.rs` (Default/env) · `src/io/save/voxel_occupation_overlay.rs` · construction commit/demolish (verify) · city/save fixtures |
| **Authority** | Same execute funnel; Default `commit_authority_enabled=true` (env still overrides off) |
| **Do** | Default-on; RON hydrate sole loader; round-trip save; massing RO unchanged |
| **Don't** | Dual mutable `TileOccupationBook` · `RepresentationResult` write · test `*_green` resources |
| **Accept** | `cargo test -p proc_A_dine01 --lib voxel_occupation` · `tile_occupation` · `voxel_massing` · hydrate round-trip · `validate_cargo_report` error_count 0 |
| **Unlocks** | Play product path |

#### OCC-P6 — DR close witness
| Field | Value |
|:---|:---|
| **Needs** | OCC-P5 green |
| **Files** | tests in `src/strategic/site/voxel_occupation.rs` (`cq07_default_config_is_off_until_save_proven`, `voxel_occupation_snapshot_ron_round_trip`, `apply_site_commit_appends_on_recommit_and_picks_lowest_site_id`, `voxel_occupation_commit_authority_writes_and_blocks_reserve`) · `plan_deferral_registry_v1.md` **DR-VOXEL-OCCUPATION** |
| **Do** | Emit live witness (revision, cell count, `dual_writer_guard_ok`, flags); close DR row |
| **Don't** | Verdict predicates in production code |
| **Accept** | Witness present · DR closed/product_shipped · dump keys non-zero under playcheck+dump env |
| **Unlocks** | **DR-VOXEL-OCCUPATION** close |

⛔ **Out of scope (unless new DR):** freeform dig/Minecraft placement, PHYS stacks, AI-on-columns, nav-on-stacks, Studio sim writes, second GPU occupation extractor.

---

## Diagnostics / witnesses (X2 targets)

| Witness | Purpose |
|:---|:---|
| `sim_run_dump.json` occupation section (debug run, local) + `voxel_occupation_dual_writer_guard_tile_mutation_without_mirror` test | revision, cell count, dual-writer guard |
| Lib tests `voxel_occupation::` + migrated `tile_occupation::` | overlap, road reserve, hydrate round-trip |
| sim_review dump keys `occupation.*` | regression counts |

---

## Edge cases

- **Partial-weight tiles** today (parametric Σw) — X2 must define voxel equivalent or staged deprecation of fractional overlap.
- **Chunk stream / off-map** — sparse book only; no global dense volume allocation at strategic zoom.
- **Undo/redo** — `ConstructionHistory` must capture voxel delta alongside site/road actions.
- **Transport reload** — re-applying R8 must not clear occupation unless explicit demolish; reserve mask recomputed from markers.
- **Minimap / macro band** — `RepresentationResult` may collapse visuals; occupation truth remains full-fidelity sim resource.

---

## Global rejects (reinforce v3 §9)

- Second occupation authority writable from UI, Studio, repr bake, or pixel_world weather layers.
- Inferring gameplay validity from `massing_columns` stack height.
- Skipping construction execute funnel for “quick voxel place”.
- Single-matrix book orient on X2 commits (dual-cite **PCG-06** + **ECS-15** + legacy keywords).

---

## Acceptance (X1)

- **Charter path:** `src/dev/plan_voxel_occupation_v1.md` (**SIGNED**)
- **EV/Cx note:** MED-HIGH — steward review before X2-P0; authority bounded to construction funnel + one RON slice
- **Dual-cite evidence:** **PCG-06** + **ECS-15** + LEGACY `shape_grammars` · `asset_pipeline`
- **Do not implement:** X2 code in this slice

**Deferrals:** [`plan_deferral_registry_v1.md`](plan_deferral_registry_v1.md) — **DR-VOXEL-OCCUPATION** (partial predicates satisfied).  
**Migration truth:** CI green on `master` (`cargo test --lib`); [`plan_bevy_020_migration_v1.md`](plan_bevy_020_migration_v1.md)

---

## Changelog

| Version | Date | Notes |
|:---|:---|:---|
| 1.0.0 | 2026-11-15 | X1 SIGNED — conflict matrix, sole writer, save/hydrate, DR predicates, EV/Cx. |
| 1.0.1 | 2026-11-15 | `@sim-steward` EV/Cx **PASS** — X2-P0 clear; witness `voxel_occupation_steward_evcx_2026-11-15.json`. |
| 1.0.2 | 2026-11-15 | X2-P0 **ADOPTED** — Accept `voxel_occupation` 6 passed; `strategic::` re-exports + `VoxelOccupationProvenance: Default`. |
| 1.0.3 | 2026-11-15 | X2-P1 **ADOPTED** — commit dual-write + corridor reserve (flags default off); Accept 7 passed. |
| 1.0.4 | 2026-11-15 | X2-P2 **ADOPTED** — flag-gated sole voxel writer on commit/demolish; tile derived via `replace_from_voxel`; default off; P3+ massing derive still OPEN. |
| 1.0.5 | 2026-11-15 | X2-P3 **ADOPTED** — massing RO-reads voxel `z_layers` when commit/mirror on; `sim_run_dump` `occupation.*` counts; dual-cite **PCG-06** · **ECS-15**; flag default off. |
| 1.1.0 | 2026-11-15 | **OCC-P4…P6** productize (flag-on / EV-Cx / DR close) added. |
| 1.1.1 | 2026-11-15 | **OCC-P4 PASS** — `voxel_occupation_productize_evcx_2026-11-15.json`; OCC-P5 clear. |
| 1.2.0 | 2026-11-15 | **OCC-P5** — `VoxelOccupationConfig` default on; env `0/false/off` forces debug off; Accept `voxel_occupation` 11 passed. |
| 1.3.0 | 2026-11-15 | **OCC-P6** — `write_authority_witness_json` → `voxel_occupation_authority_live.json`; Accept `--lib occ_p6`; **DR-VOXEL-OCCUPATION** closed. **PROGRAM CLOSE**. |
| 1.4.0 | 2026-11-16 | **REOPENED** — review: not persisted while default-on, HashMap nondeterminism, witness code, failing P2 test; DR-VOXEL-OCCUPATION reopened; cleanup CQ-07/20/30. |