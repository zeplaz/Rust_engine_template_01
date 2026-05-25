use crate::knowledge::KnowledgeBase;
use crate::models::{DiagnosticIssue, Severity, SystemLifecycle, WarningState};
use crate::models::SourceAnnotation;
use crate::subsystem::{related_systems_for_tag, trace_subsystem};

pub fn classify_warnings(
    issues: &mut [DiagnosticIssue],
    annotations: &[SourceAnnotation],
    knowledge: &KnowledgeBase,
) {
    for issue in issues.iter_mut() {
        let tag = trace_subsystem(&issue.file);
        issue.subsystem = tag.display_path();
        issue.related_systems = related_systems_for_tag(&tag);

        if let Some(ann) = annotations.iter().find(|a| a.file == issue.file && a.line == issue.line)
        {
            apply_annotation(issue, ann);
        }

        classify_by_rules(issue, knowledge);
        issue.architectural_context = build_context(issue, knowledge);
        issue.lifecycle = infer_lifecycle(issue);
        if issue.recommended_action.is_empty() {
            issue.recommended_action = default_action(issue);
        }
    }
}

fn apply_annotation(issue: &mut DiagnosticIssue, ann: &SourceAnnotation) {
    issue.owner = ann.owner.clone();
    issue.do_not_touch = ann.do_not_cleanup
        || ann.status.eq_ignore_ascii_case("IN_PROGRESS")
        || ann.status.eq_ignore_ascii_case("INTENTIONAL_STAGING");
    match ann.status.to_uppercase().as_str() {
        "IN_PROGRESS" => {
            issue.state = WarningState::InProgressMigration;
            issue.lifecycle = SystemLifecycle::InProgress;
        }
        "INTENTIONAL_STAGING" | "STAGED" => {
            issue.state = WarningState::IntentionalStub;
            issue.lifecycle = SystemLifecycle::IntentionalStaging;
        }
        "LEGACY_TRANSITION" | "LEGACY" => {
            issue.state = WarningState::TransitionalArchitecture;
            issue.lifecycle = SystemLifecycle::LegacyTransition;
        }
        "BROKEN" => {
            issue.state = WarningState::ActiveBug;
            issue.lifecycle = SystemLifecycle::Broken;
        }
        _ => {}
    }
    if let Some(note) = &ann.note {
        issue.architectural_context.push(note.clone());
    }
}

