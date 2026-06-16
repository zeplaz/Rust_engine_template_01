# Micro tools registry `v1`

**Rule:** Deterministic steps = **micro CLI** or **MCP tool** that calls the same code. The LLM picks *which* tool, not *how to edit a glb byte-by-byte*.

## Tier 1 — shipped (repo)

| CLI command | MCP tool | Does |
|:---|:---|:---|
| `ping` | `ping` | Repo + blender path |
| `locate-blender` | `locate_blender` | Resolve Blender exe |
| `validate-spec <path>` | `spec_validate` | JSON schema AssetSpec |
| `write-spec <path>` | `spec_write` | Write staging spec |
| `run-geometry <job.json>` | `geometry_run_job` | Blender headless bpy |
| `job-status <id>` | `geometry_job_status` | Read status file |
| `validate-glb <path>` | `validate_glb_asset` | Header + vertex budget |
| `list-staging` | `list_staging` | Staging folders |
| `promote <job_id>` | `promote_staging_module` | Copy to modules/ |
| `library-register <job_id>` | `library_register` | Upsert `_module_index.ron` + `_module_index.json` |
| `library-register --rebuild-all` | `library_register(rebuild_all)` | Rebuild index from all promoted folders |
| `library-search [--batch-id X]` | `library_search` | Filter index by tags / archetype / style_pack |
| `write-witness <batch_id>` | `write_witness` | Rebuild `debug_runs/art_pipeline/<batch>_live.json` |
| `promote <job_id> [--no-register]` | `promote_staging_module` | Promote; auto-registers unless `--no-register` |
| — | `geometry_operations` | List bpy operation ids |
| — | `micro_tool_help` | CLI help JSON |

## Tier 1a — agent queue + token briefs (orchestration)

| CLI | MCP tool | Does |
|:---|:---|:---|
| `agent-queue-next <agent>` | `agent_queue_next` | Next drainable slice; drain fallback when stop-point blocked |
| `agent-queue-update <id> <status>` | `agent_queue_update` | Checkpoint slice (`done` / `blocked` / `in_progress`) |
| `agent-queue-board` | `agent_queue_board` | Tab-separated board (no full JSON dump) |
| `witness-brief <path>` | `witness_brief` | Witness JSON: green + capped errors only |
| `handoff-brief` | `handoff_brief` | HANDOFF Goal/Blockers/Next sections only |
| `file-digest <path>` | `file_digest` | Head N lines + total line count |
| `orchestrator-brief` | `orchestrator_brief` | `last_run.json` summary |
| `token-savings-guide` | `token_savings_guide` | Policy: which tools replace raw logs |
| `agent-lang-demo` | `agent_lang_demo` | Tk UI + `--headless` — multi-agent BLANG workflow proof |

Queues: `grammar` → `grammar_continuation_queue.json` · `continuation` → `continuation_queue.json`  
Doc: [`docs/archive/2026-06-src-dev/plans/plan_agent_queue_mcp_v1.md`](../../docs/archive/2026-06-src-dev/plans/plan_agent_queue_mcp_v1.md)

## Tier 1a-BLANG — session loop (AGENT-LANG)

**Spec:** [`src/dev/agent_lang_v1.md`](../../src/dev/agent_lang_v1.md) · **Commit flow:** [`src/dev/a2c_commit_flow_v1.md`](../../src/dev/a2c_commit_flow_v1.md)

