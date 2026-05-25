use crate::architectural::ArchitecturalAnalysis;
use crate::cargo_collect::{count_rustc_stderr_warnings, CargoPhaseResult};
use crate::scanner::scan_deprecated_symbols;
use crate::models::{
    ContinuationTask, DiagnosticIssue, OrchestratorSnapshot, SemanticMarker, ThreadHealth,
    WarningState,
};
use crate::ownership::ownership_map_markdown;
use crate::state::OrchestratorPaths;
use crate::subsystem::subsystem_graph_markdown;
use std::fs;
use std::io::Write;

pub fn generate_reports(
    paths: &OrchestratorPaths,
    snapshot: &OrchestratorSnapshot,
    phases: &[CargoPhaseResult],
    analysis: &ArchitecturalAnalysis,
    repo_root: &std::path::Path,
    markers: &[SemanticMarker],
) -> std::io::Result<()> {
    write_file(&paths.reports.join("build_report.md"), &build_report(snapshot, phases))?;
    write_file(
        &paths.reports.join("warning_registry.md"),
        &warning_registry(&snapshot.issues),
    )?;
    write_file(
        &paths.reports.join("subsystem_graph.md"),
        &format!(
            "# Subsystem graph\n\n{}\n",
            subsystem_graph_markdown()
        ),
    )?;
    write_file(
        &paths.reports.join("ownership_map.md"),
        &ownership_map_markdown(&snapshot.issues),
    )?;
    write_file(
        &paths.reports.join("migration_tasks.md"),
        &migration_tasks(&analysis.continuation_tasks),
    )?;
    write_file(
        &paths.runbooks.join("viewport_pipeline.md"),
        &viewport_pipeline_runbook(),
    )?;
    write_file(
        &paths.runbooks.join("render_pipeline.md"),
        &render_pipeline_runbook(),
    )?;
    write_file(
        &paths.runbooks.join("ui_pipeline.md"),
        &ui_pipeline_runbook(),
    )?;
    let agent_q = agent_queue(&analysis);
    write_file(&paths.queues.join("agent_queue.md"), &agent_q)?;
    write_file(&paths.reports.join("agent_queue.md"), &agent_q)?;
    let static_dep = scan_deprecated_symbols(&repo_root.join("src"));
    write_file(
        &paths.reports.join("deprecation_tracker.md"),
        &deprecation_tracker(&snapshot.issues, &static_dep),
    )?;
    write_file(
        &paths.reports.join("thread_health.md"),
        &thread_health_report(&snapshot.thread_health),
    )?;
    write_file(
        &paths.reports.join("system_completion.md"),
        &crate::architectural::system_completion_markdown(analysis),
    )?;
    write_file(
        &paths.reports.join("marker_triage.md"),
        &marker_triage(markers),
    )?;
    write_file(
        &paths.agents.join("coordination_report.md"),
        &agent_coordination_report(snapshot, analysis),
    )?;
    write_file(
        &paths.queues.join("continuation_queue.json"),
        &serde_json::to_string_pretty(&analysis.continuation_tasks)?,
    )?;
    Ok(())
}

fn write_file(path: &std::path::Path, content: &str) -> std::io::Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    let mut f = fs::File::create(path)?;
    f.write_all(content.as_bytes())?;
    Ok(())
}

fn build_report(snapshot: &OrchestratorSnapshot, phases: &[CargoPhaseResult]) -> String {
    let mut s = String::from("# Build report\n\n");
    s.push_str(&format!("- **Run ID:** `{}`\n", snapshot.meta.run_id));
    s.push_str(&format!("- **Started:** {}\n", snapshot.meta.started_at));
    s.push_str(&format!("- **Finished:** {}\n", snapshot.meta.finished_at));
    s.push_str(&format!("- **Issues:** {}\n", snapshot.meta.issue_count));
    s.push_str(&format!(
        "- **Do-not-touch:** {}\n\n",
        snapshot.meta.do_not_touch_count
    ));
    let stderr_warnings = count_rustc_stderr_warnings(phases);
    s.push_str(&format!(
        "- **Rustc stderr warnings (scraped):** {}\n\n",
        stderr_warnings
    ));
    s.push_str("## Cargo phases\n\n| Phase | OK | Stderr warnings |\n|-------|----|----------------:|\n");
    for p in phases {
        let phase_warns = p
            .stderr
            .lines()
            .filter(|l| l.contains("warning:") && !l.contains("warnings emitted"))
            .count();
        s.push_str(&format!(
            "| `{}` | {} | {} |\n",
            p.name,
            if p.ok { "yes" } else { "no" },
            phase_warns
        ));
    }
    s.push_str("\n> Diagnostics parsed from `--message-format=json` (not stderr scraping).\n");
    s
}

