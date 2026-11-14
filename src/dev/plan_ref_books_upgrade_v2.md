# PLAN-REF-BOOKS-UPGRADE-v2 — book index → engine, subsystems, tooling, studio

| Field | Value |
|:---|:---|
| **Id** | **PLAN-REF-BOOKS-UPGRADE-v2** |
| **Status** | **WAVES 0–2 CLOSED** — exit criteria met (F1/E7 PARTIAL with written reasons; Wave S studio still OPEN) |
| **Date** | 2026-11-14 |
| **Supersedes** | [`plan_ref_books_upgrade_use_v1.md`](plan_ref_books_upgrade_use_v1.md) (**SHIPPED**, built on the old `$REF_BOOKS_DIR\lookup\` CSVs — do not use those) |
| **Tool** | [`ref_books_index/`](ref_books_index/README.md): `systems.md` → `lookup.py` → `--read` |
| **Obeys** | `AGENTS.md`: one writer per domain · Sim → View → Render → UI never inverted · play UI only · no test-shaped production code |
| **Written for** | Any agent (Cursor, small models). Every slice is self-contained: do not plan, just follow the card. |

## 0. How to execute a slice (the whole protocol)

One slice per session. Pick the first `OPEN` slice in §3 order whose **Needs** are `ADOPTED`.

```text
1. ORIENT   python src/dev/ref_books_index/lookup.py --code <first file in the card>
2. RECON    run the card's Recon command. If the "Done-if" condition is true:
            set the ledger row (§9) to ADOPTED with the evidence line, commit, STOP.
3. READ     python src/dev/ref_books_index/lookup.py --sec <ref>     (page range only)
            python src/dev/ref_books_index/lookup.py --read <ref>    (only if the card's Do is not enough)
            Budget: one section, at most ~12 PDF pages. Never read a chapter.
4. DO       edit only the card's Files. Follow Do. Obey Don't.
5. ACCEPT   run every Accept command. All must pass.
6. LEDGER   update §9 status + evidence (file:line or test name). Commit: "[area] <slice id>: <what>".
```

**Stop and report instead of improvising when:** Recon output does not match what the card expects ·
the change needs a file outside **Files** · the diff passes ~300 changed lines · an Accept command fails
twice · a card says 🧠 and you are a small model (do Recon, write findings under the card, stop).

Markers: ⚙ = mechanical, safe for any agent · 🧠 = needs design judgement, use a strong model or @planner first ·
`V/C/R` = value / cost / risk, 1 (low) to 5 (high).

Paste-ready prompt for another agent:

```text
Execute slice <ID> of src/dev/plan_ref_books_upgrade_v2.md. Follow §0 exactly and §1 rejects.
Touch only the card's Files. Run Recon first and stop if Done-if holds. Run all Accept commands
and paste their final lines. Update the §9 ledger row. Do not start another slice.
```

## 1. Global rejects (apply to every slice)

- **No second authority.** A book idea attaches to the existing sole writer and SystemSet. Never add a parallel queue, book, fleet, ViewManager writer, extraction path or LOD stack.
- **No inversion.** Render/UI/audio read snapshots; they never write sim resources. Preview never mutates gameplay.
- **Determinism.** In sim dirs (`combat/ economy/ systems/ strategic/ ai/ construction/`): no wall-clock time, no `thread_rng`, no unordered `HashMap` iteration feeding results, fixed iteration counts, ties broken by stable id. Seeds derive from `(seed, key)` like `ai/faction/rng.rs`.
- **No test-shaped production code.** Checks live in `#[cfg(test)]`, `tools/validators/` or `tools/sim_review/`. No `*_green`, no witness resources, no dev panels or toggles.
- **No new crate or Python dependency** without the user saying yes. Stdlib-only for `lookup.py`.
- **No book text in the repo.** Cite `book:sec`; paraphrase in one or two lines. `--read` output is never committed.
- **Behaviour-changing tuning** (any constant that shifts sim outcomes) must be compared with T6 multi-seed scores once T6 exists; before that, state the before/after playcheck scores in the commit body.
- **Units are metric** in comments, UI and docs (m, km/h, kW/m², °C).
- **World-changing slices** (marked ⚠) alter generated worlds or save data: need explicit user sign-off before merge.

Owning plans — check the domain plan does not contradict the card before editing:
fire [`plan_fire_entity_exposure_v1.md`](plan_fire_entity_exposure_v1.md) · weather [`plan_weather_wind_v1.md`](plan_weather_wind_v1.md) ·
logistics [`plan_logistics_orders_v1.md`](plan_logistics_orders_v1.md), [`plan_haul_amplify_v1.md`](plan_haul_amplify_v1.md) ·
combat [`plan_front_combat_v1.md`](plan_front_combat_v1.md) · growth [`plan_town_growth_v1.md`](plan_town_growth_v1.md) ·
schedule [`plan_schedule_sync_v1.md`](plan_schedule_sync_v1.md) · studio [`plan_aps_refactor_v1.md`](plan_aps_refactor_v1.md).

## 2. What v1 already shipped (do not redo)

`SimAttentionFrame` + `AttentionTier` (`systems/sim_attention.rs`) · `MissionIntentQueue` → `MissionBook` sole ingress ·
sensory stack (SFX, blast funnel) · missile catalog and picker · haul-amplify on order-sim · four-layer schedule graph
documented in `.cursor/skills/bevy-simulation-grade/07-repo-authority-map.md`.
Verified in code on 2026-11-14 (see §9): site-stage transition table, CA fire spread on a snapshot, AI sub-seed hashing,
field-layer epoch, map-camera smoothing pipeline.

## 3. Pick order

| Wave | Slices | Why this order |
|:---|:---|:---|
| **0 Tooling** | T1 → T2 → T3 → T4 → T5 → T6 → T7 | Cheap, makes every later slice cheaper and measurable |
| **1 Engine-wide** | E1 → E4 → E6 → E2 → E3 → E5 → E7 | Guards first (E1, E4), then behaviour |
| **2 Subsystems** | C1 → F2 → L1 → U1 → C2 → F1 → A1 → A2 → L3 → W1 → L6 → F3 → L5 → L2 → G2 → F4 → W3 → U2 → U3 → R2 → K2 → R3 | Correctness bugs first (C1, F2), then highest value/cost |
| **S Studio** | S1 → S3 → S4 → S6 → S2 → S5 | Parallel lane, owner `@coder-mcp`; no overlap with `src/` |
| **3 Future features** | §8 | Each needs a @planner plan before code |

Wave S and Wave 0 can run in parallel with anything. Within Waves 1–2, two agents must never hold slices that share a file.

---

## 4. Wave 0 — tooling that multiplies the index

### T1 ⚙ `lookup.py` machine output — V4 C1 R1
- **Why**: agents should parse results, not read prose. Cuts tokens per lookup.
- **Files**: `src/dev/ref_books_index/lookup.py`, `src/dev/ref_books_index/README.md`
- **Do**: add `--json` (works with search, `--code`, `--niche`; emits `[{id,tier,concept,refs:[{book,sec,start,end,title}],code,take}]`), `--row <ID>` (one row with refs), `--brief <code path>` (max 12 lines: row ids, one best ref each, and the exact `--read` command).
- **Don't**: add dependencies; change the existing text output (other docs quote it).
- **Accept**: `python src/dev/ref_books_index/lookup.py --json fire spread | python -c "import json,sys; print(len(json.load(sys.stdin)))"` prints a number > 0 · `python -I src/dev/ref_books_index/lookup.py --check` → `0 problems`.

### T2 ⚙ Adoption ledger in the tool — V5 C2 R1 · Needs T1
- **Why**: stops agents re-implementing what exists; turns the matrix into a backlog.
- **Files**: new `src/dev/ref_books_index/adoption.tsv` (`id<TAB>status<TAB>evidence<TAB>date`), `lookup.py`, `README.md`
- **Do**: seed the TSV from §9 of this plan. Statuses: `OPEN ADOPTED PARTIAL DEFER REJECTED`. Show `[STATUS]` after the row id in all outputs (rows absent from the TSV show nothing). Add `--open [DOMAIN]` = ★ rows that are not `ADOPTED`/`REJECTED`. `--check` fails on a TSV id that is not in the matrix and on an unknown status.
- **Accept**: `--check` → `0 problems` · `python src/dev/ref_books_index/lookup.py --open NAV` lists rows · `--row ANIM-02` shows `[ADOPTED]`.

### T3 ⚙ Read cache + targeted grep — V4 C2 R1
- **Why**: `--read` re-runs pdftotext and dumps whole sections; most questions need three paragraphs.
- **Files**: `lookup.py`, `README.md`
- **Do**: cache `--read` text at `debug_runs/ref_books_cache/<book>_<sec>.txt` (already git-ignored by `/debug_runs/**`); add `--grep <regex> <book:sec>` printing matching lines with 2 lines of context and the PDF page; add `--pages` to `--read` to print only the page range and line count.
- **Don't**: write the cache anywhere tracked by git.
- **Accept**: run `--read aig3:6.2.8` twice, second run does not spawn pdftotext (add a `cached` note on stderr) · `git status --short` shows no new files under `debug_runs/`.

### T4 ⚙ Wire the index into agents — V5 C2 R1 · owner `@coder-mcp`
- **Files**: new `.cursor/rules/ref-books.mdc`, new `.claude/skills/ref-books/SKILL.md` (then sync to `.cursor/skills/` with the existing `sync-claude-skills` skill), `tools/mcp/python/rust_engine_mcp/cli.py`
- **Do**: rule text (≤ 25 lines, globs `src/**/*.rs`, `tools/**/*.py`): "before designing an algorithm, data layout or schedule change, run `lookup.py --brief <file>`; cite the row id in the commit; follow plan v2 §0–§1". Skill = the §0 protocol plus the five lookup commands. CLI: add subcommand `ref-lookup` next to the other `sub.add_parser(...)` calls (cli.py:472 onward) that forwards its args to `lookup.py` via `subprocess` and returns its stdout.
- **Don't**: copy matrix content into the rule or skill (it goes stale); they point at the files.
- **Accept**: `python -m rust_engine_mcp.cli ref-lookup --check` (run from `tools/mcp/python`) → `0 problems` · the rule file has `alwaysApply: false` and globs.

### T5 ⚙ Coverage and drift guard — V3 C2 R1 · Needs T1
- **Files**: `lookup.py`, new `tools/mcp/python/tests/test_ref_books_index.py`
- **Do**: `--gaps` = every `src/<dir>/` and every `.rs` file over 400 lines that no matrix `code` cell covers, sorted by line count (top 40). Test runs `--check` and asserts exit 0 (no PDFs needed; `--check` reads only `matrix.md` and `toc/`).
- **Accept**: `python -m pytest tools/mcp/python/tests/test_ref_books_index.py -q` passes · `--gaps` prints a list.
- **Follow-up (not in this slice)**: add matrix rows for the top gaps; each new row must pass `--check`.

### T6 🧠 Multi-seed scoring with variance — `LGT-08` · gea4v2:12.6.2.7 · V5 C3 R2
- **Why**: one playcheck run cannot tell a real improvement from seed noise. Every tuning slice below depends on this.
- **Files**: `tools/sim_review/playcheck.py`, `tools/sim_review/score_dump.py`, `tools/sim_review/README.md`
- **Recon**: `grep -n "add_argument" tools/sim_review/playcheck.py` → today there is one `--seed` (verified). Done-if: a `--seeds` flag already exists.
- **Do**: `--seeds N` (default 1 = current behaviour). Seeds = base seed, then `sha256(base, i)` truncated to u64, so the set is reproducible. Build once, run N times with `--no-build` semantics, keep each dump as `debug_runs/sim_review/dump_<seed>.json`. `score_dump.py --multi <dir>` writes `matrix_multi.json`: per metric `mean, stdev, min, max, n, stderr`. Dashboard shows mean ± stderr. Print the verdict rule: a change is real only if `|mean_after − mean_before| > 2 × max(stderr_before, stderr_after)`.
- **Don't**: change single-seed output files or the scoring rules in `expected/matrix_v1.json`. A run takes at least 360 s, so keep N small (3) by default in docs.
- **Accept**: `python tools/sim_review/score_dump.py --multi tools/sim_review/fixtures` works on fixture dumps (add a second fixture dump) · existing single-run command output is byte-identical.

### T7 🧠 Constant auto-tuner — `LEARN-01` · aig3:7.2.4 · V4 C4 R3 · Needs T6
- **Recon**: `grep -nE "Deserialize|ron|from_file" src/combat/balance.rs` → Done-if n/a. If `FrontBalance` is **not** loadable from a RON file, the first sub-slice is only that (RON load with the current constants as defaults; AGENTS.md serialization rule), then stop.
- **Do**: new `tools/sim_review/tune.py`: hill climbing with random restarts over a whitelist of named constants (start with ≤ 6 in `FrontBalance`), objective = mean matrix score from T6, step accepted only under the T6 verdict rule. Writes `debug_runs/sim_review/tune_proposal.ron` + a table of tried values.
- **Don't**: write into `assets/` or `src/`; a human applies the proposal. No annealing or genetic variants until hill climbing has produced one accepted change.
- **Accept**: `python tools/sim_review/tune.py --dry-run` prints the whitelist and the planned perturbations without launching the game.

---

## 5. Wave 1 — engine-wide

### E1 ⚙ Sole-writer static check — `CONC-03` · gea4v1:4.9.7.5 · V5 C2 R1
- **Why**: "one writer per domain" is a rule with no enforcement. The book's cheap assertion becomes a static check here (a runtime assertion would be test-shaped production code).
- **Files**: new `tools/validators/sole_writer_check.py`, new `tools/validators/knowledge/sole_writers.tsv`, `tools/validators/README.md`
- **Do**: TSV = `Resource<TAB>owner path prefix` taken from the single-writer table in `.cursor/skills/bevy-simulation-grade/07-repo-authority-map.md` (start with `MissionBook`, `LogisticsOrderSim`, `SimAttentionFrame`, `WeatherField`, `ViewManager`). Script greps `src/**/*.rs` for `ResMut<Name>` and `&mut Name` in system params outside the owner prefix, skipping `#[cfg(test)]` blocks and `tests/`. Exit 1 with `file:line` list.
- **Don't**: "fix" violations in this slice. Record the baseline count; each fix is its own slice routed to the owner.
- **Accept**: script runs in < 10 s and prints `N resources, M violations`; the M list is pasted under this card.

Baseline (2026-11-14): `5 resources, 4 violations`

```text
src/render/view_runtime/bridge.rs:217  ResMut/&mut ViewManager outside src/gui/view_authority.rs
src/render/view_runtime/bridge.rs:255  ResMut/&mut ViewManager outside src/gui/view_authority.rs
src/render/view_runtime/phase_d_parity_stress.rs:51  ResMut/&mut ViewManager outside src/gui/view_authority.rs
src/render/view_runtime/phase_d_parity_stress.rs:66  ResMut/&mut ViewManager outside src/gui/view_authority.rs
```

### E4 ⚙ Determinism lint (report-only) — `MEM-08` `LOOP-02` `MATH-02` · gea4v1:3.3.1.4, gea4v1:8.4 · V4 C2 R1
- **Files**: new `tools/validators/determinism_lint.py`, `tools/validators/README.md`
- **Do**: in the sim dirs from §1 flag: `thread_rng|rand::random`, `Instant::now|SystemTime`, `Res<Time>` / `Time<Real>` / `delta_secs` (sim must step on the sim clock, e.g. `SimTimeMicros`), and `for … in` / `.iter()` / `.values()` / `.keys()` over a binding declared as `HashMap`/`HashSet` in the same file. Output `rule file:line` sorted, plus counts per rule. Skip `#[cfg(test)]`.
- **Don't**: edit `src/`. Report only. `combat/relax.rs` uses `BTreeMap` and is the model to copy in later fixes.
- **Accept**: prints counts per rule; paste the top 15 under this card as the fix backlog.

Baseline (2026-11-14): `rng: 3` · `wall_clock: 2` · `real_time: 42` · `hash_iter: 41`

```text
hash_iter src/combat/formation.rs:146
hash_iter src/combat/fortification.rs:100
hash_iter src/combat/front.rs:193
hash_iter src/combat/front.rs:304
hash_iter src/combat/front.rs:346
hash_iter src/construction/procedural/types.rs:181
hash_iter src/construction/procedural/types.rs:204
hash_iter src/construction/procedural/types.rs:209
hash_iter src/construction/weighted_footprint.rs:47
hash_iter src/economy/logistics/district_scope.rs:31
hash_iter src/economy/resource_flow.rs:274
hash_iter src/strategic/construction_book.rs:164
hash_iter src/strategic/settlement/block_rollout.rs:351
hash_iter src/strategic/settlement/district.rs:274
hash_iter src/strategic/settlement/town.rs:223
```

### E6 ⚙ Streaming hysteresis band — `IO-02` · gea4v2:17.4.2 · V4 C2 R2
- **Why**: a focus sitting on a chunk edge loads and unloads the same chunks every few frames.
- **Files**: `src/io/streaming/residency.rs`, `src/io/streaming/budget.rs` (+ the one caller found by Recon)
- **Recon**: `grep -rnE "chunk_window_coords|unload|evict" src/io/streaming src/substrate | head -30`. `chunk_window_coords(focus, radius)` takes one radius (verified). Done-if: unload already uses a larger radius than load.
- **Do**: load when inside radius `R`; unload only when outside `R + H`, `H ≥ 1` chunk, constant next to the existing budget constants. Pure function + test.
- **Accept**: new test: focus oscillating across a chunk border 20 times causes zero load/unload events after the first step · `cargo test -p proc_A_dine01 --lib io::streaming`.

### E2 ⚙ Name the one-tick-stale reads — `LOOP-07` · gea4v2:17.6.3.3 · V3 C1 R1
- **Files**: `src/render/extraction/sim_visual_extract.rs`, `src/strategic/world_read_snapshot.rs`, `.cursor/skills/bevy-simulation-grade/07-repo-authority-map.md`
- **Do**: add a `//! Staleness:` line to each snapshot/extract system saying which tick it reads (this tick or previous) and why that is acceptable; add a "Staleness" table (reader · source · lag · reason) to the authority map.
- **Don't**: reorder any system. Documentation only.
- **Accept**: `cargo check -p proc_A_dine01` · table has a row for every snapshot type in the two files.

### E3 🧠 Phase-offset the heavy periodic systems — `EXEC-01` · aig3:10.1.1 · V4 C3 R3 · Needs E4
- **Why**: periodic systems that all fire on `tick % N == 0` stack up in one frame and cause spikes.
- **Recon**: `grep -rnE "% *[A-Za-z0-9_:]+ *(==|!=) *0" src/systems src/combat src/economy src/strategic src/ai | head -40`. List (system, period). Done-if: fewer than 3 systems share a coincident tick.
- **Do**: give each periodic system a constant `(period, phase)` with distinct phases; prefer periods that are pairwise co-prime where gameplay allows. Constants stay in the owning module. Add one test that walks one LCM window and asserts max coincident heavy systems ≤ 2.
- **Don't**: randomize phases; change periods of systems whose period is gameplay-visible; touch `AdvanceSimTick` ordering (gated by `plan_schedule_sync_v1.md`).
- **Accept**: the test · playcheck `PerfScope upd_*` max does not rise (trust `upd_*`, not `STALL` lines).

### E5 🧠 Per-layer update rates for field layers — `TAC-06` `LOOP-09` · aig3:6.2.5 · V3 C3 R2
- **Recon**: `grep -rn "WorldFieldLayerEpoch" src | head -20`. One global epoch exists in `strategic/world_field_layers.rs` (verified, 49 lines). Done-if: layers already have separate epochs.
- **Do**: classify each layer `Static | Slow | Dynamic`; one epoch per class; writers bump only their class; consumers compare the class epoch they depend on.
- **Don't**: add a second field store. Same resource, more epochs.
- **Accept**: test: bumping `Dynamic` leaves `Static` consumers' cached value untouched · `cargo test -p proc_A_dine01 --lib strategic::`.

### E7 🧠 Post-load rebuild audit — `IO-08` `IO-07` · gea4v1:7.2.2.10, gea4v1:7.2.2.9 · V4 C3 R3
- **Recon**: `grep -nE "pub struct|Serialize" src/io/save/dto.rs | head -50` and `grep -n "fn " src/io/save/apply.rs`. Write the list: field · truth or derived · rebuilt where.
- **Do**: for every **derived** field found in a DTO: stop saving it and rebuild in the apply/hydrate pass. For every `Entity` stored in a DTO: replace with the stable id and remap once in `apply.rs`.
- **Don't**: break existing saves without a version bump and a loader branch (⚠ needs user sign-off if a save format changes).
- **Accept**: round-trip test: save → load → K ticks equals no-save K ticks on the compared counters · `cargo test -p proc_A_dine01 --lib io::save`.
- **Close (2026-11-14)**: **IO-07 ADOPTED** (no `Entity` on wire; chunk coords + string keys). **IO-08 PARTIAL** — new saves omit derived stand `heat`/`fuel`/`burn_hold` and fire aggregates; transport `nodes` rebuilt on hydrate (`post_load_rebuild.rs`). Legacy stand RON may still carry inlined heat vectors (still deserializes); stripping `successors` would need a save-format version bump + user sign-off. `io::save` lib tests green (30).

---

## 6. Wave 2 — subsystems

### C1 ⚙ Swept ballistic impact — `PHYS-02` · gea4v2:14.3.5.7 · V5 C2 R1
- **Why**: `combat/ballistics.rs` (146 lines: `launch`, `step_ballistic_flights`) tests the end position only (verified: no swept test). A fast shell can jump over a cell.
- **Files**: `src/combat/ballistics.rs`
- **Do**: in `step_ballistic_flights`, walk every grid cell on the segment previous position → new position with an integer grid traversal (DDA / supercover) and resolve impact at the first hit cell. Fixed order, no floats in the cell walk beyond the initial setup.
- **Accept**: new test: a flight moving ≥ 3 cells per step hits a target placed on a cell strictly between two step positions · `cargo test -p proc_A_dine01 --lib combat::ballistics`.

### F2 🧠 Sleeping chunks wake on neighbour fire — `PHYS-06` · gea4v2:14.4.7.7 · V5 C3 R2
- **Recon**: `grep -nE "quiet|short|skip|sleep|idle|dormant" src/systems/fire/chunk_fire_overlay.rs src/systems/chunk_sim_lod.rs src/systems/fire/cell_spread.rs`. Find the early-out for quiet chunks and what re-arms it. Done-if: a test already shows fire crossing into a quiet chunk.
- **Do**: a chunk on the short path wakes when any of its 8 neighbours has a burning cell on the shared edge (or wind-ward within spread reach). Wake is decided by the fire domain's writer from the previous-tick snapshot.
- **Don't**: wake by scanning all chunks every tick; use the active-chunk set.
- **Accept**: test: burning cell at a chunk's east edge ignites the west edge of a sleeping neighbour within 2 ticks · `cargo test -p proc_A_dine01 --lib systems::fire`.

### L1 🧠 Open-goal source search — `NAV-09` · aig3:4.7.1 · V5 C3 R2
- **Recon**: `grep -rnE "nearest|best_source|pick_source|candidate|closest" src/economy/logistics/orders | head -25`. Done-if: source selection is already one search with a goal predicate.
- **Do**: if the code computes a path or distance per candidate depot: replace with one search outward from the destination that stops at the first node whose predicate "stock ≥ quantity and owner allowed" holds. Tie-break by stable node id.
- **Don't**: add a second pathfinder; reuse the flood/search already used by order-sim. `LogisticsOrderSim` stays the only writer.
- **Accept**: test comparing the new picker against the brute-force result on a fixture graph (same source, same cost) · `cargo test -p proc_A_dine01 --lib economy::logistics`.

### U1 ⚙ Ducking and instance limits — `AUD-02` · gea4v2:15.5.8.2 · V4 C2 R1
- **Recon**: `grep -nE "duck|limit|max_|cap|gain" src/audio/mix_frame.rs`. `AudioOneShotQueue` has `push_ui / push_warning / push_combat` (verified). Done-if: per-bus caps and a duck gain already exist.
- **Do**: in `build_audio_mix_frame`: cap concurrent combat one-shots (keep nearest to camera, then newest); while any warning plays, lower the ambience bus by about 6 dB with attack/release smoothing on real time.
- **Don't**: read or write sim state beyond the existing snapshot inputs.
- **Accept**: unit tests for cap selection and for duck gain reaching target then releasing · `cargo test -p proc_A_dine01 --lib audio::`.

### C2 🧠 Iterative firing solution under wind — `MOVE-09` · aig3:3.5.4 · V4 C3 R2 · Needs C1
- **Recon**: `grep -rn "missile_weather_drift" src | head`. Drift exists on the weather side; `ballistics.rs` has no drag or aim iteration (verified).
- **Do**: aim refinement: simulate with the current aim, measure miss vector, correct aim, repeat a **fixed** 6 iterations; stop early when miss < half a cell. Uses the same step function as live flight so prediction and flight agree.
- **Accept**: test: under constant crosswind the corrected launch lands within one cell where the uncorrected one misses · same test filter as C1.

### F1 🧠 Separable kernel for heat diffusion — `TAC-03` · aig3:6.2.7 · V3 C3 R3
- **Recon**: read the stencil in `src/compute/heat_diffusion.rs` (ping-pong buffers, verified) and find the WGSL twin: `grep -rn "heat_diffusion" src assets --include=*.wgsl -l`.
- **Do**: only if the stencil is a symmetric box/gaussian: two 1-D passes over the dense buffers. CPU and GPU kernels must stay identical: change both or neither.
- **Don't**: apply to `systems/fire/cell_spread.rs` (wind makes it anisotropic, not separable) or `combat/relax.rs` (sparse `BTreeMap`, 4-neighbour).
- **Accept**: test: new output equals old output within 1e-5 on a fixture grid · `cargo test -p proc_A_dine01 --lib compute::`.
- **Recon findings (2026-11-14, PARTIAL)**: `grep` under `src/` + `assets/**/*.wgsl` for `heat_diffusion` → **no WGSL compute kernel** (only `prepare_heat_diffusion_gpu_buffer` uploads CPU rows). Stencil in `run_heat_diffusion_step` (`heat_diffusion.rs:155–250`) is a **sparse** `HashMap` von-Neumann step: mean of **present** 4-neighbours only (variable divisor), plus per-step `DECAY_RATE`, `DIFFUSION_RATE`, and fire `SOURCE_BLEND` — not a symmetric box/Gaussian convolution on dense buffers; two separable 1-D passes would not match this operator within 1e-5 without a planned dense-grid redesign (and there is no GPU twin to pair with).

### A1 🧠 Score goals per unit time — `DEC-08` · aig3:5.7.4 · V4 C2 R3
- **Files**: `src/ai/faction/decide.rs`
- **Do**: add one named `ScoreTerm` ("duration") that discounts a goal by its expected ticks to complete, so a quick small win can beat a slow big one. Keep plain weights and deterministic tie-breaks (file header: `MAI-CUT-UTILITY`, no utility crate, no curves).
- **Accept**: test: two goals of equal value, different duration → shorter wins; the term appears in the explain output · `cargo test -p proc_A_dine01 --lib ai::faction` · multi-seed compare (T6) in the commit body.

### A2 ⚙ Explain panel shows behaviour changes — `DEC-15` · aig3:2.1.4 · V4 C2 R1 · owner `@designer` + `@coder`
- **Files**: `src/strategic/ai_explainability.rs`, `src/gui/ai_explainability_ui.rs`, `src/ai/faction/log.rs`
- **Do**: record an entry only when a faction's chosen goal changes: `tick · old → new · deciding term`. Panel (`Insert`) lists the last 8. UI reads the snapshot only.
- **Accept**: test that an unchanged goal over 10 ticks adds no entries and one switch adds exactly one.

### L3 🧠 Replan only what a closure touches — `NAV-10` · aig3:4.7.2 · V4 C3 R3
- **Recon**: `grep -n "fn " src/economy/logistics/orders/vehicle_fire.rs src/systems/navigation/logistics_floodfill.rs`. Find what is recomputed when a road cell burns. Done-if: invalidation is already per-route.
- **Do**: keep a dirty-cell set per tick; invalidate only orders whose cached route intersects it; others keep their route.
- **Accept**: test: closing one edge re-routes the affected order and leaves an unrelated order's route identical.

### W1 🧠 Weather reads through cached stations — `SENSE-02` · aig3:11.3 · V3 C3 R2
- **Recon**: `grep -rln "WeatherField" src --include=*.rs | grep -v "systems/weather"`. For each reader note whether it samples per entity per tick.
- **Do**: per-entity samplers switch to a per-chunk (or per-station) value cached once per tick by the weather domain, the same shape as `AirOpsWeatherStatus`.
- **Don't**: let a reader build its own cache (that is a second authority).
- **Accept**: reader list pasted under the card with before/after sample counts · `cargo test -p proc_A_dine01 --lib systems::weather`.

### L6 🧠 Anytime throughput solver — `EXEC-04` · aig3:10.2 · V3 C3 R3
- **Recon**: `grep -nE "fn |budget|max_iter" src/economy/logistics/solver.rs`. Done-if: the solve already has an iteration budget and keeps the last valid answer.
- **Do**: bounded work per tick scaled by `AttentionTier::work_scale`; always publish the last solution that satisfies `reservations_within_capacity` and `book_matches_soa_reserved`.
- **Accept**: existing solver invariants hold after every partial step (test loops them).

### Smaller cards (same protocol; Files = the matrix row's code cell, see `lookup.py --row <ID>`)

| Id | Row · ref | Mark | Do (one line) | Accept |
|:---|:---|:---|:---|:---|
| **F3** | `LGT-09` · gea4v2:12.2.4.5 | 🧠 | Radiant exposure falls with distance squared, clamped near the source; expose flux in kW/m². Recon the current falloff in `systems/fire/entity_exposure.rs`. | test: exposure at 2d is ¼ of exposure at d outside the clamp; T6 compare |
| **L5** | `NAV-14` · aig3:6.3.2 | 🧠 | Per-order-type concern weights (time vs fire risk) applied to the existing flood cost; weights in RON. | test: high-risk-averse order takes the longer safe route |
| **L2** | `DEC-10` · aig3:5.7.7 | 🧠 | Demand adverts gain urgency with unmet time and a distance discount. Recon scoring in `orders/plant_demand.rs` first; likely PARTIAL. | test: older unmet demand outranks a newer equal one |
| **G2** | `MATH-03` · aig3:8.1.2 | ⚙ | Halton-sequence jitter (integer-only, seeded per chunk) for vegetation/prop placement in `render/world_map_vegetation_mesh.rs`. Render-only. | visual capture + test that points are identical for the same chunk seed |
| **F4** | `ANIM-01` · gea4v2:13.4.1.5 | ⚙ | Flame loop phase = `fract(global_time / period + hash(cell))` so LOD swaps do not pop. Render-only, `render/fire_vfx`. | `VFX_CAPTURE=1` before/after frames |
| **W3** | `AUD-05` · gea4v2:15.4.3.4 | ⚙ | Rain/fog intensity from `systems/weather/camera_local.rs` drives a low-pass + attenuation parameter in `audio/mix_frame.rs`. | test: heavier rain → lower cutoff |
| **U2** | `AUD-03` · gea4v2:15.5.7.2 | ⚙ | One cue id → N variants in `audio/sfx_catalog.rs`; pick with an audio-local RNG, never the sim RNG; no immediate repeat. | test: no variant twice in a row |
| **U3** | `AUD-08` · gea4v2:15.6.2.6 | ⚙ | Alert barks carry priority; a higher one interrupts, equal ones do not stack (`audio/ui_hooks.rs`). | test of the priority table |
| **R2** | `RND-20` · gea4v2:11.2.3.13 | ⚙ | Heatmap ramps monotonic in lightness; overlay blending in linear space (`gui/style`, `render/pipelines/domain_overlay_gpu.rs`). Pairs with S2. | test: ramp lightness strictly increasing |
| **K2** | `INPUT-02` · gea4v1:9.5.4.3 | ⚙ | Named click-vs-drag distance and time thresholds in `construction/build_interaction.rs`; no build on a sub-threshold drag. | test of threshold classification |
| **R3** | `LGT-02` · gea4v2:12.5.12.4 | 🧠 | Recon `render/lighting/fire_light_cluster.rs`; if lights are a flat list, bin by screen tile with a per-tile cap. Likely PARTIAL. | light count per tile ≤ cap in capture stats |

---

## 7. Wave S — studio tool (Art Pipeline Suite) and art tooling

Scope: `tools/mcp/art_pipeline_suite/` (Tk app, `app.py`) and `tools/mcp/python/rust_engine_mcp/`. Owner `@coder-mcp`; specs from `@designer-mcp`.
Extra rejects: the `mcp-production-rules` skill applies (deterministic seeded output, no AI-generated final art, batch/atlas, grid alignment);
the studio never reads live game state, only promoted assets and dumps (tool vs runtime split, `ECS-15` · gea4v2:16.2.2).
Tests: `python -m pytest tools/mcp/python/tests -q` must stay green for every card.

### S1 ⚙ Atlas bleed and texel-density QC — `RND-08` · gea4v2:11.3.14.6 · V4 C2 R1
- **Files**: `tools/mcp/python/rust_engine_mcp/aps_atlas_qc.py`, its test
- **Recon**: `grep -nE "gutter|padding|bleed|mip|density" tools/mcp/python/rust_engine_mcp/aps_atlas_qc.py`. Done-if: both checks exist.
- **Do**: check 1: gutter around each tile ≥ `2^(mip levels used)` px, edge pixels extruded into the gutter. Check 2: texels per metre constant across the tiles of one zoom band (tolerance 5 %). Report through the existing QC result structure and plain-language validator text.
- **Accept**: test with a synthetic atlas that fails each check and one that passes.

### S3 ⚙ Generate-and-test report — `PCG-08` · aig3:8.4.4 · V4 C2 R1
- **Files**: `grammar_iterate.py`, `building_quality_qc.py`, `golden_seed_review.py` (all under `rust_engine_mcp/`)
- **Do**: for N seeded candidates produce: pass rate, top-k kept, and a histogram of **rejection reasons**. Surface the histogram in the APS assembly QC strip (`assembly_qc_strip.py`).
- **Accept**: CLI output on the golden seed set is identical across two runs (determinism) and lists reasons with counts.

### S4 🧠 Grammar rule coverage — `PCG-06` · aig3:8.5.2 · V4 C3 R2 · Needs S3
- **Files**: `building_grammar.py`, `grammar_layers.py`, `kit_coverage_audit.py`
- **Do**: across the golden seeds report (a) rules that never fire, (b) terminals with no kit module, (c) rules that always fail the gate. Output a table the designer can act on.
- **Don't**: auto-delete rules. Report only.
- **Accept**: report file written under staging; test on a tiny grammar with one dead rule.

### S6 ⚙ Sound-cue validator — `AUD-03` · gea4v2:15.5.7.2 · V3 C2 R1
- **Recon**: `grep -n "pub struct\|pub enum" src/audio/sfx_catalog.rs` to learn the cue format.
- **Do**: CLI command validating cue groups: ≥ 2 variants for repeated one-shots, peak level spread within 1.5 dB, no duplicate files, all paths exist. Stdlib `wave` module only.
- **Accept**: test with fixture WAVs; command listed by `python -m rust_engine_mcp.cli commands`.

### S2 ⚙ Perceptual ramp generator — `RND-20` · gea4v2:11.2.3.13 · V3 C2 R1
- **Do**: deterministic generator for overlay/heatmap ramps (lightness-monotonic, computed in linear space, exported as a small LUT + RON entry); QC rejects non-monotonic ramps. Game-side consumer is R2.
- **Accept**: same input → byte-identical LUT; QC test for a non-monotonic ramp.

### S5 🧠 Scatter presets with engine parity — `MATH-03` · aig3:8.1.2 · V3 C3 R2 · Needs G2
- **Do**: landscape presets (`landscape_grammar_presets.py`) preview scatter with the **same Halton function as the engine**; share one test vector file (first 16 points for seed 1) asserted by both the Python test and the Rust test.
- **Don't**: implement Poisson-disk on both sides (two implementations drift). If Poisson is wanted, bake points studio-side and ship them as data.
- **Accept**: both tests read the same vector file and pass.

---

## 8. Wave 3 — future features (planner first, no code from this file)

Each item needs a short @planner plan (authority, SystemSet, save impact) before any slice is cut.
Start it with `lookup.py --row <ID>` and one `--read`.

| Feature | Rows · ref | Why it is not a slice yet | Trigger to start |
|:---|:---|:---|:---|
| Formation members, cost-based slot assignment, bounding overwatch | `MOVE-05` `MOVE-06` `MOVE-07` · aig3:3.7.4, aig3:3.7.8, aig3:3.7.10 | `combat/formation.rs` is one entity per formation; `faction_slot` is **not** a formation slot. Needs a unit-member model. | Front combat wants visible unit positions |
| Thermal then hydraulic erosion ⚠ | `PCG-03` · aig3:8.3.5 | Changes every generated world and golden seeds | User sign-off on new world look |
| Chokepoint / defensibility precompute | `TAC-05` · aig3:6.2.3 | Needs consumers (fortification, faction AI) agreed first | A1 shipped and AI needs terrain reasons |
| Weather-degraded sensing over cells | `SENSE-04` · aig3:11.4.5 | Needs W1 station cache | W1 ADOPTED |
| Time-varying route costs (forecast windows) | `NAV-13` · aig3:4.8 | Needs L1 and L3 | L1 + L3 ADOPTED |
| Player-pattern N-grams | `LEARN-02` · aig3:7.3.4 | Decide: sim truth (saved) or derived (rebuilt) | A2 shipped, AI feels predictable |
| Multi-tier AI coordination | `TAC-10` · aig3:6.4.1 | Crosses faction, formation, missions authorities | After formation members |
| Settlement road skeleton and blocks | `PCG-07` · aig3:8.4.2 | Owned by `plan_town_growth_v1.md` | That plan's next phase |
| Virtual-texture residency for stage 6 | `RND-07` · gea4v2:11.3.14.8 | Large render change | Atlas memory becomes the bottleneck |
| LOD seam fix | `RND-06` · gea4v2:11.6.8.4 | Only if seams are visible | A capture shows cracks |
| MCTS for faction decisions | `TAC-14` · aig3:9.5 | **DEFER**: plain weights are explainable and cheap | Plain scoring fails a T6-measured goal twice |

## 9. Ledger (seed for `adoption.tsv`, T2)

Update the row when a slice closes. Evidence = `file:line`, test name, or commit.

| Row | Slice | Status | Evidence / note |
|:---|:---|:---|:---|
| ANIM-02 | — | ADOPTED | `construction/site_stage_transitions.rs` pure transition table (CON-P2-002) |
| TAC-04 | — | ADOPTED | `systems/fire/cell_spread.rs` 8-neighbour CA in snapshot order |
| MATH-02 | — | PARTIAL | `ai/faction/rng.rs` mix64; determinism_lint backlog remains (E4 report-only) |
| LOOP-09 | E5 | ADOPTED | `WorldFieldLayerEpoch` static/slow/dynamic epochs |
| INPUT-05 | — | ADOPTED | `gui/tactical/map_camera.rs` ApplyInput → DeriveDesired → Smooth |
| ECS-07 | — | ADOPTED | v1 Order 3: `MissionIntentQueue` → `MissionBook` |
| EXEC-05 | — | PARTIAL | v1 Order 2: `SimAttentionFrame` scaffold (not a Wave 1/2 gap) |
| LOOP-08 | — | ADOPTED | `WeatherField` prev/next |
| CONC-03 | E1 | ADOPTED | `sole_writer_check.py` baseline 5 resources, 4 violations |
| MEM-08 | E4 | ADOPTED | `determinism_lint.py` report-only |
| LOOP-02 | E4 | ADOPTED | `determinism_lint` real_time rule |
| IO-02 | E6 | ADOPTED | unload hysteresis + oscillation test |
| LOOP-07 | E2 | ADOPTED | Staleness docs on extract + world_read_snapshot |
| EXEC-01 | E3 | ADOPTED | `heavy_periodic` phase offsets + LCM test |
| TAC-06 | E5 | ADOPTED | `FieldLayerUpdateClass` epochs |
| IO-08 | E7 | PARTIAL | omit/rebuild derived on hydrate; legacy stand heat may still deserialize |
| IO-07 | E7 | ADOPTED | chunk coords + string keys; no `Entity` in save DTOs |
| PHYS-02 | C1 | ADOPTED | swept DDA + `swept_impact` test |
| MOVE-09 | C2 | ADOPTED | `refine_missile_aim` 6-iter + wind tests |
| PHYS-06 | F2 | ADOPTED | `sleeping_neighbor_west_edge_ignites_within_two_ticks` |
| NAV-09 | L1 | ADOPTED | `best_source_open_goal` + nav09 fixture |
| AUD-02 | U1 | ADOPTED | combat oneshot cap + ambience duck |
| TAC-03 | F1 | PARTIAL | sparse Jacobi + no WGSL twin — not separable (see F1 card) |
| DEC-08 | A1 | ADOPTED | duration `ScoreTerm` in `decide.rs` |
| DEC-15 | A2 | ADOPTED | behaviour-change log + snapshot panel |
| NAV-10 | L3 | ADOPTED | `LogisticsRouteDirtyTiles` selective invalidation |
| SENSE-02 | W1 | ADOPTED | `WeatherPollingStations` + migrated readers |
| EXEC-04 | L6 | ADOPTED | anytime throughput solve + invariant test |
| LGT-09 | F3 | ADOPTED | radiant `flux_kw_m2` inverse-square |
| NAV-14 | L5 | ADOPTED | `logistics_path_concerns_v1.ron` + risk_averse test |
| DEC-10 | L2 | ADOPTED | demand advert unmet_ticks + distance discount |
| MATH-03 | G2 | ADOPTED | Halton `scatter_jitter_in_cell` (S5 still OPEN) |
| ANIM-01 | F4 | ADOPTED | `flame_loop_phase` in `particle_anim.rs` |
| AUD-05 | W3 | ADOPTED | weather low-pass + attenuation in mix |
| AUD-03 | U2 | ADOPTED | cue variants + `AudioLocalRng` (S6 still OPEN) |
| AUD-08 | U3 | ADOPTED | alert bark priority exclusivity |
| RND-20 | R2 | ADOPTED | heatmap ramp linear lightness (S2 still OPEN) |
| INPUT-02 | K2 | ADOPTED | `BUILD_POINTER_DRAG` thresholds |
| LGT-02 | R3 | ADOPTED | screen-tile fire light cap in extract |
| LGT-08 | T6 | ADOPTED | playcheck `--seeds` + `score_dump --multi` |
| LEARN-01 | T7 | ADOPTED | `tune.py --dry-run` whitelist |
| RND-08 | S1 | OPEN | Wave S |
| PCG-08 | S3 | OPEN | Wave S |
| PCG-06 | S4 | OPEN | Wave S |
| MOVE-05 | §8 | DEFER | needs unit-member model |
| MOVE-06 | §8 | DEFER | needs unit-member model |
| MOVE-07 | §8 | DEFER | needs unit-member model |
| PCG-03 | §8 | DEFER | ⚠ world-changing |
| TAC-05 | §8 | DEFER | |
| SENSE-04 | §8 | DEFER | |
| NAV-13 | §8 | DEFER | |
| LEARN-02 | §8 | DEFER | |
| TAC-10 | §8 | DEFER | |
| PCG-07 | §8 | DEFER | |
| RND-07 | §8 | DEFER | |
| RND-06 | §8 | DEFER | |
| TAC-14 | §8 | DEFER | |

## 10. Plan exit

**Waves 0–2:** closed 2026-11-14 (Accept spot-checks green; F1 + E7/IO-08 locked **PARTIAL** with written reasons).  
**Remaining for full plan close:** Wave S (`RND-08`/`PCG-08`/`PCG-06` and S2/S5/S6 companions) → `@coder-mcp`.  
Then: every Wave S row `ADOPTED`/`PARTIAL`/`REJECTED` and `python -I src/dev/ref_books_index/lookup.py --check` → `0 problems`. Wave 3 items stay in their own plans.
