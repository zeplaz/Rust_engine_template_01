# Skills Reference

Source of truth: `.claude/skills/` → mirrored to `.cursor/skills/`

## Sync First

```powershell
powershell -NoProfile -File .cursor/skills/sync-claude-skills/scripts/sync.ps1
```

## Every Session

1. Attach [agent-lang/SKILL.md](agent-lang/SKILL.md) — BLANG, queue, witnesses, validators
2. Read domain skills per role (see `opencode/agents/README.md`)

## Skill Tree

```
agent-lang/           Base layer (all agents)
├── driver.mjs       CLI bootstrap
├── SKILL.md         Core instructions
└── reference.md     Reference docs

bevy-simulation-grade/  ECS / viewport authority
├── 00-core-ecs-execution-model.md
├── 01-view-authority-viewmanager.md
├── 02-viewport-authority-pipeline.md
├── 03-map-view-projection-and-render-contract.md
├── 04-render-projection-graph.md
├── 05-construction-ghost-overlay.md
├── 06-parallel-simulation-and-cleanup.md
├── 07-repo-authority-map.md
├── 08-bevy-018-guardrails.md
├── 09-sim-map-projection-placement.md
└── SKILL.md

debug-intelligence/       Witness drift, viewport/ECS
validation-first/         Build validation reports
operations-intelligence/  OPS witness spine, DSM/QCE
cleanup-completion-intelligence/  Before deletes
mcp-asset-pipeline/       MCP art lane
mcp-production-rules/     Ship gates, bake source
tile-generation/          Iso tile / atlas
blender-geometry/         bpy modules
```