//! Bevy simulation-grade authority scan (static): who mutates key resources.

use regex::Regex;
use std::fs;
use std::path::Path;
use walkdir::WalkDir;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct AuthorityWriteSite {
    pub file: String,
    pub line: usize,
    pub symbol_hint: String,
    pub resource: String,
}

/// Canonical sole writer domains — extra `ResMut` sites are flagged, not auto-failed.
const RESOLVED_VIEWPORTS_CANONICAL: &[&str] = &[
    "src/render/viewport_pipeline.rs",
    // VM-C2: authority → legacy read cache (not UI measure).
    "src/render/view_runtime/commit.rs",
];

const VIEW_MANAGER_CANONICAL: &[&str] = &["src/gui/view_authority.rs"];

pub fn scan_authority_writes(src_root: &Path) -> Vec<AuthorityWriteSite> {
    let mut out = Vec::new();
    let patterns: Vec<(&str, Regex)> = [
        (
            "ResolvedViewports",
            Regex::new(r"ResMut\s*<\s*ResolvedViewports\s*>").expect("regex"),
        ),
        (
            "ViewManager",
            Regex::new(r"ResMut\s*<\s*ViewManager\s*>").expect("regex"),
        ),
    ]
    .to_vec();

    for entry in WalkDir::new(src_root)
        .into_iter()
        .filter_map(Result::ok)
        .filter(|e| e.path().extension().and_then(|s| s.to_str()) == Some("rs"))
    {
        let path = entry.path();
        let Ok(content) = fs::read_to_string(path) else {
            continue;
        };
        let rel = path
            .strip_prefix(src_root.parent().unwrap_or(src_root))
            .unwrap_or(path)
            .to_string_lossy()
            .replace('\\', "/");

        for (line_no, line) in content.lines().enumerate() {
            for (resource, re) in &patterns {
                if re.is_match(line) {
                    let hint = line.trim().chars().take(120).collect();
                    out.push(AuthorityWriteSite {
                        file: rel.clone(),
                        line: line_no + 1,
                        symbol_hint: hint,
                        resource: (*resource).to_string(),
                    });
                }
            }
        }
    }

    out
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct AuthorityAlert {
    pub resource: String,
    pub severity: String,
    pub message: String,
    pub sites: Vec<AuthorityWriteSite>,
}

pub fn authority_alerts(sites: &[AuthorityWriteSite]) -> Vec<AuthorityAlert> {
    let mut alerts = Vec::new();
    for resource in ["ResolvedViewports", "ViewManager"] {
        let writers: Vec<_> = sites
            .iter()
            .filter(|s| s.resource == resource)
            .cloned()
            .collect();
        if writers.is_empty() {
            continue;
        }
        let canonical = match resource {
            "ResolvedViewports" => RESOLVED_VIEWPORTS_CANONICAL,
            "ViewManager" => VIEW_MANAGER_CANONICAL,
            _ => &[],
        };
        let non_canonical: Vec<_> = writers
            .iter()
            .filter(|w| !canonical.iter().any(|c| w.file == *c))
            .cloned()
            .collect();
        if !non_canonical.is_empty() {
            alerts.push(AuthorityAlert {
                resource: resource.to_string(),
                severity: if resource == "ResolvedViewports" {
                    "MED".into()
                } else {
                    "LOW".into()
                },
                message: format!(
                    "{} ResMut write site(s) outside canonical path(s): {:?}",
                    non_canonical.len(),
                    canonical
                ),
                sites: non_canonical,
            });
        }
    }
    alerts
}
