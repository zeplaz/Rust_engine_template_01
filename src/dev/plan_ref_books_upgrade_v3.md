# PLAN-REF-BOOKS-UPGRADE-v3 — dual matrices · debug · tools suite · voxels in scope

| Field | Value |
|:---|:---|
| **Id** | **PLAN-REF-BOOKS-UPGRADE-v3** |
| **Status** | **REVIEWED 2026-11-16** — several ★ ADOPTED rows re-graded to PARTIAL/UNWIRED in `adoption.tsv` (cleanup plan §5); §2.1 counts are stale until regenerated (CQ-80). Was: PROGRAM CLOSE — V0–V4 cuttable OPEN ★ ADOPTED; residual DEFER/PARTIAL unlock roadmap: [`plan_ref_books_defer_unlock_v1.md`](plan_ref_books_defer_unlock_v1.md) |
| **Date** | 2026-11-15 · closed 2026-11-15 |
| **Parents** | [`plan_ref_books_upgrade_v2.md`](plan_ref_books_upgrade_v2.md) (**WAVES 0–S CLOSED**) · [`plan_ref_books_upgrade_use_v1.md`](plan_ref_books_upgrade_use_v1.md) (**SHIPPED**) · [`plan_artist_studio_expansion_v1.md`](plan_artist_studio_expansion_v1.md) · [`plan_voxel_massing_presentation_v1.md`](plan_voxel_massing_presentation_v1.md) · [`plan_shared_atmosphere_weather_fire_v1.md`](plan_shared_atmosphere_weather_fire_v1.md) |
| **Obeys** | `AGENTS.md` · one writer · Sim→View→Render→UI · Studio = DCC (parametric frames) · play UI only · no test-shaped production |
| **Written for** | Any agent. Dual-matrix orient is mandatory; voxels are **in scope**. |
| **Continuity** | §0.1 honest exits — `BLOCKED` / `UNWIRED` / asking are valid exits; marking unwired work ADOPTED is the failure. |

## 0. Stance (locked)

| Lock | Meaning |
|:---|:---|
| **Voxels = full scope** | Voxel / massing work is **in** this program — presentation, studio param frames, engine consumers, and (when chartered) sim occupation. Do **not** treat “presentation-only forever” as a freeze; push the upgrade across tools **and** voxel surfaces. Prior `DR-VOXEL-OCCUPATION` is a **sequencing gate for sim-authority replacement**, not a ban on voxel work. |
| **Dual matrices = parallel combined** | Legacy keyword matrix **and** new concept matrix are **one orient**. Both run every slice. Not alternatives, not “risky dual source” — complementary coverage (keywords ↔ rows ↔ pages ↔ code). |
| **Tools suite first multiplier** | Artist Studio, MCP, `studio_dash`, validators, `sim_review`, `ref_books_index` / legacy lookup — upgrade these so voxel + grammar + debug loops stay parametric and measurable. |
| **Studio = DCC** | Parametric frames / grammars / modifiers / connection logic in Studio; Blender/audio escalate only when the frame cannot express intent ([`plan_artist_studio_expansion_v1.md`](plan_artist_studio_expansion_v1.md) § Creative DCC). |

### 0.1 Honest exits (replaces "Anti-stall", removed 2026-11-16)

The old anti-stall rule treated stopping, asking, or reporting a gap as a "protocol failure". The 2026-11-16 review found it pushed agents to mark slices ADOPTED with no production caller, and with tests written to pass (see [`plan_cleanup_cursor_wave_v1.md`](plan_cleanup_cursor_wave_v1.md)).

- **Valid exits:** Accept lines pasted **and** a production caller `file:line` cited; or `BLOCKED: <reason>`; or `UNWIRED: <what is missing>`; or a question to the operator.
- **Invalid:** ADOPTED / DONE / PROGRAM CLOSE without a production caller; "follow-on" notes inside a DONE row; a test that compares a path with a copy of itself; tests that grep source or doc comments as Accept.
- **File ownership:** two agents never edit the same file. On conflict, the second agent picks another OPEN slice whose files are free.

## 1. Dual-matrix protocol (every slice)

```text
ORIENT (both, same session)
  A. NEW   python src/dev/ref_books_index/lookup.py --brief <file>
           python src/dev/ref_books_index/lookup.py --row <ID>   # when known
  B. LEGACY  keyword → C:\dev\ref books\lookup\keyword_book_matrix.csv
             → keyword_lookup.csv (section + pdf_page)
             → catalog.json → PDF page
  C. COMBINE  cite NEW row id + LEGACY keyword(s) in the commit / ledger
              Prefer gea4v* / gea_v*_e4 over gea3 / gea_e3 when both hit.
READ        NEW: lookup.py --read <book:sec>  (≤ ~12 pages)
            LEGACY: open PDF at keyword_lookup pdf_page when NEW TOC misses the niche
DO / ACCEPT / LEDGER  same as v2 §0; ledger records BOTH cites
```