| Token | MCP / CLI | When |
|:---|:---|:---|
| `BLANG:PRE` | `pipeline_preflight()` | Session start |
| `BLANG:Q+` | `agent_queue_next(agent)` | Pick slice |
| `BLANG:Q✓` | `agent_queue_update(id, status, note)` | After COMMIT:WIT |
| `BLANG:HO` | `handoff_brief()` | Orient — not full HANDOFF Read |
| `BLANG:OPS` | `ops_get_project_brief()` | Compressed ops_project_brief_v1 — not HANDOFF + 80 witnesses |
| `BLANG:WIT` | `witness_brief(path)` | After work — not full witness Read |
| `BLANG:DIGEST` | `snapshot_digest(path)` | Touch snapshot |
| `BLANG:DIFF` | `snapshot_diff_brief(before, after)` | Grammar iterate |
| `BLANG:P0` | `validate_p0_gate_plain(path)` | P0 gate artist text |
| `BLANG:DOC` | `agent_doc_touch(path, intent)` | Doc orient/ref — ledger |
| `BLANG:STATS` | `agent_doc_reads_brief(min_reads=2)` | Hot re-read rollup |
| `BLANG:BOOT` | `agent_session_bootstrap(agent)` | Brief stack + FIELD◈ |
| `BLANG:CACHE` | `agent_doc_digest_cached(path)` | Cached digest — skip re-touch |
| `BLANG:PROMOTE` | `agent_doc_promote_hot_reads()` | Repeated paths → MCP cache |
| `BLANG:RUN` | `agent_run_append(event)` | End-of-slice telemetry |
| `BLANG:CARGO` | `validate_cargo_report(compress=4)` | @coder regression |
| `BLANG:BEVY` | `validate_bevy_report(compress=4)` | @coder bevy |
| `BLANG:MARK` | `agent_marker_append(...)` | extend row — not rewrite |
| `BLANG:ORCH` | `cargo orchestrate` | post-edit hook only |
| `BLANG:PY` | `pytest tools/mcp/python/tests/` | @coder-mcp |
| `BLANG:S5` | `cargo test -p proc_A_dine01 --lib stage5` | Stage5 spine |

**Session:** `BLANG:PRE → BLANG:OPS → BLANG:HO → L4 work → L5 tools → L6 WIT → BLANG:Q✓`  
**Policy card:** `token_savings_guide()` → key `blang` (full token map + `by_agent`)

## Tier 1e — Sim product validators (**SHIPPED**)

**Plan:** `$ref:docs/archive/2026-06-src-dev/plans/plan_mcp_sim_product_validators_v1.md` · ⟨MCP-P2-SIM-VALIDATORS-PLAN-001⟩ **SIGNED**

| BLANG | MCP tool | CLI | Status |
|:---|:---|:---|:---|
| `BLANG:REVIEW` | `review_order_brief()` | `review-order-brief` | **SHIPPED** |
| `BLANG:SLICE` | `slice_exec_brief(id)` | `slice-exec-brief` | **SHIPPED** |
| `BLANG:PLACE` | `validate_construction_report` | `validate-report construction` | **SHIPPED** |
| `BLANG:WIT` | `witness_brief(path, profile=...)` | `witness-brief --profile` | **SHIPPED** (profiles) |

Queue: `phase4` → `$ref:tools/orchestrator/queues/post_drain_phase4_queue.json`

## Tier 1-OPS — operations intelligence (JSON backend)

| CLI | MCP tool | Does |
|:---|:---|:---|
| `ops-get-project-brief` | `ops_get_project_brief` | `ops_project_brief_v1` — quality/utility, active picks, delta_wf focus |
| `ops-get-retry-guidance <task_id>` | `ops_get_retry_guidance` | Phase3 queue row status + witness path (stub v1) |

Witness: `debug_runs/agent_ops/ops_mcp_function_layer_live.json` · brief: `debug_runs/agent_ops/ops_project_brief_v1.json`  
Doc: [`src/dev/ops_mcp_function_layer_v1.md`](../../src/dev/ops_mcp_function_layer_v1.md)

## Tier 1c — MCP productivity P0 (token chain)

| CLI | MCP tool | Does |
|:---|:---|:---|
| `pipeline-preflight [--queue grammar]` | `pipeline_preflight` | Blender, schemas, repo paths, queue stale — replaces 5–8 ad-hoc checks |
| `snapshot-digest <path>` | `snapshot_digest` | Compact assembly summary — placements, materials, grammar, hint |
| `validate-p0-gate-plain <path>` | `validate_p0_gate_plain` | P0 gate + artist sentences from `aps_validator_plain_language_v1.md` |
| `mcp-productivity-p0-witness` | — | Refresh `debug_runs/mcp_productivity_p0_live.json` |

Doc: [`docs/archive/2026-06-src-dev/plans/plan_mcp_productivity_chain_v1.md`](../../docs/archive/2026-06-src-dev/plans/plan_mcp_productivity_chain_v1.md)

