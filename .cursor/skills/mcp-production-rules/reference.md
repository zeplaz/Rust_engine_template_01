# MCP Production Rules — Reference

Merged from [`docs/archive/2026-06-fleet-drain/prompts_drafts/rules_skills_draft.md`](../../docs/archive/2026-06-fleet-drain/prompts_drafts/rules_skills_draft.md) §3, [`tools/mcp/MICRO_TOOLS_REGISTRY_v1.md`](../../tools/mcp/MICRO_TOOLS_REGISTRY_v1.md) anti-patterns, and [`.cursor/agents/designer.md`](../../agents/designer.md) art pipeline section.

## Purpose

Rules are **hard constraints**, not suggestions. Enforced **pre_execution** before toolchain calls.

## Rule definitions

### no_ai_generated_images

```yaml
id: no_ai_generated_images
type: hard_rule
enforcement: pre_execution
conditions:
  - never_generate_images_from_prompts
  - never_use_diffusion_models_for_final_assets
  - only_use_deterministic_tools
```

**Blocks:** `GenerateImage`, diffusion MCP, prompt-to-texture as final albedo.

**Allows:** Reference photos for metadata; OSM/manual cite (read-only).

### deterministic_output

```yaml
id: deterministic_output
type: hard_rule
enforcement: pre_execution
conditions:
  - same_input_always_same_output
  - no_random_unseeded_variation
  - all_variation_must_be_seed_based
```

**Requires:** Explicit `seed` in job JSON when variation is requested.

### batch_processing

```yaml
id: batch_processing
type: hard_rule
enforcement: pre_execution
conditions:
  - no_single_asset_generation
  - always_process_asset_groups
  - prefer_atlas_or_packaged_output
```

**Requires:** Batch specs (`tile_batch`, module sets, style pack groups).

### grid_alignment

```yaml
id: grid_alignment
type: hard_rule
enforcement: pre_execution
conditions:
  - all_assets_must_align_to_grid
  - no_free_rotation_in_isometric_system
  - tile_size_must_be_fixed_unit
```

**Aligns with:** [`design_procedural_module_kit_v1.md`](../../docs/archive/2026-06-src-dev/plans/design_procedural_module_kit_v1.md) — 1u module grid, bottom-center pivots.

## Enforcement pipeline (draft Rust pseudocode)

```rust
pub fn enforce_rules(agent: &Agent, input: &McpRequest) -> Result<(), String> {
    for rule_id in &agent.rules {
        match rule_id.as_str() {
            "no_ai_generated_images" => {
                if input.tool.contains("diffusion") {
                    return Err("AI image generation blocked".into());
                }
            }
            "deterministic_output" => {
                if input.contains_randomness() && input.seed.is_none() {
                    return Err("Non-deterministic input blocked".into());
                }
            }
            _ => {}
        }
    }
    Ok(())
}
```

In Cursor: agent manually applies checklist before `CallMcpTool` / CLI.

## Agent rule bundles (draft)

| Agent | Rules |
|-------|-------|
| designer_agent | grid_alignment, deterministic_output, no_ai_generated_images |
| coder_agent | no_pseudocode, always_compile_ready_rust, modular_ecs_design |
| simulation_agent | system_level_only, no_direct_asset_creation, must_emit_mcp_requests |

## Violation responses

| Violation | Response |
|-----------|----------|
| AI texture request | Reroute to Material Maker CLI or Blender bake |
| Chat bpy | Reroute to `geometry_run_job` + JSON |
| Unseeded random | Add `seed` or remove randomness |
| Single orphan tile | Wrap in batch/atlas spec |
| Off-grid mesh | Re-spec with module kit dimensions |

## Reproducibility (draft)

```rust
pub fn compute_hash(req: &McpRequest) -> String {
    // SHA256 of canonical JSON — full chain logging in future runtime
}
```

## Why this exists

Prevents:
- LLM hallucinated assets
- inconsistent pipelines
- ad-hoc prompts
- unstructured generation
- "AI slop" geometry/textures

Produces:
- deterministic pipelines
- strict tool execution
- reusable skill modules
- enforceable rules
- production-grade asset control
