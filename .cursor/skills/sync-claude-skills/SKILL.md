---
name: sync-claude-skills
description: >-
  Retrieve skills from .claude/skills and sync them into the Cursor project
  base (.cursor/skills) when missing, empty, or stale. Use at session start,
  after editing .claude/skills, when agent-lang or other skills are absent from
  Cursor, or when the user asks to sync, retrieve, or mirror Claude skills into
  the base. Triggers: sync skills, retrieve skills, .claude skills, mirror
  skills, skill parity, empty SKILL.md.
---

# sync-claude-skills — mirror `.claude/skills` into Cursor base

## Authority map

| Role | Path |
|------|------|
| **Source (authoritative)** | `.claude/skills/<skill-name>/` |
| **Cursor project base** | `.cursor/skills/<skill-name>/` |
| **Optional personal base** | `~/.cursor/skills/<skill-name>/` |

`.claude/skills` is the canonical authoring tree (also referenced by agents, driver.mjs, and BLANG docs). `.cursor/skills` is what Cursor agents discover in this repo. Keep them aligned.

## When to run

- Session start if domain skills may be stale
- After creating or editing a skill under `.claude/skills/`
- When `.cursor/skills/<name>/SKILL.md` is missing or **0 bytes**
- When the user says: sync skills, retrieve skills, add to base, mirror claude skills

## Quick sync (preferred)

From repo root:

```powershell
powershell -NoProfile -File .cursor/skills/sync-claude-skills/scripts/sync.ps1
```

Dry run:

```powershell
powershell -NoProfile -File .cursor/skills/sync-claude-skills/scripts/sync.ps1 -WhatIf
```

Force overlay (overwrite even when destination exists and is non-empty):

```powershell
powershell -NoProfile -File .cursor/skills/sync-claude-skills/scripts/sync.ps1 -Force
```

## Manual checklist (if script unavailable)

```text
Task progress:
- [ ] List .claude/skills/* directories
- [ ] For each name, check .cursor/skills/<name>/SKILL.md exists and is non-empty
- [ ] Copy missing or empty skills from .claude → .cursor (entire directory tree)
- [ ] Do not delete .cursor-only extras (e.g. bevy-simulation-grade reference docs)
- [ ] Report added / updated / skipped skill names
```

## Sync rules

1. **Overlay only** — copy files from `.claude/skills/<name>/` into `.cursor/skills/<name>/`; never delete cursor-only files.
2. **Bootstrap** — if the destination directory is missing, create it and copy the full source tree.
3. **Repair** — if `SKILL.md` is missing or 0 bytes in `.cursor/skills`, copy from `.claude/skills`.
4. **Stale** — copy when a source file is newer than the destination (unless skipped by `-Force` logic above).
5. **Skip** — source directories without `SKILL.md` are not skills; warn and skip.

## Expected skill set (this repo)

After sync, these names should exist under `.cursor/skills/` with non-empty `SKILL.md`:

`agent-lang`, `bevy-simulation-grade`, `blender-geometry`, `cleanup-completion-intelligence`, `debug-intelligence`, `mcp-asset-pipeline`, `mcp-production-rules`, `operations-intelligence`, `tile-generation`, `validation-first`

## Optional: personal base

`AGENTS.md` notes `bevy-simulation-grade` may also live in `~/.cursor/skills/`. Only copy there when the user explicitly wants cross-project personal skills:

```powershell
$src = '.claude/skills/bevy-simulation-grade'
$dst = Join-Path $HOME '.cursor/skills/bevy-simulation-grade'
# mirror same overlay rules; never use ~/.cursor/skills-cursor/ (Cursor internal)
```

## After sync

1. Read newly synced `SKILL.md` files before relying on them in the session.
2. For `agent-lang`, the driver stays at `.claude/skills/agent-lang/driver.mjs` — paths in synced skills still point there by design.
3. Do not edit `.cursor/skills-cursor/` — Cursor-managed built-ins only.