## Tier 1d — MCP productivity P1 (spine + atlas brief)

| CLI | MCP tool | Does | Status |
|:---|:---|:---|:---:|
| `tile-spine-run <req.json>` | `tile_spine_run_tool` | WRK→ATL chain — p0_gate → digest → preview → build → batch → pack → validate | **SHIPPED** |
| `tile-spine-run-witness` | — | Refresh `debug_runs/tile_spine_run_001_live.json` | **SHIPPED** |
| `atlas-meta-brief <folder>` | `atlas_meta_brief_tool` | ≤40-line UV grid + missing lookups + plain FAIL sentences | **SHIPPED** |
| `atlas-meta-brief-witness` | — | Refresh `debug_runs/mcp_atlas_brief_001_live.json` (v1 fail + v2 pass) | **SHIPPED** |
| `rt-registry [--batch-id]` | `rt_registry_tool` | RT-REG-001 rowhouse production register + lookup stamp | **SHIPPED** |
| `rt-registry-witness` | — | Refresh `debug_runs/rt_registry_001_live.json` | **SHIPPED** |
| `runtime-lookup-brief [--atlas-id]` | `runtime_lookup_brief_tool` | RT-BRIEF-001 index row + missing cells plain brief | **SHIPPED** |
| `runtime-lookup-brief-witness` | — | Refresh `debug_runs/rt_lookup_brief_001_live.json` | **SHIPPED** |

| `build-read-grammar-v0-002-witness` | — | Refresh OPS-006 + consumer contract witnesses | **SHIPPED** |
| `aps-dna-consumer-witness` | `arch_dna_consumer_contract` | @coder snapshot DNA+β consumer contract | **SHIPPED** |
| `arch-dna-snapshot-brief <snapshot.json>` | `arch_dna_snapshot_brief` | Compressed ARCH-DNA + β rollup | **SHIPPED** |
| `ops-007-warehouse-pause-witness` | — | Refresh `debug_runs/ops_007_warehouse_production_pause_live.json` | **SHIPPED** |
| `pilot-hardcode-lint` | — | Scan src/tests/examples for pilot id literals outside allowlist | **SHIPPED** |
| `pilot-hardcode-lint-witness` | — | Refresh `debug_runs/pilot_hardcode_lint_live.json` (MCP-GUARD-001) | **SHIPPED** |
| `mcp-p2-run-event-001-witness` | — | Refresh `debug_runs/mcp_p2_run_event_001_live.json` | **SHIPPED** |
| `mcp-p2-honest-bake-001-witness` | — | Refresh `debug_runs/mcp_p2_honest_bake_001_live.json` | **SHIPPED** |
| `validate-report arch_build_grammar <preset.json>` | `validate_report` | ARCH-DNA preset schema | **SHIPPED** |
| `validate-report landscape_grammar <preset.json>` | `validate_report` | LAND-DNA + topology graph v0 | **SHIPPED** (LG-0-001) |
| `validate-report tile_promotion_honest <batch.json>` | `validate_report` | Reject ortho/dry-run ship bakes | **SHIPPED** |
| `rail-warehouse-pilot-batch-write` | — | Materialize tile_batch + variant_set + bdef from staging spec | **SHIPPED** |
| `rail-warehouse-pilot-batch-witness` | — | Refresh `debug_runs/tile_rail_warehouse_pilot_batch_live.json` | **SHIPPED** |

Doc: [`docs/archive/2026-06-src-dev/plans/mcp_productivity_p1_plan_v1.md`](../../docs/archive/2026-06-src-dev/plans/mcp_productivity_p1_plan_v1.md)

**Grammar → spine handoff:** `grammar_iterate` + `snapshot_diff_brief` → `tile_spine_run_request_v1.snapshot_path`

## Tier 1g — grammar / building-set guards (**SHIPPED**)

**Plan:** [`src/dev/plan_mcp_grammar_build_set_guards_v1.md`](../../src/dev/plan_mcp_grammar_build_set_guards_v1.md) · todos: [`src/dev/mcp_grammar_build_set_todos_v1.md`](../../src/dev/mcp_grammar_build_set_todos_v1.md)

