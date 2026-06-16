# Validation Runtime Architecture `v1`

| Field | Value |
|:---|:---|
| **Plan ID** | PLAN-VALIDATION-RUNTIME-001 |
| **Date** | 2026-06-02 |
| **Status** | **SHIPPED (foundation)** |

## Problem

Agents running `cargo check` / reading build logs in a loop burn tokens on repeated raw compiler output.

## Target flow

```text
Agent → Task request → Execution runtime → Validators → ValidationReport → Agent
```

Agents **never** see raw logs unless report confidence is low.

## Shipped components

| Component | Path | Status |
|:---|:---|:---:|
| ValidationReport schema | `tools/validators/schemas/validation_report_v1.schema.json` | SHIPPED |
| Error knowledge base | `tools/validators/knowledge/error_signatures.json` | SHIPPED |
| cargo_validator | `rust_engine_mcp/validators/cargo.py` | SHIPPED |
| bevy_validator | `rust_engine_mcp/validators/bevy.py` | SHIPPED |
| mcp_schema_validator | `rust_engine_mcp/validators/mcp_schema.py` | SHIPPED |
| asset_glb_validator | `rust_engine_mcp/validators/asset.py` | SHIPPED |
| MCP tools | `validate_cargo_report`, `validate_bevy_report`, `validate_asset_report`, `validate_report` | SHIPPED |
| CLI | `validate-report <validator>` | SHIPPED |
| Project rule | `.cursor/rules/validation-first.mdc` | SHIPPED |
| Skill | `.cursor/skills/validation-first/` | SHIPPED |
| Orchestrator bridge | `--cached` reads `tools/orchestrator/state/last_run.json` | SHIPPED |

## Planned

| Validator | Notes |
|:---|:---|
| test_validator | Parse `cargo test --message-format=json` |
| tile_validator | Tile batch schema + atlas rules |
| blender_validator | Headless job log classifier |
| gltf_validator | gltf-transform integration |
| schema_validator | Generic JSON schema registry |
| Rust `art_validator` crate | Grid/pivot engine rules |

## Compression levels

1. Raw issues (cap 50) + log path  
2. Cap 20 issues  
3. Cap 8 issues + known_fixes (**default**)  
4. summary + known_fixes only  

## Agent policy

All agents (`coder`, `designer`, `orchestrator`, `*-mcp`, `sim-steward`) attach **validation-first** skill for build/test/MCP work.

## Extending knowledge base

When an agent fixes a recurring error, append to `error_signatures.json`:

```json
{
  "signature": "E0308_MySymbol",
  "match": { "rustc_code": "E0308", "symbol": "MySymbol" },
  "fix": "Replace Foo with Bar in ...",
  "confidence": 0.95
}
```
