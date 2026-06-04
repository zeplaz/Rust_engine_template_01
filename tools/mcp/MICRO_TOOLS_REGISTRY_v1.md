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

Queues: `grammar` → `grammar_continuation_queue.json` · `continuation` → `continuation_queue.json`  
Doc: [`src/dev/plan_agent_queue_mcp_v1.md`](../../src/dev/plan_agent_queue_mcp_v1.md)

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

Docs: [`utils/LEGACY_ART_PIPELINE_README.md`](../../utils/LEGACY_ART_PIPELINE_README.md) · [`src/dev/plan_art_preview_hub_v1.md`](../../src/dev/plan_art_preview_hub_v1.md).

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
