---
name: validation-first
description: >-
  Reason on STRUCTURED validation reports, never raw compiler/tool output. Use
  after any build, test, GLB/asset check, or schema validation — when about to
  read `cargo check` stderr, a wall of warnings, or tool logs. The pattern:
  validator → ValidationReport JSON (compressed) → act on summary/errors/known_fixes;
  escalate to raw logs only on low confidence. Triggers: cargo, build failed,
  warnings, validate-report, ValidationReport, known_fixes, compile errors, GLB validate.
---

`⟦SYM⟧ lang⊳ $ref:prompts/SYMBOLIC_LANGUAGE.meta.md`

# validation-first — structured reports, not raw logs

## The pattern (form B + ⬡ gate)

Raw compiler/tool dump = high 💰, low signal, tempts line-by-line flailing. Route every check through a validator that emits a typed report; reason only on its fields.

```text
▢task ▷⊳ ▢validator ─⬡[emit ValidationReport JSON ▾3–▾4]▶ ◎report
  reason ON ⦃ status ║ summary ║ errors[] ║ known_fixes[] ║ confidence ⦄  ⛔▶ stderr

◆ verdict ?
 ├─═[known_fix ∧ conf ≥ ◕(.9)]▶ ▢apply
 ├─═[known_fix ∧ conf < ◕]▶ ▢propose ¬blind-apply
 └─⬡ escalate-to-raw ═[conf < ◑(.7) ∨ (errors∅ ∧ status:failed) ∨ user-asks]▶ ◎raw_log_path
       read the PATH ¬contents until escalation earned
```

Transfers to any pipeline: wrap the noisy tool, return a compressed verdict, gate on confidence. Adapt validator set + schema to the env.

## In this repo (reference implementation)

Run validators through the shared agent-lang driver (auto-UTF-8, right cwd):

```bash
# cargo — use the orchestrator cache (instant); omit --cached only for a fresh build
node .claude/skills/agent-lang/driver.mjs validate-report cargo --cached --compress 4
# GLB asset (path is repo-relative; resolves internally)
node .claude/skills/agent-lang/driver.mjs validate-report asset_glb assets/staging/wall_brick_1u_example/model.glb --compress 4
# discover all validators
node .claude/skills/agent-lang/driver.mjs validate-report --help
```

Verified output shapes (this session):

```text
◎cargo --cached  ▷⊳ {"status":"passed","error_count":0,"warning_count":45,"compression_level":4,…}
◎asset_glb       ▷⊳ {"status":"passed","summary":"model.glb: verts=24 tier=smoke arch=module_wall profile=brick","confidence":1.0}
```

Validator ids: `cargo, bevy, mcp_spec, mcp_job, asset_glb, tile_batch, atlas_meta_v2, visual_config, tile_promotion, assembly_grammar, assembly_p0, assembly_production, material_profiles`.

| Validator | Driver command |
|:--|:--|
| cargo | `validate-report cargo --cached --compress 4` |
| bevy (API-sensitive) | `validate-report bevy -p <package> --compress 4` |
| GLB asset | `validate-report asset_glb <path> --compress 4` |
| MCP spec / job | `validate-report mcp_spec <path>` / `validate-report mcp_job <path>` |

### Compression levels

| Level | Agent sees |
|:--|:--|
| 1 | up to 50 issues + `raw_log_path` |
| 2 | up to 20 issues |
| 3 | up to 8 issues + `known_fixes` (default) |
| 4 | `summary` + `known_fixes` only |

## Gotchas

```text
⚠ --cached reads tools/orchestrator/state/last_run.json   instant · as fresh as last orchestrated run ; drop --cached ▶ real `cargo` build (slow ⏱)
⚠ path args differ by validator   asset_glb/witness/telemetry = repo-relative (resolved internally) ; run-geometry & file-opening cmds = relative to tools/mcp/python ⇒ use absolute paths there
⚠ also a Cursor *rule* (alwaysApply)   applies to ∀ agent ⦃ @coder ║ @designer ║ @planner ║ -mcp lane ║ @sim-steward ⦄
```

## Source

```text
◎.cursor/skills/validation-first/SKILL.md   Cursor original
◎.cursor/rules/validation-first.mdc          always-on rule
◎tools/validators/schemas/validation_report_v1.schema.json   schema
⊗ pairs-with [agent-lang](../agent-lang/SKILL.md) (BLANG:CARGO / BLANG:BEVY)
```

```text
⟦/validation-first⟧ NEXT ⚑ validate-report <id> --compress 4 → ◆verdict → act on known_fix(conf≥◕) · escalate raw only <◑
```
