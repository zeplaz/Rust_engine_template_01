use crate::models::{SemanticMarker, SourceAnnotation};
use regex::Regex;
use std::fs;
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

/// Word-boundary patterns — avoids matching `temperature` for `TEMP`.
const MARKER_PATTERNS: &[(&str, &str)] = &[
    ("TODO", r"\bTODO\b"),
    ("FIXME", r"\bFIXME\b"),
    ("HACK", r"\bHACK\b"),
    ("TEMP", r"(//\s*TEMP\b|@TEMP\b|\bTEMP:)"),
    ("MIGRATION", r"\bMIGRATION\b"),
    ("DEPRECATED", r"#\[deprecated"),
    ("REMOVE_AFTER", r"\bREMOVE_AFTER\b"),
    ("WORKAROUND", r"\bWORKAROUND\b"),
    ("VIEWPORT_AUTHORITY", r"VIEWPORT_AUTHORITY"),
];

pub fn scan_source_tree(src_root: &Path) -> (Vec<SemanticMarker>, Vec<SourceAnnotation>) {
    let mut markers = Vec::new();
    let mut annotations = Vec::new();

    let marker_res: Vec<(&str, Regex)> = MARKER_PATTERNS
        .iter()
        .map(|(kind, pat)| (*kind, Regex::new(pat).expect("marker regex")))
        .collect();

    let orchestrator_re = Regex::new(
        r"@orchestrator-(status|owner|do-not-cleanup)\s+([^\n\r]+)",
    )
    .expect("regex");

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
            scan_line_markers(&rel, line_no + 1, line, &marker_res, &mut markers);
            scan_orchestrator_annotations(
                &rel,
                line_no + 1,
                line,
                &orchestrator_re,
                &mut annotations,
            );
        }
    }

    (markers, annotations)
}

fn scan_line_markers(
    file: &str,
    line: usize,
    text: &str,
    patterns: &[(&str, Regex)],
    out: &mut Vec<SemanticMarker>,
) {
    for (kind, re) in patterns {
        if re.is_match(text) {
            out.push(SemanticMarker {
                file: file.to_string(),
                line,
                kind: (*kind).to_string(),
                text: text.trim().to_string(),
            });
        }
    }
}

fn scan_orchestrator_annotations(
    file: &str,
    line: usize,
    text: &str,
    re: &Regex,
    out: &mut Vec<SourceAnnotation>,
) {
    if !text.contains("@orchestrator") {
        return;
    }

    let mut status = String::new();
    let mut owner = None;
    let mut do_not_cleanup = false;
    let mut note = None;

    for cap in re.captures_iter(text) {
        match cap.get(1).map(|m| m.as_str()) {
            Some("status") => status = cap.get(2).map(|m| m.as_str().trim().to_string()).unwrap_or_default(),
            Some("owner") => owner = cap.get(2).map(|m| m.as_str().trim().to_string()),
            Some("do-not-cleanup") => do_not_cleanup = true,
            _ => {}
        }
    }

    if text.contains("@orchestrator-note") {
        note = text
            .split("@orchestrator-note")
            .nth(1)
            .map(|s| s.trim().trim_start_matches(':').trim().to_string());
    }

    if status.is_empty() && owner.is_none() && !do_not_cleanup {
        return;
    }

    out.push(SourceAnnotation {
        file: file.to_string(),
        line,
        status: if status.is_empty() {
            "IN_PROGRESS".into()
        } else {
            status
        },
        owner,
        do_not_cleanup,
        note,
    });
}

pub fn scan_deprecated_symbols(crate_src: &Path) -> Vec<(String, String, usize)> {
    let mut found = Vec::new();
    let dep_re = Regex::new(r#"#\[deprecated\([^\]]*note\s*=\s*"([^"]*)""#).unwrap();
    let fn_re = Regex::new(r"pub fn (\w+)").unwrap();

    for entry in WalkDir::new(crate_src)
        .into_iter()
        .filter_map(Result::ok)
        .filter(|e| e.path().extension().and_then(|s| s.to_str()) == Some("rs"))
    {
        let path = entry.path();
        let Ok(content) = fs::read_to_string(path) else {
            continue;
        };
        let lines: Vec<&str> = content.lines().collect();
        for (i, line) in lines.iter().enumerate() {
            if !line.contains("#[deprecated") {
                continue;
            }
            let note = dep_re
                .captures(line)
                .and_then(|c| c.get(1))
                .map(|m| m.as_str().to_string())
                .unwrap_or_default();
            let sym = lines
                .get(i + 1)
                .and_then(|l| fn_re.captures(l))
                .and_then(|c| c.get(1))
                .map(|m| m.as_str().to_string())
                .unwrap_or_default();
            if !sym.is_empty() {
                let rel = path
                    .strip_prefix(crate_src.parent().unwrap_or(crate_src))
                    .unwrap_or(path)
                    .to_string_lossy()
                    .replace('\\', "/");
                found.push((sym, note, i + 1));
                let _ = rel;
            }
        }
    }
    found
}

pub fn repo_src_root(repo_root: &Path) -> PathBuf {
    repo_root.join("src")
}
