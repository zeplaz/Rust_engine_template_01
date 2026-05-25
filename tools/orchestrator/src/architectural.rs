use crate::knowledge::KnowledgeBase;
use crate::models::{
    ActiveMigration, ContinuationTask, DiagnosticIssue, SystemLifecycle, WarningState,
};
use crate::models::SemanticMarker;

pub fn analyze_architectural_state(
    issues: &[DiagnosticIssue],
    markers: &[SemanticMarker],
    knowledge: &KnowledgeBase,
) -> ArchitecturalAnalysis {
    let mut broken = 0usize;
    let mut in_progress = 0usize;
    let mut staging = 0usize;
    let mut legacy = 0usize;
    let mut stable = 0usize;

    for issue in issues {
        match issue.lifecycle {
            SystemLifecycle::Broken => broken += 1,
            SystemLifecycle::InProgress => in_progress += 1,
            SystemLifecycle::IntentionalStaging => staging += 1,
            SystemLifecycle::LegacyTransition => legacy += 1,
            SystemLifecycle::Stable => stable += 1,
        }
    }

    let continuation_tasks = build_continuation_tasks(issues, knowledge);
    let active_migrations = knowledge.active_migrations.clone();

    let incomplete_systems = detect_incomplete_systems(issues, markers);

    ArchitecturalAnalysis {
        broken,
        in_progress,
        intentional_staging: staging,
        legacy_transition: legacy,
        stable,
        continuation_tasks,
        active_migrations,
        incomplete_systems,
    }
}

pub struct ArchitecturalAnalysis {
    pub broken: usize,
    pub in_progress: usize,
    pub intentional_staging: usize,
    pub legacy_transition: usize,
    pub stable: usize,
    pub continuation_tasks: Vec<ContinuationTask>,
    pub active_migrations: Vec<ActiveMigration>,
    pub incomplete_systems: Vec<String>,
}

fn build_continuation_tasks(
    issues: &[DiagnosticIssue],
    knowledge: &KnowledgeBase,
) -> Vec<ContinuationTask> {
    let mut tasks = knowledge.seed_continuation_tasks.clone();

    let deprecated: Vec<_> = issues
        .iter()
        .filter(|i| i.state == WarningState::DeprecatedApi)
        .collect();
    if !deprecated.is_empty() && !tasks.iter().any(|t| t.id == "viewport_authority_refactor") {
        let symbols: Vec<String> = deprecated
            .iter()
            .map(|i| i.symbol.clone())
            .filter(|s| !s.is_empty())
            .collect();
        tasks.push(ContinuationTask {
            id: "viewport_authority_refactor".into(),
            title: "Viewport authority migration incomplete".into(),
            risk: "HIGH".into(),
            subsystem: "GUI / VIEWPORT_AUTHORITY".into(),
            deprecated_symbols: symbols,
            replacement_symbols: vec![
                "commit_authority_from_semantic".into(),
                "semantic_viewport_from_map_fill".into(),
            ],
            affected_systems: vec![
                "GUI".into(),
                "viewport authority".into(),
                "render synchronization".into(),
                "camera viewport".into(),
                "semantic layout solver".into(),
            ],
            blockers: vec!["viewport drift unresolved".into()],
            recommended_agent: "viewport_cleanup_agent".into(),
            do_not_touch: true,
        });
    }

    tasks
}

fn detect_incomplete_systems(
    issues: &[DiagnosticIssue],
    markers: &[SemanticMarker],
) -> Vec<String> {
    let mut systems = Vec::new();

    if issues.iter().any(|i| {
        i.file.contains("viewport")
            && matches!(
                i.state,
                WarningState::DeprecatedApi | WarningState::DeferredCleanup
            )
    }) {
        systems.push("viewport_authority (migration incomplete)".into());
    }

    if markers.iter().any(|m| m.kind == "VIEWPORT_AUTHORITY") {
        systems.push("viewport_authority (marked TODO/MIGRATION in source)".into());
    }

    if issues.iter().any(|i| i.file.contains("sim_view_sync_debug")) {
        systems.push("sim_view_sync_debug (instrumentation evolving)".into());
    }

    systems.sort();
    systems.dedup();
    systems
}

pub fn system_completion_markdown(analysis: &ArchitecturalAnalysis) -> String {
    format!(
        r#"# System completion

| Lifecycle | Count |
|-----------|------:|
| BROKEN | {} |
| IN_PROGRESS | {} |
| INTENTIONAL_STAGING | {} |
| LEGACY_TRANSITION | {} |
| STABLE | {} |

## Incomplete systems

{}

## Continuation queue size

{}
"#,
        analysis.broken,
        analysis.in_progress,
        analysis.intentional_staging,
        analysis.legacy_transition,
        analysis.stable,
        if analysis.incomplete_systems.is_empty() {
            "_None flagged._\n".into()
        } else {
            analysis
                .incomplete_systems
                .iter()
                .map(|s| format!("- {}\n", s))
                .collect::<String>()
        },
        analysis.continuation_tasks.len()
    )
}