fn warning_registry(issues: &[DiagnosticIssue]) -> String {
    let mut s = String::from("# Warning registry\n\n");
    s.push_str("| State | File | Line | Symbol | Do-not-touch |\n");
    s.push_str("|-------|------|-----:|--------|:------------:|\n");
    for i in issues {
        s.push_str(&format!(
            "| {:?} | `{}` | {} | `{}` | {} |\n",
            i.state,
            i.file,
            i.line,
            i.symbol,
            if i.do_not_touch { "yes" } else { "no" }
        ));
    }
    s
}

fn migration_tasks(tasks: &[ContinuationTask]) -> String {
    let mut s = String::from("# Migration tasks\n\n");
    for t in tasks {
        s.push_str(&format!("## {}\n\n", t.id));
        s.push_str(&format!("**{}** — risk: {}\n\n", t.title, t.risk));
        s.push_str(&format!("- Agent: `{}`\n", t.recommended_agent));
        s.push_str(&format!("- Do not touch: {}\n", t.do_not_touch));
        s.push_str("- Deprecated:\n");
        for d in &t.deprecated_symbols {
            s.push_str(&format!("  - `{d}`\n"));
        }
        s.push_str("- Replacement:\n");
        for r in &t.replacement_symbols {
            s.push_str(&format!("  - `{r}`\n"));
        }
        s.push_str("- Blockers:\n");
        for b in &t.blockers {
            s.push_str(&format!("  - {b}\n"));
        }
        s.push('\n');
    }
    s
}

fn viewport_pipeline_runbook() -> String {
    r#"# Viewport pipeline runbook

## STATUS: migration COMPLETE (2026-05-20)

Witness: `debug_runs/viewport_authority_migration_witness.json`

## File map

| Stage | File | Symbol / resource |
|-------|------|-------------------|
| Measure | `src/gui/authoritative_viewport.rs` | `measure_sim_map_fill_viewport` |
| Semantic | `src/gui/viewport_layout_solver.rs` | `semantic_viewport_from_map_fill` |
| Commit | `src/gui/viewport_layout_solver.rs` | `commit_authority_from_semantic` |
| Rescue floor | `src/gui/viewport_layout_solver.rs` | `viewport_rescue_floor` |
| Debug trace | `src/gui/hud/viewport_authority_debug.rs` | `trace_viewport_authority` |
| Sync trace | `src/gui/hud/sim_view_sync_debug.rs` | `trace_sim_view_sync_state` |
| Camera | `src/gui/map_camera.rs` | `MainWorldCamera`, `MapCameraDesired` |
| Render | `src/render/` | `ResolvedViewports` |

## Authoritative path

1. `sim_map_fill` UI measure → `SemanticViewportRect`
2. `commit_authority_from_semantic` → `AuthoritativeViewport` / `SimulationMapViewport`
3. Camera + render copy — **no** window-chrome re-derive

## Drift reproduction

```text
SIM_VIEW_SYNC_DEBUG=1
STAGE5_VERBOSE=1
--debug-sim-view-sync
RUST_LOG=sim_view_sync=info,sim_view_sync::anomaly=warn
```

## Staging (do not delete)

- `frozen_exceeds_semantic_authority` — heal hud_root overshoot (IN_PROGRESS)
- `sim_view_sync_debug` imports — instrumentation expansion

## Agent rules

| Action | Allowed |
|--------|---------|
| Change semantic solver | only with viewport_migration_agent |
| Delete frozen/rescue helpers | **no** without witness update |
| Visibility tighten | yes (`pub(crate)`) |
"#
    .to_string()
}

