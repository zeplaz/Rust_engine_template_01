use crate::models::{CargoMessageLine, CompilerMessage, DiagnosticIssue, Severity};
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

#[derive(Debug, Clone)]
pub struct CargoPhaseResult {
    pub name: String,
    pub ok: bool,
    pub stdout: String,
    pub stderr: String,
}

pub fn run_cargo_check(repo_root: &Path) -> CargoPhaseResult {
    run_cargo_json(repo_root, "check", &[])
}

pub fn run_cargo_clippy(repo_root: &Path) -> CargoPhaseResult {
    run_cargo_json(repo_root, "clippy", &["--all-targets"])
}

pub fn run_cargo_test(repo_root: &Path) -> CargoPhaseResult {
    run_cargo_json(repo_root, "test", &["-p", "proc_A_dine01", "--lib", "--no-run"])
}

fn run_cargo_json(repo_root: &Path, subcommand: &str, extra: &[&str]) -> CargoPhaseResult {
    let mut cmd = Command::new("cargo");
    cmd.current_dir(repo_root);
    cmd.arg(subcommand);
    cmd.args(extra);
    cmd.args(["--message-format=json"]);
    cmd.stdout(Stdio::piped());
    cmd.stderr(Stdio::piped());

    let output = cmd.output().unwrap_or_else(|e| {
        panic!("failed to spawn cargo {subcommand}: {e}");
    });

    let stdout = String::from_utf8_lossy(&output.stdout).into_owned();
    let stderr = String::from_utf8_lossy(&output.stderr).into_owned();
    CargoPhaseResult {
        name: subcommand.to_string(),
        ok: output.status.success(),
        stdout,
        stderr,
    }
}

/// Count `warning:` lines in cargo stderr (rustc human output), per phase.
pub fn count_rustc_stderr_warnings(phases: &[CargoPhaseResult]) -> usize {
    phases
        .iter()
        .map(|p| {
            p.stderr
                .lines()
                .filter(|l| l.contains("warning:") && !l.contains("warnings emitted"))
                .count()
        })
        .sum()
}

pub fn collect_compiler_output(phases: &[CargoPhaseResult], repo_root: &Path) -> Vec<CompilerMessage> {
    let mut messages = Vec::new();
    for phase in phases {
        messages.extend(parse_cargo_json_stream(&phase.stdout));
        messages.extend(parse_rustc_stderr_lines(&phase.stderr, repo_root));
    }
    messages
}

pub fn compiler_messages_to_issues(
    messages: &[CompilerMessage],
    repo_root: &Path,
) -> Vec<DiagnosticIssue> {
    let mut issues = Vec::new();
    let mut seen = std::collections::HashSet::new();

    for msg in messages {
        if !matches!(msg.level.as_str(), "warning" | "error") {
            continue;
        }
        let Some(span) = msg.spans.first() else {
            continue;
        };
        let file = normalize_path(&span.file_name, repo_root);
        let key = format!(
            "{}:{}:{}",
            file,
            span.line_start,
            msg.message
        );
        if !seen.insert(key) {
            continue;
        }

        let severity = match msg.level.as_str() {
            "error" => Severity::Fatal,
            "warning" => Severity::Warning,
            _ => Severity::Info,
        };

        let symbol = extract_symbol(&msg.message);
        let id = format!(
            "diag-{}-{}-{}",
            msg.code.as_ref().map(|c| c.code.as_str()).unwrap_or("rustc"),
            file.replace(['/', '\\'], "_"),
            span.line_start
        );

        issues.push(DiagnosticIssue {
            id,
            subsystem: String::new(),
            file,
            line: span.line_start,
            severity,
            state: crate::models::WarningState::ActiveBug,
            symbol,
            message: msg.message.clone(),
            owner: None,
            migration_target: None,
            blockers: Vec::new(),
            related_systems: Vec::new(),
            recommended_action: String::new(),
            do_not_touch: false,
            architectural_context: Vec::new(),
            lifecycle: crate::models::SystemLifecycle::Broken,
            rustc_code: msg.code.as_ref().map(|c| c.code.clone()),
        });
    }

    issues
}

fn parse_cargo_json_stream(stdout: &str) -> Vec<CompilerMessage> {
    let mut out = Vec::new();
    for line in stdout.lines() {
        let line = line.trim();
        if line.is_empty() {
            continue;
        }
        let Ok(parsed): Result<CargoMessageLine, _> = serde_json::from_str(line) else {
            continue;
        };
        if parsed.reason.as_deref() == Some("compiler-message") {
            if let Some(message) = parsed.message {
                out.push(message);
            }
        }
    }
    out
}

fn parse_rustc_stderr_lines(stderr: &str, repo_root: &Path) -> Vec<CompilerMessage> {
    let mut out = Vec::new();
    for line in stderr.lines() {
        if !(line.contains("warning:") || line.contains("error:")) {
            continue;
        }
        let level = if line.contains("error:") {
            "error"
        } else {
            "warning"
        };
        out.push(CompilerMessage {
            code: None,
            level: level.to_string(),
            message: line.to_string(),
            spans: vec![crate::models::CompilerSpan {
                file_name: repo_root.display().to_string(),
                line_start: 0,
                column_start: 0,
            }],
        });
    }
    out
}

fn normalize_path(file: &str, repo_root: &Path) -> String {
    let p = PathBuf::from(file);
    p.strip_prefix(repo_root)
        .map(|r| r.to_string_lossy().replace('\\', "/"))
        .unwrap_or_else(|_| file.replace('\\', "/"))
}

fn extract_symbol(message: &str) -> String {
    if let Some(rest) = message.strip_prefix("function `") {
        if let Some(name) = rest.split('`').next() {
            return name.to_string();
        }
    }
    if let Some(rest) = message.strip_prefix("unused import: `") {
        return import_symbol(rest);
    }
    if let Some(rest) = message.strip_prefix("unused variable: `") {
        if let Some(name) = rest.split('`').next() {
            return name.to_string();
        }
    }
    if message.contains("more private than the item") {
        if let Some(name) = message.split('`').nth(1) {
            return name.to_string();
        }
    }
    if message.contains("is deprecated") {
        for token in message.split_whitespace() {
            if token.starts_with('`') && token.ends_with('`') {
                return token.trim_matches('`').to_string();
            }
        }
    }
    String::new()
}

fn import_symbol(rest: &str) -> String {
    let trimmed = rest.trim_end_matches('`');
    trimmed
        .split("::")
        .last()
        .unwrap_or(trimmed)
        .to_string()
}

pub fn merge_phase_stderr(phases: &[CargoPhaseResult]) -> String {
    phases
        .iter()
        .map(|p| format!("=== cargo {} ===\n{}\n{}", p.name, p.stdout, p.stderr))
        .collect::<Vec<_>>()
        .join("\n")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_compiler_message_json_line() {
        let line = r#"{"reason":"compiler-message","message":{"level":"warning","message":"unused import","spans":[{"file_name":"src/lib.rs","line_start":3}]}}"#;
        let msgs = parse_cargo_json_stream(line);
        assert_eq!(msgs.len(), 1);
        assert_eq!(msgs[0].level, "warning");
    }
}