| Index | Path | Role |
|:---|:---|:---|
| **NEW** | `src/dev/ref_books_index/` (`matrix.md` · `systems.md` · `lookup.py` · `adoption.tsv`) | Concept rows ★/●, code paths, adoption ledger, `--open` / `--gaps` |
| **LEGACY** | `C:\dev\ref books\lookup\` (`keyword_book_matrix.csv` · `keyword_lookup.csv` · `INDEX_MATRIX.md`) | Keyword × 4 books → PDF page; Vol II niches; rebuild via `_build_index.py` |

**Book id crosswalk (combined):**

| NEW | LEGACY | Prefer |
|:---|:---|:---|
| `gea4v1` | `gea_v1_e4` | foundations / debug / HID / asset pipeline |
| `gea4v2` | `gea_v2_e4` | graphics / lighting / streaming / gameplay runtime |
| `aig3` | `aig_e3` | PCG / grammars / AI tooling |
| `gea3` | `gea_e3` | fallback only |

**Reject:** picking only one matrix and calling orient done · inventing a third matrix · copying book text into the repo.

## 2. Pass results (2026-11-15) — books × debug × tools × voxels

### 2.1 NEW matrix (`lookup.py --check` → 0 problems · 217 rows)

| Bucket | Signal | Upgrade implication |
|:---|:---|:---|
| Adoption | **117 ADOPTED · 2 PARTIAL (LOCKED) · 14 DEFER · 0 OPEN** (`adoption.tsv` · `--open` = DEFER/PARTIAL only) | Cuttable ★ closed; leave MOVE/TAC/RND/LEARN/PCG DEFERs until triggers; PARTIAL EXEC-05/TAC-03 stay locked |
| DBG | DBG-04 / DBG-06 **ADOPTED** | GPU capture + pause/step shipped |
| Tools / authoring | ECS-15 · IO-09 · DES-04 · PCG-06 **ADOPTED** (V1 S0–S5) | Studio param frames + MCP promote shipped |
| Voxel / PCG | PCG-02/04/05 · RND-09 · LGT-04 **ADOPTED** (V3); **PCG-07** DEFER (town-growth) · **PCG-03** DEFER ⚠ world | Massing + flora + atmosphere presentation shipped; occupation flag default-off |
| Gaps (large files) | residual tooling briefs only | Not blocking PROGRAM CLOSE |

### 2.2 LEGACY matrix (158 keywords)

| Keyword | Primary page | Pairs with NEW |
|:---|:---|:---|
| `debug_tools` | gea_v1_e4 p566 | DBG-01..07 |
| `debug_drawing` | gea_v1_e4 p571 | DBG-03 |
| `debug_cameras_pause` | gea_v1_e4 p582 | DBG-06 |
| `render_debug_tools` | gea_v2_e4 p248 | DBG-04 |
| `asset_pipeline` | gea_v1_e4 p68 | IO-09 |
| `pipeline_app_layer` | gea_v2_e4 p133 | RND app extract |
| `shape_grammars` | aig_e3 p748 | PCG-06 |
| `pcg` | aig_e3 p690 | PCG-* |
| `ai_tools` | aig_e3 p896 | DES-04 |
| `overlays_ui` | gea_v2_e4 p245 | UI read snaps |
| `geometry_visual_effects` | gea_v2_e4 p141 | voxel / VFX layers |
| `world_streaming` | gea_v2_e4 p589 | IO streaming |
| `engine_configuration` | gea_v1_e4 p454 | IO-11 |

### 2.3 Debug + tools suite (in-repo)

| Surface | Path | v3 job |
|:---|:---|:---|
| Sim review | `tools/sim_review/` | Atmosphere + voxel/massing dump counters; PlayQuad seed path; keep T6 multi-seed |
| Validators | `tools/validators/` + MCP `validate_*` | Massing / grammar / cargo reports stay structured |
| Studio dash | `tools/studio_dash/` | Param-frame QC HTML; massing preview; mission adapters |
| APS / Studio | `tools/mcp/art_pipeline_suite/` | DCC loop: frames → grammar/modifier → preview; Blender escalate optional |
| MCP + Blender | `tools/mcp/` · addon | Same StudioTask schemas; voxel/massing export jobs |
| Ref-books NEW | `src/dev/ref_books_index/` | Keep `--open`/`--gaps`; add dual-cite helper (slice T0) |
| Ref-books LEGACY | `C:\dev\ref books\lookup\` | Keep rebuild; document dual orient in README (both sides) |
| Engine dump | `src/dev/sim_run_dump*.rs` | `atmosphere.*` + massing/voxel presentation counts (no `*_green`) |

## 3. Pick order (waves)

| Wave | Focus | Why |
|:---|:---|:---|
| **V0 Dual-matrix tooling** | T0–T2 | Make combined orient cheap for every later slice |
| **V1 Tools suite** | S0–S5 | Studio DCC param frames, dash, MCP parity — multipliers |
| **V2 Debug** | D0–D3 | Capture / pause / dump / playcheck — prove voxel + weather work |
| **V3 Voxels (full scope)** | X0–X5 | Massing ↔ engine ↔ occupation charter path |
| **V4 Book open ★ (engine)** | B0+ | Remaining NEW open ★ that tools/debug/voxel unlocked; still dual-cite |

Waves V1–V3 may run **in parallel** when file ownership does not overlap. V0 before any other wave’s first Accept.

---

## 4. Wave V0 — dual-matrix tooling

### T0 ⚙ Dual-orient CLI — V5 C2 R1
- **Why**: agents stop treating matrices as either/or.
- **Files**: `src/dev/ref_books_index/lookup.py`, `README.md`; note in `C:\dev\ref books\lookup\INDEX_MATRIX.md`
- **Do**: add `--dual <terms| --code PATH>` that prints NEW hits **and** LEGACY keyword rows (read legacy CSVs if present; if missing, print `legacy: unavailable` and still exit 0). JSON mode includes `legacy:[{keyword,book_id,pdf_page,section}]`.
- **Don't**: merge CSVs into the repo; don’t fail `--check` when legacy path absent on CI.
- **Accept**: `--dual shape grammar` shows PCG-06 **and** `shape_grammars` · `--check` still `0 problems`.

### T1 ⚙ Crosswalk + systems table — V3 C1 R1 · Needs T0
- **Files**: `src/dev/ref_books_index/systems.md` (short § Dual), `adoption.tsv` note column optional
- **Do**: document book-id crosswalk + “cite both” rule; list DBG/IO/PCG dual pairs from §2.
- **Accept**: README mentions both roots; `--dual --code construction/voxel_massing.rs` returns ≥1 NEW + ≥0 LEGACY.

### T2 ⚙ Gaps brief for tools + voxels — V4 C2 R1
- **Files**: `lookup.py` or `tools/mcp/python/tests/test_ref_books_index.py`
- **Do**: `--gaps` annotation: tag paths under `tools/` and `*voxel*|*massing*|*atmosphere*` as `tools` / `voxel` / `atmos` buckets in output.
- **Accept**: `--gaps` shows tagged lines · pytest still green.

---

## 5. Wave V1 — tools suite (Studio DCC)

Owner default `@coder-mcp` / `@designer-mcp` unless Files are under `src/`.

### S0 🧠 Parametric frame contract — V5 C3 R2 · Needs T0
- **Refs**: NEW IO-09 · ECS-15 · PCG-06 · LEGACY `asset_pipeline` · `shape_grammars` · `ai_tools`
- **Files**: `tools/mcp/schemas/` (frame schema), `studio_task` docs, [`plan_artist_studio_expansion_v1.md`](plan_artist_studio_expansion_v1.md)
- **Do**: define **ParamFrame** JSON (seed, grammar chain, modifiers, connection graph ids, massing intent) as StudioTask payload family; Studio edits frames without requiring bpy.
- **Don't**: mirror every Blender knob; don’t write sim occupation from Studio.
- **Accept**: schema validate · one APS panel round-trip writes/reads a frame · dual-cite in commit.

### S1 ⚙ Grammar / modifier / connection browsers — V5 C3 R2 · Needs S0
- **Refs**: PCG-06 · PCG-08 · DES-04 · `pcg` / `shape_grammars`
- **Files**: APS panels under `tools/mcp/art_pipeline_suite/`
- **Do**: expose generative grammars, modifiers, connection logic as Studio tools (open other grammars without live backend call).
- **Accept**: panel tests · generate-and-test path still gates promote.

### S2 ⚙ Massing / voxel studio path — V5 C3 R2 · Needs S0 · **voxels in scope**
- **Refs**: PCG-06 · RND-09 · LGT-04 · `geometry_visual_effects`
- **Files**: `voxel_massing_panel.py`, `tools/studio_dash/`, massing schemas, export → `massing_preview.json`
- **Do**: deepen Studio massing/voxel param frames → preview HTML + engine override path; escalate to Blender only for mesh that frames can’t express.
- **Don't**: skip engine sole writer of `massing_columns`.
- **Accept**: export validates · demo env override documented · dual-cite.

### S3 ⚙ StudioDash + mission adapters — V4 C2 R1
- **Files**: `tools/studio_dash/`
- **Do**: frame QC + massing QC in static HTML; keep sim_review pattern.
- **Accept**: render_report pytest / smoke.

### S4 ⚙ MCP job parity (escalate path) — V4 C2 R1 · Needs S0
- **Refs**: IO-09 · `pipeline_app_layer`
- **Files**: `tools/mcp/`, Blender addon
- **Do**: StudioTask → headless/addon uses **same** schemas; optional escalate from frame.
- **Accept**: job schema tests · addon send/poll still works.

### S5 ⚙ Validator + CLI briefs — V3 C1 R1
- **Files**: `tools/validators/`, MCP `validate_report` wrappers
- **Do**: massing + frame + grammar validate-report paths documented; agents use structured reports only.
- **Accept**: one validate-report for massing/frame returns JSON status.

---

## 6. Wave V2 — debug

### D0 ⚙ Dump counters for atmosphere + massing/voxel — V5 C2 R1
- **Refs**: DBG-02 · LGT-08 · LEGACY `debug_tools`
- **Files**: `src/dev/sim_run_dump*.rs`, `tools/sim_review/score_dump.py`
- **Do**: counts-only `atmosphere.*` (per shared-atmosphere plan) + `massing_*` / voxel presentation counters; matrix rows; no `*_green`.
- **Accept**: dump JSON keys present under env dump · score_dump doesn’t crash.

### D1 ⚙ GPU / VFX capture path — V4 C2 R1
- **Refs**: DBG-04 · DBG-07 · `render_debug_tools`
- **Files**: `render/probes`, `tools/sim_review/playcheck.py`
- **Do**: document + harden `VFX_CAPTURE` / voxel view capture for PlayQuad / PixelWorld weather+massing.
- **Accept**: playcheck or lib probe test path green · dual-cite.

### D2 ⚙ Pause / step for review — V3 C2 R1
- **Refs**: DBG-06 · `debug_cameras_pause`
- **Files**: `systems/sim_control.rs` (bounded)
- **Do**: ensure pause keeps render/UI live for Studio/playcheck stills.
- **Accept**: lib test or documented env flag.

### D3 ⚙ PlayQuad / tools suite playcheck recipe — V4 C2 R1
- **Files**: `tools/sim_review/README.md`, harness docs
- **Do**: one recipe: PlayQuad + voxel view + weather + massing dump → score.
- **Accept**: README commands run (or dry-run documented).

---

## 7. Wave V3 — voxels (full scope)

**Explicit:** voxels are **in scope**. Presentation **and** the path toward sim occupation are upgrade targets. `DR-VOXEL-OCCUPATION` gates **only** the moment `FootprintMatrix` is replaced as sim authority — charter + conflict matrix required before that cut, but **do not** stall Studio/engine voxel presentation or tool exports on that DR.

### X0 ⚙ Engine massing consumer hardening — V5 C3 R2 · Needs S2
- **Refs**: PCG-06 · ECS-15 · RND-01/09
- **Files**: `construction/voxel_massing.rs`, ghost derive, pixel_world / world_map consumers
- **Do**: sole writer `massing_columns`; Studio override hydrate; RO consumers.
- **Accept**: lib tests · no second FootprintMatrix.

### X1 🧠 Occupation charter (unblock DR when ready) — V5 C4 R3 · Needs X0
- **Files**: new `plan_voxel_occupation_v1.md` (planner), conflict matrix vs roads/zones/`RepresentationResult`
- **Do**: sign authority map for voxel occupation; set `DR-VOXEL-OCCUPATION` unblock predicates honestly.
- **Don't**: implement replacement in this slice — charter only.
- **Accept**: signed plan + EV/Cx note.

### X2 ⚙ Occupation implement (after X1 sign-off) — V5 C5 R4 · Needs X1
- **Do**: single writer migration per charter; save/hydrate rules; dump counters.
- **Accept**: charter Accept list · dual-cite · user sign-off if world-changing.

### X3 ⚙ Flora / cover voxel parity tools — V4 C3 R2
- **Refs**: PCG-05 · tactical↔voxel extract
- **Files**: vegetation extract / pixel_world flora · APS veg tools as needed
- **Do**: parametric instances; Studio-visible params where useful.
- **Accept**: lib tests.

### X4 ⚙ Atmosphere ↔ voxel presentation (tools + dump) — V4 C3 R2
- **Refs**: LGT-04 · shared atmosphere W6 when ready
- **Files**: pixel_world clouds/rain, gpu field bridge (coord with atmosphere plan)
- **Do**: frame-fed presentation; one upload path; dump counters.
- **Accept**: atmosphere plan Accept subset · no second smoke cache.

### X5 ⚙ RND-09 instanced massing batches (when count grows) — V3 C3 R2 · Needs X0
- **Refs**: RND-09 · `geometry_visual_effects`
- **Do**: instanced column batches if profiling says so.
- **Accept**: visual_perf / playcheck no regress beyond T6 rule.

---

## 8. Wave V4 — remaining open ★ (engine), dual-cite

**CLOSED** for cuttable OPEN ★ (2026-11-15). `--open` now returns only intentional DEFER/PARTIAL. Keep v2 §8 DEFERs (MOVE-*, TAC-14, PCG-03 world, RND-06/07, LEARN-*, DEC-04) until triggers — **PCG-07** when town-growth asks; **TAC-03** only under a new dense+WGSL charter.

---

## 9. Global rejects

- Single-matrix orient.
- Locking voxels “out of scope” / treating presentation-only as a permanent ban.
- Second massing or occupation authority.
- Studio/UI writing sim occupation without X1 charter.
- Parallel weather/smoke caches (atmosphere plan).
- Test-shaped production (`*_green`, witness resources, play debug panels).
- New Python deps without user yes; book text committed.

## 10. Ledger (seed)

| Id | Wave | Status | Evidence |
|:---|:---|:---|:---|
| T0 | V0 | ADOPTED | `--dual shape grammar` → PCG-06 + `shape_grammars`; `--check` 0 problems |
| T1 | V0 | ADOPTED | `systems.md` § Dual (book-id crosswalk + cite-both + DBG/IO/PCG pairs); README both roots; Accept: `--dual shape grammar` → PCG-06 + `shape_grammars`; `--dual --code construction/voxel_massing.rs` → 1 NEW + 0 LEGACY; `--check` 0 problems |
| T2 | V0 | ADOPTED | `--gaps` `[tools]`/`[voxel]`/`[atmos]` tags + gap buckets section; pytest test_ref_books_index green |
| S0 | V1 | ADOPTED | `param_frame_v1.schema.json` + example; `kind: param_frame` on StudioTask; `pytest test_param_frame.py` green; dual-cite IO-09 · ECS-15 · PCG-06 + `asset_pipeline` · `shape_grammars` · `ai_tools` |
| S1 | V1 | ADOPTED | `param_frame_studio.py` + `param_frame_panel.py` (grammar/modifier/connection browse, no bpy); `pytest test_param_frame_s1.py` green; PCG-08 gate on enqueue; dual-cite PCG-06 · PCG-08 · DES-04 + `pcg` · `shape_grammars` |
| S2 | V1 | ADOPTED | `massing_from_param_frame.py` + APS ParamFrame export + `studio_task.run_queued_param_frame`; `massing_preview_v1` param_frame_id/dual-cite; `demo_massing_override.md` env override; Accept: `pytest test_massing_from_param_frame.py` green · `validate-report massing_preview debug_runs/studio_dash/massing_preview.json` passed · `studio-task-enqueue --kind param_frame --run` done; dual-cite PCG-06 · RND-09 · LGT-04 + `geometry_visual_effects` |
| S3 | V1 | ADOPTED | frame_qc + massing_qc HTML (`tools/studio_dash/`); `pytest tools/studio_dash/tests/test_render_report.py -q` → 7 passed; `python tools/studio_dash/qc_smoke.py` → `debug_runs/studio_dash/frame_qc_smoke.html` + `massing_qc_smoke.html`; dual-cite IO-09 · ECS-15 · PCG-06 + `asset_pipeline` · `shape_grammars` · `overlays_ui` |
| S4 | V1 | ADOPTED | `studio_task.escalate_from_param_frame` + `studio-task-escalate` CLI; enqueue schema validates geometry/massing/assembly/param_frame; addon `escalate_param_frame` + poll parity README; `pytest test_studio_task_escalate.py` + addon package test green; dual-cite IO-09 · `pipeline_app_layer` |
| S5 | V1 | ADOPTED | `validators/studio_dcc.py` + `run_validator` param_frame/massing_preview/massing/building_grammar; `tools/validators/README.md` brief; `pytest test_studio_dcc_validators.py` green; Accept: validate-report param_frame + massing_preview → `status: passed` JSON; dual-cite IO-09 · ECS-15 · PCG-06 + legacy shape_grammars |
| D0 | V2 | ADOPTED | `atmosphere.*` + `presentation.massing_*` in `sim_run_dump*.rs`; matrix rows; `score_dump.py` section; dual-cite DBG-02 · LGT-08 · LEGACY `debug_tools` |
| D1 | V2 | ADOPTED | `render/probes/vfx_capture_env.rs` + playcheck `--view`/`--scene playquad`; Accept: `cargo test -p proc_A_dine01 --lib vfx_capture_env` → 4 passed; dual-cite **DBG-04** + **DBG-07** + LEGACY `render_debug_tools` (gea_v2_e4 p248) |
| D2 | V2 | ADOPTED | DBG-06 · LEGACY `debug_cameras_pause` (gea_v1_e4 p582); `systems/sim_control.rs` — gameplay gated on `should_tick`/`dt_scale`, render+UI on Bevy `Time`; `RUST_ENGINE_SIM_PAUSE=1` PreStartup latch; `queue_steps`; Accept: `cargo test -p proc_A_dine01 --lib systems::sim_control::tests` → 4 passed |
| D3 | V2 | ADOPTED | `tools/sim_review/README.md` § PlayQuad recipe + `test_harness.rs` module doc; dual-cite DBG-04 · DBG-02 · LGT-04; Accept: `playcheck.py --help` last line `--min-secs`; `score_dump.py fixtures/honest_dump.json` → `wrote debug_runs/sim_review/dashboard.html` |
| X0 | V3 | ADOPTED | dual-cite PCG-06 · ECS-15 · RND-01/09 + legacy `shape_grammars` · `pipeline_app_layer` · `geometry_visual_effects`; Accept: `cargo test -p proc_A_dine01 --lib voxel_massing` → 11 passed; `--dual --code construction/voxel_massing.rs` → 1 NEW + 0 LEGACY; sole writer `sync_voxel_massing_visual_requests` + studio hydrate + RO world_map/pixel_world; no `FootprintMatrix` replace |
| X1 | V3 | ADOPTED | `plan_voxel_occupation_v1.md` SIGNED — conflict matrix roads/zones/`RepresentationResult` + sole writer `VoxelOccupationBook` + RON `VoxelOccupationSnapshot` hydrate order; DR partial unblock (steward EV/Cx before X2); dual-cite **PCG-06** · **ECS-15** + LEGACY `shape_grammars` · `asset_pipeline`; Accept: charter `src/dev/plan_voxel_occupation_v1.md` + EV/Cx MED-HIGH |
| X2 | V3 | ADOPTED | X2-P0–P3: sole voxel writer under flag · tile `replace_from_voxel` · massing RO `z_layers` intent · `occupation.*` dump counts; dual-cite **PCG-06** · **ECS-15**; flag default **off**; Accept `voxel_massing` + `voxel_occupation` · `validate_cargo_report` error_count **0** |
| X3 | V3 | ADOPTED | dual-cite PCG-05 + LEGACY `l_systems`; VEG-SHAPE-001 cascade (`veg_species_v1.ron` → `VegExtractRow.species` → `sample_row_instances` → tactical symbols / voxel tiers); Accept: `cargo test -p proc_A_dine01 --lib -- vegetation_species world_map_vegetation_mesh pixel_world::layers` → 20 passed; `pytest test_veg_species_table.py` → 2 passed; `run_validator veg_species` → passed; residual VEG-SHAPE-D* (no `massing_columns` touch) |
| X4 | V3 | ADOPTED | LGT-04 (+ shared W6); frame-fed PixelWorld clouds/rain + gpu field bridge; one upload path; dump `gpu_field_upload_paths`/`presentation_frame_revision`; dual-cite LGT-04 skies/clouds/volumetrics (gea4v2 12.5.13) · LEGACY unavailable keywords |
| X5 | V3 | ADOPTED | RND-09 · LEGACY `geometry_visual_effects` (systems.md dual); `MASSING_INSTANCE_BATCH_THRESHOLD=64` + `MassingColumnInstance` + `pack_massing_instances`; sole writer packs `massing_instances`; pixel_world/world_map prefer instances when non-empty; Accept: `cargo test -p proc_A_dine01 --lib voxel_massing` → **14 passed**; `validate_cargo_report` error_count **0** |
| — | atmos | DONE | `plan_shared_atmosphere_weather_fire_v1` **W1–W7 DONE** (2026-11-15); sole L0 advect; frame-fed GPU; second_writer_violations 0 |
| V4-L03 | V4 | ADOPTED | LOOP-03 · LEGACY `game_loop`/`time_measurement` (gea_v1_e4 p522/p514); `clamp_sim_frame_delta_secs` + `SIM_FRAME_DT_CLAMP_SECS` (= `SIM_STEP_MAX_PER_FRAME`) in `sim_frame_delta.rs`; `advance_sim_tick` → `sim_delta_micros_from_frame`; Accept: `cargo test -p proc_A_dine01 --lib sim_frame_delta` → **4 passed** · `sim_control` → **6 passed**; `validate_cargo_report` error_count **0** |
| V4-L04 | V4 | ADOPTED | LOOP-04 · LEGACY `time_measurement` (gea_v1_e4 p522/p138); `SimTick`/`SimTimeMicros` u64 truth + `sim_time_secs_display` HUD-only; test `u64_tick_and_micros_remain_exact_when_f32_display_collapses`; dual-cite in `sim_control.rs` module docs; Accept shared with V4-L03 |
| V4-I04 | V4 | ADOPTED | INPUT-04 · ECS-10 · LEGACY `hid`; UI→surface→world capture (`InputCaptureOwner`); pointer-gate fold after Publish; map camera respects `world_gameplay_input_disabled`; Accept `input_routing` → **6 passed** |
| V4-L11 | V4 | ADOPTED | LOOP-11 · LEGACY `game_object_update` (gea_v1_e4 p511 / gea_v2_e4 p606); `throughput_solve_unchanged` + `network_flow_digest_changed`/`network_flow_chunk_needs_solve`; Accept: `cargo test -p proc_A_dine01 --lib loop11_revision_gate` → **10 passed** · `solver_skips_an_unchanged` → **1** · `network_flow::loop11` → **6**; `validate_cargo_report` error_count **0** |
| V4-L06 | V4 | ADOPTED | LOOP-06 · LEGACY `game_object_update` (gea_v2_e4 p606); `configure_sim_dependency_tier_edges` env→logistics→combat→AI (`Weather`→`NetworkFlow`→`ZoneAndReadModel`→`FactionAiSet`); Accept: `cargo test -p proc_A_dine01 --lib chunk_environment_set` → **3 passed**; `validate_cargo_report` error_count **0** |
| V4-IO03 | V4 | ADOPTED | IO-03 · LEGACY `memory_management` (gea_v1_e4 p412) · `world_streaming` (gea_v2_e4 p589); `spawn_reserve_for_batch` + `clamp_hydrate_batch_by_spawn_budget` gate hydrate/reconstruct; RemoteAllocator reserve uses budget; Accept: `entity_reserve` + `streaming::budget` lib tests · `validate_cargo_report` error_count **0** |
| V4-IO11 | V4 | ADOPTED | IO-11 · INPUT-03 · LEGACY `engine_configuration` (gea_v1_e4 p454); `ConfigStoreKind` + `EngineGameTuning` cvars vs per-user `InputBindings`; save-on-rebind; Accept: `cargo test -p proc_A_dine01 --lib input_bindings` → **7 passed**; `validate_cargo_report` error_count **0** |
| V4-ECS08 | V4 | ADOPTED | ECS-08 · CONC-01 · LEGACY job systems / async GO update (gea_v2_e4 p626); `apply_stream_io_completion` + `apply_edge_fire_deposits` / `apply_one_edge_deposit` main-thread sole apply; Accept: `cargo test -p proc_A_dine01 --lib task_pool::tests` + `edge_deposits_apply_on_main_thread` · `validate_cargo_report` error_count **0** |
| V4-ECS15 | V4 | ADOPTED | ECS-15 · LEGACY IF-TRG/SIM-TRN; transport tool→runtime docs; Studio `parse_massing_preview_json` refuses `sim_occupation: true`; tripwire `ecs15_studio_path_cannot_write_sim_occupation` + lane_b source guard; dual-cite gea4v2 16.2.2; Accept: `cargo test -p proc_A_dine01 --lib voxel_massing` + `voxel_occupation` · `validate_cargo_report` error_count **0** |
| V4-CONC09 | V4 | ADOPTED | CONC-09 · LEGACY async / job / streaming (gea_v2_e4 p621-624); dual-cite gea4v2 17.7.2 / 17.7.2.1; request half (`SaveIoDispatcher::submit` / `submit_stream_hydrate_work`, non-blocking) vs consume half (new `apply_save_io_completion` via `poll_save_io_completions`; `apply_stream_io_completion`; `try_recv` only); `hydrate.rs` documented as worker-side body, sync only via opt-in `STREAMING_SYNC_HYDRATE`; tripwires `submit_before_start_is_rejected_not_blocking` · `request_does_not_apply_until_main_thread_poll` · `failed_completion_does_not_enqueue_apply_jobs` · `hydrate_request_issued_early_products_applied_a_frame_later`; Accept: `cargo test -p proc_A_dine01 --lib io::save::async_io` → **3 passed** · `io::streaming::hydrate` → **2 passed** · `io::streaming::task_pool` → **3 passed**; `validate_cargo_report` error_count **0** |
| V4-ECS11 | V4 | ADOPTED | ECS-11 · LEGACY `events_message_passing` (gea_v2_e4 p632) · `event_managers` (aig_e3 p859) interest/subscribe; dual-cite gea4v2 17.8.8 + aig3 11.2.2; `registered_priority_for_chunk`/`chunk_has_registered_interest`/`retain_registered_chunks` = sole interest gate (`None` = unsubscribed), `priority_for_chunk` now O(1) via `orb_registers_chunk` (no window Vec alloc); hysteresis R+H superset stays post-gate (IO-02); Accept: `cargo test -p proc_A_dine01 --lib interest` → **14 passed** · `io::streaming` → **46 passed**; `validate_cargo_report` error_count **0** |
| V4-ECS10 | V4 | ADOPTED | ECS-10 · LEGACY hid; gea4v2 17.8.7 chain of responsibility; `InputCaptureOwner` UI→surface→world + ecs10_* consume tripwires; Accept: `--lib ecs10` → **13 passed** · `input_routing` → **9 passed**; `validate_cargo_report` error_count **0** |
| V4-CONC04 | V4 | ADOPTED | CONC-04 · LEGACY `lock_free` (gea_v1_e4 §4.9 p285); gea4v1 4.9.4.6/4.9.4.7/4.9.5.1/4.9.7.4; audit: `NEXT_ID` all `Relaxed` (pure monotonic counter, no published memory → no happens-before edge); module docs in `idgen.rs`; tripwires `idgen_atomics_stay_relaxed` + `relaxed_counter_unique_across_threads_and_monotonic_per_thread`; debt not taken: u32 wrap at 2^32; Accept: `cargo test -p proc_A_dine01 --lib idgen` → **2 passed**; `validate_cargo_report` error_count **0** |
| V4-ECS12 | V4 | ADOPTED | ECS-12 · LEGACY `events_message_passing`; dual-cite gea4v2 17.8.11.1–2; pathway resolve → emit intent only (no sim write); `ecs12_*` tripwires; Accept: `cargo test -p proc_A_dine01 --lib -- scenario:: --skip ai::faction` → **22 passed**; `validate_cargo_report` error_count **0** |
| V4-CONC10 | V4 | ADOPTED | CONC-10 · LEGACY `concurrency`/`concurrent_object_updates` (gea_v2_e4 p620-625); dual-cite gea4v2 17.7.3; `ComputeStage` + `COMPUTE_STAGE_EDGES` (FireInfluence→HeatPlan→HeatAdvance→NavSync) = declared dependency graph, acyclic `debug_assert` in plugin build; `compute_critical_path_len` / `compute_graph_parallel_width` / `compute_parallel_width_for_cores` = min(cores, width) (chain ⇒ width 1 at 64 cores); `check_parallel_claim` rejects claim > width; cyclic edges → `None`; re-exported via `compute/mod.rs` (no new dead-code warnings); Accept: `cargo test -p proc_A_dine01 --lib compute_dispatch_graph` → **8 passed** (5 `conc10_*`); `validate-report cargo` compress 4 error_count **0** (MCP `validate_cargo_report` timed out; CLI equivalent used) |
| V4-CONC08 | V4 | ADOPTED | CONC-08 · LEGACY `simd` (gea_v1_e4 §4.10 p324; gea_e3 p350); gea4v1 4.10.6–7 p335-340; `cell_spread.rs`: `spread_deposit_mask` (all gates non-short-circuit, NaN-admit contract) + select-style writes in `spread_with_gain`; `relax.rs`: `slot_live` mask + `step_once_lanes::<SKIP_DEAD>` (production = early-out, `false` = compute-then-select), dead lane bit-identical / no conquest credit; tripwires `conc08_predicated_spread_matches_scalar_on_gate_lattice` · `conc08_predicated_step_matches_early_out`; debt not taken: no real SIMD lane kernel (scalar CPU early-out stays faster on sparse grids), `chunk_fire_overlay` apply path untouched; Accept: `--lib cell_spread` → **28 passed** · `--lib relax` → **11 passed** · `--lib conc08` → **2 passed**; `validate_cargo_report` error_count **0** (142 pre-existing warnings, none new) |
| V4-MEM07 | V4 | ADOPTED | MEM-07 · LEGACY `memory_management` (gea_v1_e4 §6.2 p412; no dedicated hashed-id keyword); gea4v1 6.4.3/6.4.3.1/6.4.3.2 (p443-446); new `utils::StringId` (const FNV-1a 64, no pointer identity/no RandomState seed); `CargoKindRegistry.by_id` + `SfxCatalog.by_id/cues` keyed by id with `get_by_id`, collision rejected at load/insert; tripwires `mem07_*` (cargo ×2, sfx ×2) + `string_id` reference vectors ×3; debt not taken: `AudioCuePicker.last_index` still `&str`-keyed, RON files still keyed by string (ids derived at load); Accept: `cargo test -p proc_A_dine01 --lib mem07` → **4 passed** · `cargo_registry` 4 · `sfx_catalog` 4 · `string_id` 3 · `audio::` 13 passed; `validate_cargo_report` compress 4 error_count **0** |
| V4-MEM05 | V4 | ADOPTED | MEM-05 · LEGACY `memory_management` (gea_v1_e4 p412); gea4v1 6.3.5; `SLAB_GROW_QUANTUM=64` + `chunk_multiple_capacity` + quantum insert/reserve; tripwires `mem05_*`; Accept: `--lib slab` → **9 passed** · `--lib mem05` → **5 passed**; `validate_cargo_report` error_count **0** |
| V4-MEM02 | V4 | ADOPTED | MEM-02 · LEGACY `memory_layout` (gea_v1_e4 §3.3 p135; no `alignment`/`packing` keyword hit); gea4v1 3.3.7.1–2 p162-168; `gpu_packed_formats.rs`: module-doc std430-friendly contract (repr(C)+Pod, vec4-lane-only rows, stride = size_of = k·16), `GPU_ROW_ALIGN` + `is_aligned_row_stride` + `ALL_PACKED_FORMATS` table with compile-time `const _` assert; tripwires `mem02_row_sizes_are_pinned` · `mem02_field_offsets_are_vec4_lanes` · `mem02_format_table_is_aligned_and_unique`; debt not taken: no WGSL-side struct parse (CPU/Rust contract only), registry allocation sites still call per-format fns rather than the table; Accept: `cargo test -p proc_A_dine01 --lib gpu_packed_formats` → **9 passed**; `validate_cargo_report` compress 4 error_count **0** (142 warnings, none new) |
| V4-MEM04 | V4 | ADOPTED | MEM-04 · LEGACY `memory_management` (gea_v1_e4 p412); gea4v1 6.2.2; fixed page multiples + handle/`version` relocate (`gpu_buffer_registry`) + field_page residency pool; `mem04_*` tripwires; Accept: `--lib mem04` → **7** · `gpu_buffer_registry` → **6** · `field_page_residency` → **8**; error_count **0** |
| V4-MEM06 | V4 | ADOPTED | MEM-06 · LEGACY closed hashing; gea4v1 6.3.6; solver SoA/index hot path + banned hash probes; `mem06_*`; Accept: `--lib mem06` → **3 passed**; error_count **0** |
| V4-RND10 | V4 | ADOPTED | RND-10 · LEGACY pipeline PSO; gea4v2 11.4.8.4–5; queue only in `init_*pipeline*` on `RenderStartup`; `rnd10_*` tripwires (floor ≥9); Accept: `--lib rnd10` → **5 passed**; error_count **0** |
| V4-RND11 | V4 | ADOPTED | RND-11 · LEGACY bindless/views; gea4v2 11.4.9; `get_or_build` sole cache; `rnd11_*`; Accept: `--lib gpu_bind_group_registry` → **7 passed**; error_count **0** |
| V4-MEM10 | V4 | ADOPTED | MEM-10 · LEGACY `memory_architectures` (gea_v1_e4 §3.5 p183); gea4v1 3.5.2/3.5.2.3–4; `ChunkSlab` page table+residency+fault-in (`ChunkKey` handles) + `field_page_residency` VA/`page_coord`→PA/`atlas_slot` + GPU TLB rebuild; `mem10_*` tripwires; MEM-04/05 contracts unregressed; Accept: `--lib slab` → **12** · `field_page_residency` → **12** · `mem10` → **7**; `validate_cargo_report` compress 4 error_count **0** |
| V4-MATH06 | V4 | ADOPTED | MATH-06 · LEGACY `math_3d` (gea_v1_e4 p350); gea4v1 5.1; `reduce_world_to_map_xy` / `lift_map_xy_for_presentation` + `sim_map_world_xy_to_screen_in_frame`; tripwires `math06_*`; Accept: `--lib sim_map_projection` → **8 passed** · `--lib math06` → **7 passed**; `validate-report cargo` compress 4 error_count **0** |
| V4-DBG06 | V4 | ADOPTED | DBG-06 · LEGACY `debug_cameras_pause` (gea_v1_e4 p582); gea4v1 10.5; pause freezes sim tick while Bevy Time advances + single-step; `dbg06_*` tripwires; Accept: `--lib sim_control` → **9 passed**; `validate-report cargo` error_count **0** |
| V4-ECS02 | V4 | ADOPTED | ECS-02 · LEGACY `runtime_object_model` (gea_v2_e4 §17.2 p563); gea4v2 17.2.2/17.2.2.3–4; dual-cite property-centric `ChunkSlab` pages + `FieldScalarLayer`/`ChunkFieldCell` AoS trade-offs; tripwires `ecs02_*`; MEM-05/10 unregressed; Accept: `--lib slab` · `world_field_layers` · `ecs02` · `validate_cargo_report` compress 4 error_count **0** |
| V4-RND03 | V4 | ADOPTED | RND-03 · LEGACY `geometry_visual_effects` (gea_v2_e4 §11.6 p141) · `pipeline_app_layer` (p133); gea4v2 11.5.2/11.5.3.1/11.5.3.3; `CORE2D_OVERLAY_DRAW_ORDER` sole painter order + `draw_rank`; fire last; `rnd03_*` tripwires; Accept: `--lib core2d_overlay_order` → **6 passed** · `--lib rnd03` → **6 passed**; `validate-report cargo` compress 4 error_count **0** |
| V4-RND04 | V4 | ADOPTED | RND-04 · LEGACY `geometry_visual_effects` (gea_v2_e4 p141); gea4v2 11.3.12/11.3.12.1; sort-not-OIT: `TRANSPARENT_ORDER_*` smoke→ember→spark + emit `sort_by_key` + `CLOUD_SHEET_LAYER_Z` + precip SrcAlpha/depth-write-off; read-only weather-under-fire overlay chain tripwire (did not dual-write RND-03 order file); `rnd04_*` tripwires; Accept: `--lib rnd04` → **8 passed**; `validate_cargo_report` compress 4 error_count **0**; debt not taken: A-buffer/weighted OIT |
| V4-DBG04 | V4 | ADOPTED | DBG-04 · LEGACY `render_debug_tools` (gea_v2_e4 p248); gea4v2 12.9.4; `vfx_capture_env` sole env authority (`VFX_CAPTURE`/`RTT_SPRITE_TRACE`/`VIEW=voxel`); visual_capture_probe defers; `dbg04_*` tripwires; Accept: `--lib vfx_capture` · `--lib dbg04` · `--lib probes::` · `validate-report cargo` compress 4 error_count **0** |
| V4-RND14 | V4 | ADOPTED | RND-14 · LEGACY `geometry_visual_effects` (gea_v2_e4 p141); gea4v2 11.6.4; cloud Rectangle+soft-puff cards + fire Smoke/AtmosphereFx instanced-quad billboards = intentional impostors (not ray-march); pin `CLOUD_SHEET_LAYER_Z`=[400,455,510] + `TRANSPARENT_ORDER_*` 0/1/2; `rnd14_*` tripwires; Accept: `--lib rnd14` → **8 passed** · `weather_vfx` → **15** · `fire_vfx` → **34**; `validate_cargo_report` compress 4 error_count **0**; debt not taken: true volumetrics (LGT-04) · no edit to `core2d_overlay_order.rs`/probes |
| V4-RND15 | V4 | ADOPTED | RND-15 · LEGACY `geometry_visual_effects` (gea_v2_e4 p141); gea4v2 11.6.6; surface-fire / overlay-heat scorchmark = terrain-projected **decal** layer (`surface_fire_scorch_decal01` · `overlay_heat_scorch_decal01`); seed skips residual band; pin `TRANSPARENT_ORDER_*` 0/1/2; `rnd15_*` tripwires; Accept: `--lib rnd15` → **13 passed** · `chunk_surface_fire` → **6 passed**; `validate_cargo_report` compress 4 error_count **0**; debt not taken: true projected-prism GPU decals · no edit to overlay order / probes / RND-14 card contracts |
| V4-PHYS07 | V4 | ADOPTED | PHYS-07 · LEGACY `physics_integration` (gea_v2_e4 §14.5 p423); gea4v2 14.5.3.2–4; blast intent→execute + falloff; sole `BlastCraterBook` writer; ballistics enqueue-only; voxel via demolish; `phys07_*` tripwires; Accept: `--lib blast` · `--lib phys07` → **9 passed**; `validate_cargo_report` compress 4 error_count **0**; debt not taken: PHYS-08 damage stages / no rigid-body middleware |
| V4-PHYS08 | V4 | ADOPTED | PHYS-08 · LEGACY `physics_integration` (gea_v2_e4 §14.5 p423); gea4v2 14.5.3.5; `DestructibleDamageStage` Intact→Scarred→Compromised→Collapsed + integrity/catastrophic gates; sole terminal `execute_demolish_at_tile`; voxel clear via `remove_site_commit` when commit auth; blast/fire/pending ingress tripwires; `phys08_*`; Accept: `--lib demolish` · `--lib phys08` → **7 passed**; `validate_cargo_report` compress 4 error_count **0**; debt not taken: rigid-body multi-piece / Havok Destruction / timed collapse propagation |
| V4-LGT06 | V4 | ADOPTED | LGT-06 · LEGACY `lighting_post`/`post_processing` (gea_v2_e4 p153 / §12.7 p244); gea4v2 12.7.1; `view_runtime/tonemap_hdr.rs` + `ToneMapHdrPass`; SDR map RTT + overlay `[SDR,HDR]` index + PixelWorld `Tonemapping::None`; `lgt06_*`/`tonemap_*`/`hdr_*` tripwires; Accept: `--lib lgt06` → **6** · `--lib tonemap` → **6** · `--lib hdr` → **7**; `validate_cargo_report` compress 4 error_count **0**; debt not taken: ACES/Reinhard · HDR float SimulationMap · auto-exposure |
| V4-PHYS04 | V4 | ADOPTED | PHYS-04 · LEGACY `collision_detection` · `physics_collision` (gea_v2_e4 §14.3 p359/p352); gea4v2 14.3.8/14.3.8.1/14.3.8.3; tags → `CollisionLayer`/`CollisionFilter` + parallel `CollisionMaterial` table keyed by `MaterialId` (fluid pass-through); voxel cells share same compact index; `phys04_*` tripwires; Accept: `--lib material::registry` → **12 passed** · `--lib phys04` → **4 passed**; `validate_cargo_report` compress 4 error_count **0**; debt not taken: rigid-body middleware / callback filters |
| V4-RND19 | V4 | ADOPTED | RND-19 · LEGACY gea_v2_e4 p50-51 · gea_v1_e4 p518-519 (no vsync keyword); gea4v2 11.3.4.1 · gea4v1 8.5.2.5; `PresentSurfaceConfig` in `render/view_runtime/present.rs`; sole writer `EnginePlugin` — debug AutoVsync+triple latency **3**, `PERF_NO_VSYNC`/`PERF_VSYNC=0`/release → AutoNoVsync+latency **1**; voxels share primary-window present (no PixelWorld PresentMode dual-write); `rnd19_*` tripwires; Accept: `--lib rnd19` → **6 passed**; `validate_cargo_report` compress 4 error_count **0**; debt not taken: UI toggle · force Fifo/Mailbox · per-view swapchains; did not edit tonemap (LGT-06) |
| V4-LGT03 | V4 | ADOPTED | LGT-03 · LEGACY `lighting_post`/`post_processing` (gea_v2_e4 §12 p153); gea4v2 12.5.10/12.7.2; `fire_light_emission` sole sim emissive + `fire_vfx` cite (particle self-glow ≠ light writer); heat/`HEAT_MIN`+biomass; defer tonemap to LGT-06 (`LGT03_TONEMAP_OWNER_TOKEN`); voxels share pooled lights; `lgt03_*` tripwires; Accept: `--lib fire_light_emission` → **7** · `--lib lgt03` → **5**; `validate_cargo_report` compress 4 error_count **0**; debt not taken: custom bloom · rewrite tonemap_hdr · per-voxel emissive ECS |
| V4-ANIM03 | V4 | ADOPTED | ANIM-03 · LEGACY `animation_compression` (gea_v2_e4 §13.8 p306); gea4v2 13.8.2–13.8.4; save `quantize_unit_interval_f32` + Identity-only codec (`anim03_*` on `wire_format`); snapshot fill/throughput key omission + telemetry quantize (`anim03_*` on `snapshot`); no AnimationCurve host — true keyframe curve compression deferred; did not edit `fire_light_emission`/`tonemap_hdr`; Accept: `--lib anim03` → **6 passed**; `validate_cargo_report` compress 4 error_count **0** |
| V4-LGT05 | V4 | ADOPTED | LGT-05 · LEGACY `lighting_post`/`post_processing` (gea_v2_e4 p153); gea4v2 12.1.2.2; `participating_media_visibility` + `AtmosphereCell` sole media sample DTO; denser smoke/fog/ash lowers unit visibility; shares smoke optical K with LOS (`visibility.rs`); no dual-write frame / no ray-march invent; `lgt05_*` tripwires; Accept: `--lib lgt05` → **5** · `--lib field` → **16**; debt not taken: true volumetric ray-march (LGT-04) · resurrect AtmosphereField |
| V4-PHYS09 | V4 | ADOPTED | PHYS-09 · LEGACY `physics_collision` · `physics_middleware` (gea_v2_e4 §14 p352 / §14.2 p358); gea4v2 14.1 / 14.1.2.1 / 14.1.3; ballistics = kinematic bookkeeping + BlastIntent enqueue (intent→execute) — **no** rigid-body middleware world; `phys09_*` tripwires; Accept: `--lib phys09` → **4 passed** · `--lib ballistics` as needed; `validate-report cargo` compress 4 error_count **0**; debt not taken: Rapier/Havok/PhysX host · PHYS-11 contact timing |
| V4-PHYS11 | V4 | ADOPTED | PHYS-11 · LEGACY `collision_detection`/`physics_collision` (gea_v2_e4 §14.3 p359/p352); gea4v2 14.3.6; `CollisionPlugin` + `CoherentPairCache` sole writer; warm→broad→mid→narrow; mid uses PHYS-04 filter + O(1) body map; voxels share filter path; Accept: `--lib phys11` → **4 passed**; debt not taken: SAT/GJK · rigid-body middleware |
| V4-NAV04 | V4 | ADOPTED | NAV-04 · LEGACY pathfinding/A* keywords; gea4v2/aig node-array A*; `LogisticsNodeArray` dense scratch + generation clear; dijkstra/concern uses node array (no HashMap per search); behavioral `nav04_*`; Accept: `--lib nav04` → **4 passed**; source-scan tripwire removed |
| V4-LGT10 | V4 | ADOPTED | LGT-10 · LEGACY `raster_lighting` (gea_v2_e4 §12.5 p198); gea4v2 12.5.3/12.5.4; sole `bake_weather_overlay_from_field` → Wind + CPU probe lattice; `sample_overlay_bake_probe_*` for dynamic objects; revision-gated republish; Accept: `--lib lgt10` → **2 passed** (+ `bake_publishes_wind_channel`); source-scan tripwires removed |
| V4-NAV06 | V4 | ADOPTED | NAV-06 · LEGACY `pathfinding` (aig_e3); aig3 4.4.6–4.4.7; axis-aligned string-pull [`smooth_logistics_tile_path`] / [`smooth_vehicle_tile_path`]; concern blend = cost fn; search metrics preserved; Accept: `--lib nav06` → **4 passed** |
| V4-MOVE03 | V4 | ADOPTED | MOVE-03 · LEGACY `steering` (aig_e3); aig3 3.3.13–3.3.14; `separation_steer` + `collision_avoidance_steer` + `combine_steering_for_depot_queue` in `potential_field_navigation_system`; Accept: `--lib move03` → **3 passed** |
| V4-LGT04 | V4 | ADOPTED | LGT-04 · gea4v2 12.5.13 skies/clouds/volumetrics; olumetric_haze_from_fog feeds precip visibility from LGT-05 media (W6/X4 frame-fed presentation); Accept: --lib lgt04 → **1 passed**; debt not taken: true ray-march volumes |
| V4-NAV08 | V4 | ADOPTED | NAV-08 · LEGACY `pathfinding` (aig_e3); aig3 4.6.3–4.6.4; portal attach exact→nearest within `PORTAL_ATTACH_MAX_CHUNK_MANHATTAN`; far portals excluded (no abstract latch); Accept: `--lib nav08` → **3 passed** |
| V4-NAV11 | V4 | ADOPTED | NAV-11 · LEGACY `pathfinding` (aig_e3); aig3 4.7.5–4.7.6; `advance_interruptible_solve_pass` + budgeted cursor resume; SoA pooled planner; Accept: `--lib nav11` → **2 passed** |
| V4-MOVE04 | V4 | ADOPTED | MOVE-04 · LEGACY `steering` (aig_e3); aig3 arbitration; `arbitrate_steering_pipeline` avoid→separate→seek (no force cancel); Accept: `--lib move04` → **2 passed** |
| V4-MOVE08 | V4 | ADOPTED | MOVE-08 · LEGACY `steering`; `filter_vehicle_steering_output` accel+turn caps (`VehicleMotionLimits`); Accept: `--lib move08` → **2 passed** |
| V4-MATH02 | V4 | ADOPTED | MATH-02 · LEGACY numeric mixing (aig3 8.1.1); `derive_keyed_seed`/`derive_chunk_seed` + worldgen layer offsets + `derive_golden_stream_seed`; Accept: `--lib math02` → **2 passed**; debt not taken: cross-domain HashMap (MEM-08 report-only) |
| V4-IO08 | V4 | ADOPTED | IO-08 · LEGACY post-load init (gea4v1 7.2.2.10); `rebuild_derived_after_chunk_apply` + `PostLoadDerivedBook` + `post_load_rebuild` strip/rebuild; Accept: `--lib io08_post_load` → **1** · `--lib post_load_rebuild` → **3 passed**; residual: legacy stand heat vectors may still deserialize |
| V4-DEC12 | V4 | ADOPTED | DEC-12 · LEGACY blackboard (aig3 5.9); \WorldReadSnapshot\ + \BlackboardWriteLane\ (StrategicCommand/MissionIntent); experts enqueue only; Accept: \--lib dec12\ → **1 passed** |
| V4-TAC02 | V4 | ADOPTED | TAC-02 · aig3 6.2.6 map flooding; flood_ownership_regions + derive_fronts; Accept --lib tac02 → 1 · front::tests → 9 |
| V4-TAC09 | V4 | ADOPTED | TAC-09 · aig3 6.1.5 condensation; condense_front_cells + FrontSegment::condensed_cells; Accept --lib tac09 → **1 passed** |
| V4-DEC13 | V4 | ADOPTED | DEC-13 · aig3 5.10 action execution; priority/interrupt/compound on DeploymentOrderQueue; Accept --lib dec13 → **1 passed** |
| V4-TAC15 | V4 | ADOPTED | TAC-15 · aig3 static evaluation; static_board_eval + BoardEval; Accept --lib tac15 → **1 passed** |
| V4-TAC07 | V4 | ADOPTED | TAC-07 · aig3 6.2.4 frag maps; TacticalFragMap loss heat; Accept --lib tac07 → **1 passed** |
| V4-DEC05 | V4 | ADOPTED | DEC-05 · aig3 5.5 fuzzy; fuzzy_degree/and/or + compose_decision_score_fuzzy; Accept --lib dec05 → **1 passed** |
| V4-DEC06 | V4 | ADOPTED | DEC-06 · aig3 5.6 Markov; markov_transition_row/step over WorldEvent; Accept --lib dec06 → **1 passed** |
| V4-DEC11 | V4 | ADOPTED | DEC-11 · aig3 5.8 Rete/arbitration; TriggerFactMemory + arbitrate_trigger_rules; Accept --lib dec11 → **1 passed** |
| V4-DEC04 | V4 | DEFER | DEC-04 · BT concurrency — no BT stack; read-gate only |
| V4-DEC09 | V4 | ADOPTED | DEC-09 · aig3 GOAP/IDA*; plan_goap over WorldAtoms; Accept --lib dec09 → **1 passed** |
| V4-TAC11 | V4 | ADOPTED | TAC-11 · aig3 group actions; script_group_package_intents; Accept --lib tac11_scripted → **1 passed** |
| V4-PCG02 | V4 | ADOPTED | PCG-02 · aig3 8.3.4 faults; fault_ridge_boost Gaussian ridge; Accept --lib pcg02 → **1 passed** |
| V4-PCG04 | V4 | ADOPTED | PCG-04 · aig3 8.3.7 altitude filtering; altitude_filter_elevations; Accept --lib pcg04 → **1 passed** |
| V4-EXEC02 | V4 | ADOPTED | EXEC-02 · aig3 interruptible scheduler; take_budgeted_dirty_regions; Accept --lib exec02 → **1 passed** |
| V4-EXEC03 | V4 | ADOPTED | EXEC-03 · aig3 hierarchical/priority scheduling; allocate_nested_domain_budgets; Accept --lib exec03 → **1 passed** |
| V4-SENSE03 | V4 | ADOPTED | SENSE-03 · aig3 sense management; SenseMask + perceive_faction_masked; Accept --lib sense03 → **1 passed** |
| V4-SENSE05 | V4 | ADOPTED | SENSE-05 · aig3 faking perception; PerceptionFidelity + perceive_with_fidelity; Accept --lib sense05 → **1 passed** |
| V4-DES03 | V4 | ADOPTED | DES-03 · gea4v1 startup order; ENGINE_STARTUP_PHASES; Accept --lib des03 → **1 passed** |
| V4-MOVE10 | V4 | ADOPTED | MOVE-10 · wildlife_flock seed herd + soft costs ([`plan_move10_wildlife_v1.md`](plan_move10_wildlife_v1.md)) |
| V4-DES06 | V4 | ADOPTED | DES-06 · aig3 ecosystem design; ecosystem_logistic_step; Accept --lib des06 → **1 passed** |
| V4-LEARN03 | V4 | DEFER | LEARN-03 · over-learning warnings — read-gate; no learning stack |
| CLOSE | — | **PROGRAM CLOSE** | 0 OPEN ★ · 117 ADOPTED · 2 PARTIAL LOCKED (EXEC-05 · TAC-03) · 14 DEFER keep · `--check` 0 problems · planner `PROGRAM_CLOSE_OK` (no UNDER_DEFERRED) |


## 11. Paste blocks (dispatch / resume)

**Fresh slice** (replace `<ID>`):

```text
Execute slice <ID> of src/dev/plan_ref_books_upgrade_v3.md.
Obey §0 stance + §0.1 honest exits + §1 dual-matrix + §9 rejects.
Voxels are IN SCOPE (DR-VOXEL is occupation-authority sequencing only).
Run dual orient before Do. Paste every Accept final line. Update §10.
If Accept fails twice: ledger BLOCKED:<reason> and in the SAME turn start the next OPEN slice whose Files do not overlap.
Do not ask the user whether voxels/matrices are in scope. Do not stop lazy.
```

**Resume after lazy/confused/response-stop exit:**

```text
RESUME slice <ID> of src/dev/plan_ref_books_upgrade_v3.md — prior exit was incomplete (§0.1).
Re-read §0 + §0.1. Finish Accept + §10 or BLOCKED+next slice. No clarifying questions on locks.
Do not ask the user anything. Keep cutting OPEN upgrade slices until legal stop.
```

**Wave keep-going (parent):** **PROGRAM CLOSE** — no OPEN §10 rows remain. Do **not** keep-cutting DEFER/PARTIAL. On compaction, do not re-litigate §0; route residual triggers to their owning plans (town-growth · unit-member · dense heat+WGSL · occupation productization).

**V4 keep-cutting order:** exhausted (cuttable ★ ADOPTED). Residual: skip DEFER/LOCKED until triggers fire.

## Changelog

| Version | Date | Notes |
|:---|:---|:---|
| 1.0.0 | 2026-11-15 | Signed. Dual matrices parallel-combined; tools suite; debug; voxels full scope. |
| 1.1.0 | 2026-11-15 | §0.1 anti-stall; resume/dispatch paste blocks; keep-going dispatcher rule. |
| 1.1.1 | 2026-11-15 | §0.1 response-stop + compaction tripwires; RESUME paste forbids user ping; keep-going priority on re-entry. |
| 1.1.2 | 2026-11-15 | X4 ADOPTED with shared-atmosphere W6: frame-fed GPU/PixelWorld presentation; LGT-04 dual-cite. |
| 1.1.3 | 2026-11-15 | §0.1 active-agent response-stop + polite-handoff/false-tripwire/ledger-drift; V4 keep-cutting order; parent must RESUME standing-by. |
| 1.1.4 | 2026-11-15 | §0.1 fake-tripwire tax — prefer behavioral Accept; stop parallel re-scan cargo churn. |
| 1.2.0 | 2026-11-15 | **PROGRAM CLOSE** — §2.1 live counts (117/2/14/0); V4 cuttable ★ closed; §11 keep-going retired; planner DEFER/PARTIAL keep. |
| 1.3.0 | 2026-11-16 | **REVIEWED** — §0.1 anti-stall replaced by honest exits; ADOPTED rows re-graded (UNWIRED status); counts stale (CQ-80). |