fn classify_by_rules(issue: &mut DiagnosticIssue, knowledge: &KnowledgeBase) {
    let msg = issue.message.to_lowercase();
    let code = issue.rustc_code.as_deref().unwrap_or("");
    let file = issue.file.replace('\\', "/");
    let sym = issue.symbol.to_lowercase();

    if msg.contains("deprecated") || knowledge.is_deprecated_symbol(&issue.symbol) {
        issue.state = WarningState::DeprecatedApi;
        issue.severity = Severity::Critical;
        issue.do_not_touch = true;
        issue.lifecycle = SystemLifecycle::LegacyTransition;
        if let Some(m) = knowledge.migration_for_symbol(&issue.symbol) {
            issue.migration_target = Some(m.replacement.join(", "));
            issue.blockers = m.blockers.clone();
            issue.owner = m.owner.clone();
            issue.related_systems = m.affected_systems.clone();
        }
        return;
    }

    if code == "dead_code" || msg.contains("never used") || msg.contains("never constructed") {
        if file.contains("viewport") || sym.contains("rescue") || sym.contains("frozen") {
            issue.state = WarningState::DeferredCleanup;
            issue.do_not_touch = true;
            issue.lifecycle = SystemLifecycle::InProgress;
            issue.architectural_context.push(
                "Possible abandoned viewport stabilization logic — may relate to viewport drift."
                    .into(),
            );
        } else {
            issue.state = WarningState::DeadCode;
            issue.do_not_touch = file.contains("viewport") || file.contains("stage5");
        }
        return;
    }

    if code == "unused_imports" {
        if file.contains("sim_view_sync_debug") || file.contains("viewport_authority") {
            issue.state = WarningState::TransitionalArchitecture;
            issue.do_not_touch = true;
            issue.lifecycle = SystemLifecycle::InProgress;
            issue.architectural_context.push(
                "Imports imply planned integration points; likely pending instrumentation migration."
                    .into(),
            );
        } else {
            issue.state = WarningState::DeferredCleanup;
        }
        return;
    }

    if msg.contains("private") && (msg.contains("visible") || msg.contains("interface")) {
        issue.state = WarningState::VisibilityViolation;
        issue.severity = Severity::Critical;
        issue.architectural_context
            .push("Architectural leak: debug API may expose private types.".into());
        if sym.contains("trace_sim_view_sync") {
            issue.recommended_action =
                "Prefer `pub(crate) fn trace_sim_view_sync_state` or make `SimViewSyncCtx` public intentionally.".into();
        }
        return;
    }

    if code == "private_interfaces" || msg.contains("more private than the item") {
        issue.state = WarningState::VisibilityViolation;
        issue.severity = Severity::Critical;
        if sym.is_empty() {
            if let Some(name) = msg.split('`').nth(1) {
                issue.symbol = name.to_string();
            }
        }
        return;
    }

    if code == "unused_mut" || code == "unused_variables" {
        issue.state = WarningState::DiagnosticNoise;
        issue.lifecycle = SystemLifecycle::Stable;
        return;
    }

    if code.starts_with("clippy::") && !issue.do_not_touch {
        issue.state = WarningState::DiagnosticNoise;
        return;
    }

    if issue.severity == Severity::Fatal {
        issue.state = WarningState::ActiveBug;
        issue.lifecycle = SystemLifecycle::Broken;
        issue.do_not_touch = false;
    }
}

fn infer_lifecycle(issue: &DiagnosticIssue) -> SystemLifecycle {
    if issue.do_not_touch {
        return match issue.state {
            WarningState::IntentionalStub => SystemLifecycle::IntentionalStaging,
            WarningState::DeprecatedApi | WarningState::TransitionalArchitecture => {
                SystemLifecycle::LegacyTransition
            }
            WarningState::InProgressMigration | WarningState::DeferredCleanup => {
                SystemLifecycle::InProgress
            }
            _ => SystemLifecycle::InProgress,
        };
    }
    match issue.state {
        WarningState::ActiveBug if matches!(issue.severity, Severity::Fatal) => {
            SystemLifecycle::Broken
        }
        WarningState::DeprecatedApi | WarningState::TransitionalArchitecture => {
            SystemLifecycle::LegacyTransition
        }
        _ => SystemLifecycle::Stable,
    }
}

fn build_context(issue: &DiagnosticIssue, knowledge: &KnowledgeBase) -> Vec<String> {
    let mut ctx = issue.architectural_context.clone();
    if let Some(node) = knowledge.graph_node_for_file(&issue.file) {
        ctx.push(format!("Knowledge graph: {}", node));
    }
    ctx
}

fn default_action(issue: &DiagnosticIssue) -> String {
    match issue.state {
        WarningState::DeprecatedApi => {
            format!(
                "Complete migration to `{}` before removing deprecated `{}`.",
                issue.migration_target.as_deref().unwrap_or("replacement API"),
                issue.symbol
            )
        }
        WarningState::TransitionalArchitecture | WarningState::InProgressMigration => {
            "Document intent; do not auto-delete. Finish migration path first.".into()
        }
        WarningState::DeadCode | WarningState::DeferredCleanup => {
            "Tag with @orchestrator-status; confirm abandonment vs pending wiring before removal."
                .into()
        }
        WarningState::VisibilityViolation => {
            "Tighten visibility (`pub(crate)`) or promote types intentionally.".into()
        }
        WarningState::ActiveBug if matches!(issue.severity, Severity::Fatal) => {
            "Fix compile error before cleanup.".into()
        }
        WarningState::DiagnosticNoise => "Safe for dedicated cleanup agent when migration green."
            .into(),
        _ => "Review with subsystem owner.".into(),
    }
}