fn render_pipeline_runbook() -> String {
    r#"# Render pipeline runbook

## File map

| Node | File |
|------|------|
| Projection graph | `src/render/extraction/render_projection_graph.rs` |
| Fire extract | `src/render/extraction/fire_visual_extract.rs` |
| View extract | `src/render/extraction/fire_view_extract.rs` |
| Resolved viewports | `src/render/viewport_pipeline.rs` (ViewportPipelinePlugin) |
| Visual diagnostics | `src/render/visual_diagnostics.rs` |
| Tile fallback | `src/render/tile_world_fallback.rs` |

## Spine

```text
RepresentationResult + WorldLodMap
    → RenderProjectionGraph (CPU nodes)
    → FireVisualFrame / buffers
    → ResolvedViewports (follows GUI semantic authority)
    → GPU upload / draw
```

## Coordination

Render agents treat GUI viewport authority as upstream. Never re-derive simulation map geometry from window chrome.

## STAGE5

- TODO-06–11 — frame fence, fire alignment, GPU spine (`STAGE5_TODOS`)
"#
    .to_string()
}

fn ui_pipeline_runbook() -> String {
    r#"# UI pipeline runbook

## File map

| Area | File |
|------|------|
| Map spine | `src/gui/map_view/mod.rs` |
| HUD shell | `src/gui/hud/hud_root_tick.rs` |
| World preview | `src/gui/editor/world_preview/` |
| Viewport measure | `src/gui/authoritative_viewport.rs` |

## egui / schedule order (`MapViewPlugin`)

1. `Update`: `sync_resolved_map_view_frames` (after render `ViewportPipelineSet::Resolve`)
2. `PostUpdate`: `update_world_preview_view` → `update_minimap_view` → interaction commit
3. `EguiPrimaryContextPass`: `clear_active_map_view_input` **before** `hud_product_shell_egui_root` **before** `display_world_preview`
4. `sync_map_fit_transform_components` → `validate_map_fit_system`

## IN PROGRESS

- `map_view` presentation spine (`@orchestrator-status IN_PROGRESS`)
- `sim_view_sync_debug` instrumentation

## Agents

- `ui_layout_agent` — map_view, egui ordering
- `viewport_migration_agent` — measure + semantic authority
"#
    .to_string()
}

fn marker_triage(markers: &[SemanticMarker]) -> String {
    let mut s = String::from("# Marker triage (A-06)\n\n");
    s.push_str("| Kind | File | Line | Notes |\n");
    s.push_str("|------|------|-----:|-------|\n");
    if markers.is_empty() {
        s.push_str("| — | — | — | _No markers found._ |\n");
        return s;
    }
    for m in markers.iter().take(200) {
        let note = m.text.chars().take(80).collect::<String>();
        s.push_str(&format!(
            "| {} | `{}` | {} | {} |\n",
            m.kind, m.file, m.line, note.replace('|', "/")
        ));
    }
    if markers.len() > 200 {
        s.push_str(&format!("\n_…and {} more._\n", markers.len() - 200));
    }
    s.push_str("\n## Triage policy\n\n");
    s.push_str("- `VIEWPORT_AUTHORITY` / `MIGRATION` → do not auto-clean\n");
    s.push_str("- `TODO` / `FIXME` → link to STAGE5 or continuation queue\n");
    s.push_str("- `HACK` / `TEMP` → requires owner in knowledge JSON\n");
    s
}

