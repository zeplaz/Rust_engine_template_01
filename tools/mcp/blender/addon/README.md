# Rust Engine Studio — Blender addon

Shared operators for the Artist Studio task bus (PLAN-ARTIST-STUDIO-EXPANSION-v1).

## Enable

1. Symlink or copy `rust_engine_studio/` into Blender’s addons folder, **or**
2. Preferences → Add-ons → Install → select a zip of `rust_engine_studio/`.
3. Enable **Import-Export: Rust Engine Studio**.

## Operators

| Operator | Action |
|:---|:---|
| Consume APS Handoff | Read latest `*.blender_handoff` from `tools/mcp/jobs/studio_tasks/` |
| Poll Studio Task | Print/status of `StudioTask` JSON |
| Run Geometry Job JSON | Enqueue StudioTask + run same `scripts/run_job.py` ops in-process |
| Open Staging | Open `assets/staging` |

APS writes handoffs via `python -m rust_engine_mcp.cli studio-task-handoff <task_id>`.