| CLI / MCP | Does | Status |
|:---|:---|:---|
| `pilot-hardcode-lint` / `validate-report pilot_hardcode_lint` | Block literal pilot ids outside catalog allowlist | **SHIPPED** (MCP-GUARD-001) |
| `example-teachable-audit-witness` | Refresh `debug_runs/example_teachable_audit_live.json` | **SHIPPED** (MCP-GUARD-002) |
| `single-archetype-ratio-guard` / `validate-report single_archetype_ratio_guard` | Fail if one grammar archetype >40% refs without set insurance | **SHIPPED** (MCP-GUARD-003) |
| `warehouse-track-guard` / `validate-report warehouse_track_guard` | Warehouse JSON paths need manifest or teaches | **SHIPPED** (MCP-GUARD-004) |
| `build-set-guards-witness` | Refresh all GUARD-002…004 witnesses | **SHIPPED** |
| `grammar_set_brief` | Pilot + grammar + preset inventory; gap lines | **SHIPPED** |
| `grammar_preset_pair_validate` | preset ↔ grammar_id ↔ pilot catalog row | **SHIPPED** |
| `grammar_eval_sweep` | Seed sweep massing/roof histogram | **SHIPPED** |
| `grammar_pilot_parity` | ValidationReport from catalog parity self-check | **SHIPPED** |
| `grammar-integration-validate` / `validate-report grammar_integration` | Snapshot DNA + grammar + site + materials gate | **SHIPPED** (MCP-INTEGRATE-001) |
| `complex_building_brief` | ≤40-line complex building digest | planned |
| `building_set_manifest_validate` | Multi-pilot set manifest schema | **SHIPPED** |
| `building_set_coverage_report` | F/L/I axis coverage; FAIL on singleton set | **SHIPPED** |
| `example_teachable_audit` | `_meta.teaches[]` on schema examples | **SHIPPED** |
| `single_archetype_ratio_guard` | Fail if one archetype >40% grammar refs | **SHIPPED** |
| `building_set_health_brief` | OPS rollup block | **SHIPPED** |

## Tier 1b — validators (structured reports for agents)

| CLI command | MCP tool | Does |
|:---|:---|:---|
| `validate-report cargo [--cached] [--compress 3]` | `validate_cargo_report` | cargo check JSON → ValidationReport |
| `validate-report bevy [-p pkg]` | `validate_bevy_report` | Bevy API classifier |
| `validate-report mcp_spec <path>` | `validate_report` | AssetSpec schema |
| `validate-report mcp_job <path>` | `validate_report` | GeometryJob schema + seed |
| `validate-report asset_glb <path>` | `validate_asset_report` | GLB structured issues |

Schema: `tools/validators/schemas/validation_report_v1.schema.json` · Rule: `.cursor/rules/validation-first.mdc`

## Tier 2 — Blender operations (headless)

| Operation | Params | Output |
|:---|:---|:---|
| `module_wall` | width_m, height_m, depth_m | box mesh, bottom-center pivot |
| `module_roof` | width_m, depth_m, thickness_m, profile?, pitch_height_m?, sawtooth_bays?, seed | flat \| pitched \| shed \| sawtooth |
| `module_door` | width_m, height_m, depth_m | frame box |
| `module_window` | width_m, height_m, depth_m, profile?, mullion_width_m?, seed | flat \| mullion \| arched \| curtain |
| `module_prop` | width_m, height_m, depth_m, prop_kind?, seed | box / l_corner / vent / ac greybox |

Add new ops: `tools/mcp/blender/scripts/ops/` + register in `run_job.py`.

## Tier 2c — tile + procedural batch (tri-mode: manual / CLI / MCP)

**Rule:** Same `rust_engine_mcp.tile_pipeline` module for all three surfaces.