fn agent_queue(analysis: &ArchitecturalAnalysis) -> String {
    let mut s = String::from("# Agent queue\n\n");
    s.push_str("## Project Cursor agents (`.cursor/agents/`)\n\n");
    for name in ["orchestrator", "planner", "coder", "designer"] {
        s.push_str(&format!("- `@{}` — `.cursor/agents/{name}.md`\n", name));
    }
    s.push_str("\n## Lane playbooks (`tools/orchestrator/agents/`)\n\n");
    for name in [
        "stage5_readiness_agent",
        "viewport_cleanup_agent",
        "render_pipeline_agent",
        "ui_layout_agent",
        "dead_code_agent",
        "migration_tracker_agent",
        "runbook_sync_agent",
        "thread_health_agent",
        "warning_classifier_agent",
    ] {
        s.push_str(&format!("- `{name}`\n"));
    }
    s.push_str("\n## ACTIVE MIGRATIONS\n\n");
    if analysis.active_migrations.is_empty() {
        s.push_str("_None — infrastructure viewport migration complete; use @coder for new work._\n\n");
    }
    for m in &analysis.active_migrations {
        s.push_str(&format!("### {}\n", m.id));
        s.push_str(&format!("STATUS: {}\n\n", m.status));
        s.push_str("DO_NOT_TOUCH:\n");
        for x in &m.do_not_touch {
            s.push_str(&format!("- {x}\n"));
        }
        s.push_str("\nSAFE:\n");
        for x in &m.safe {
            s.push_str(&format!("- {x}\n"));
        }
        s.push_str("\nBLOCKED_BY:\n");
        for x in &m.blocked_by {
            s.push_str(&format!("- {x}\n"));
        }
        s.push('\n');
    }
    s
}

fn deprecation_tracker(
    issues: &[DiagnosticIssue],
    static_dep: &[(String, String, usize)],
) -> String {
    let mut s = String::from("# Deprecation tracker\n\n");
    s.push_str("## Compiler (last run)\n\n");
    let deps: Vec<_> = issues
        .iter()
        .filter(|i| i.state == WarningState::DeprecatedApi)
        .collect();
    if deps.is_empty() {
        s.push_str("_None._\n\n");
    }
    for i in deps {
        s.push_str(&format!(
            "- `{}` — `{}` L{} → {}\n",
            i.symbol,
            i.file,
            i.line,
            i.migration_target.as_deref().unwrap_or("see runbook")
        ));
    }
    s.push_str("\n## Static scan (`#[deprecated]`)\n\n");
    if static_dep.is_empty() {
        s.push_str("_None found._\n");
    } else {
        for (sym, note, line) in static_dep {
            s.push_str(&format!("- `{sym}` L{line} — {note}\n"));
        }
    }
    s
}

fn thread_health_report(health: &[ThreadHealth]) -> String {
    let mut s = String::from("# Thread health\n\n");
    s.push_str("> Runtime heartbeat placeholders — wire from app diagnostics in a future cycle.\n\n");
    s.push_str("| Thread | Alive | Stalled frames | Avg frame ms | Notes |\n");
    s.push_str("|--------|:-----:|---------------:|-------------:|-------|\n");
    for t in health {
        s.push_str(&format!(
            "| {} | {} | {} | {:.2} | {} |\n",
            t.name,
            if t.alive { "yes" } else { "no" },
            t.stalled_frames,
            t.avg_frame_ms,
            t.notes
        ));
    }
    s
}

fn agent_coordination_report(
    snapshot: &OrchestratorSnapshot,
    analysis: &ArchitecturalAnalysis,
) -> String {
    format!(
        r#"# Agent coordination report

Run: `{}` at {}

## Summary

- Issues: {}
- Do-not-touch: {}
- Continuation tasks: {}
- Broken / in-progress / legacy: {} / {} / {}

## Instruction

This orchestrator preserves **architectural intent**. Cleanup agents must read `agent_queue.md` and respect `do_not_touch` on each issue before editing.

## Top continuation tasks

{}
"#,
        snapshot.meta.run_id,
        snapshot.meta.finished_at,
        snapshot.meta.issue_count,
        snapshot.meta.do_not_touch_count,
        analysis.continuation_tasks.len(),
        analysis.broken,
        analysis.in_progress,
        analysis.legacy_transition,
        analysis
            .continuation_tasks
            .iter()
            .take(5)
            .map(|t| format!("- **{}** ({}) — agent `{}`\n", t.title, t.risk, t.recommended_agent))
            .collect::<String>()
    )
}