| Capability | Manual (module viewer) | CLI | MCP tool | Status |
|:---|:---|:---|:---|:---:|
| Pack PNG folder → atlas | Pipeline → **Pack tile atlas** | `tile-atlas-pack <folder> [-pk]` | `tile_atlas_pack_tool` | **SHIPPED** |
| lod0 batch (003–010) | Pipeline → **Run batch** | `lod0-batch-run --batch X --phase Y` | `lod0_batch_run_tool` | **SHIPPED** |
| Validate `tile_batch_v1` JSON | — | `validate-report tile_batch <path>` | `tile_batch_validate` | **SHIPPED** |
| Pack keyframe PNGs (ship) | APS → **Pack atlas** | `tile-batch-run` with `bake_source: keyframe_pack` | `tile_batch_run` | **SHIPPED** — no ortho bake |
| Ortho smoke bake (CI only) | — | `tile-batch-run` + `RUST_ENGINE_TILE_DRY_RUN` | `tile_batch_run` | **CI** — not production |
| Single variant bake | Blender headless | `variant-bake <set.json> <key>` | `variant_bake` | **SHIPPED** (APS-VAR-003) |
| Validate variant_set_v1 | — | `variant-set-validate <path>` | `variant_set_validate` | **SHIPPED** (APS-VAR-001) |
| Patch variant_set layers/tags | — | `variant-set-patch <path> <patch.json>` | `variant_set_patch` | **SHIPPED** (APS-VAR-002) |
| Agent patch proposal (stub) | — | `variant-agent-request <req.json>` | `variant_agent_request` | **SHIPPED** (APS-AGENT-001) |
| Register tile atlas index | — | `tile-atlas-register <batch_id>` | `tile_atlas_register` | **SHIPPED** (TILE-REAL-001 R1) |
| Assembly import scene | Blender manual | — | `assembly_import_blender` | **PLANNED** |
| Keyframe still export (manual) | **Keyframe render addon** button | — | — | **Manual Blender** (authoritative until G4 green) |
| Keyframe still export (headless, optional) | — | `tile-keyframe-export <batch.json>` | `tile_keyframe_export_tool` | **`RUST_ENGINE_TILE_KEYFRAME_HEADLESS=1`** · `tile_keyframe_bake.py` |

**Legacy binaries (wrapped, not duplicated):** `utils/tilemapgen`, `utils/keyframe_render.py`, `utils/Light_keysshotsetup.blend`.

**Agent prompts:** use MCP tools + `validate_report` — never raw `cargo`/`blender` prose for these steps.

Docs: [`utils/LEGACY_ART_PIPELINE_README.md`](../../utils/LEGACY_ART_PIPELINE_README.md) · [`docs/archive/2026-06-src-dev/plans/plan_art_preview_hub_v1.md`](../../docs/archive/2026-06-src-dev/plans/plan_art_preview_hub_v1.md).

## Tier 3 — planned adapters (exec plan)

| Tool | Binary | CLI/MCP name (future) |
|:---|:---|:---|
| glTF transform | `@gltf-transform/cli` | `mesh-optimize`, `mesh-draco` |
| ImageMagick | `magick` | `thumb-render` |
| Material Maker | MM CLI | `material-generate-set` |
| gltfpack | `gltfpack` | `mesh-lod` |
| Rust art_validator | `cargo run -p art_validator` | `validate-engine-rules` |

## Tier 4 — reference (read-only, no generation)

| MCP (future) | Source |
|:---|:---|
| `reference_osm_tags` | Overpass API metadata |
| `reference_cite_manual` | Local PDF/manual path |

## Anti-patterns

| Don't | Do |
|:---|:---|
| Ask LLM to write bpy in chat | `geometry_run_job` with JSON params |
| Paste base64 textures | Material Maker CLI + manifest |
| Manually copy glb paths | `promote` |
| Re-describe module grid in every message | `AssetSpec` JSON once |
| End turn with “waiting on planner” | `agent_queue_next` → drain fallback slice |
| Read full witness / HANDOFF / queue JSON | `witness_brief` / `handoff_brief` / `agent_queue_board` |
| `cargo check` output in chat | `validate_cargo_report(compress=4, use_cached=true)` |

## Config files

| File | Purpose |
|:---|:---|
| `tools/mcp/config.defaults.json` | Steam Blender + repo paths |
| `tools/mcp/config.local.json` | Machine override (gitignored) |
| `~/.cursor/mcp.json` | Cursor MCP server wiring |
| `~/.cursor/rust_engine_art_mcp.env` | Machine path reference (dotenv-style) |
| `~/.cursor/rust_engine_art_mcp.env.md` | Human quick reference + re-sync commands |
