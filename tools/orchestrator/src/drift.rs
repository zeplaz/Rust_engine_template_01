use crate::models::{DiagnosticIssue, OrchestratorSnapshot};
use crate::state::OrchestratorPaths;
use std::collections::{BTreeMap, HashMap};
use std::fs;
use std::io::Write;

pub fn write_drift_summary(paths: &OrchestratorPaths, current: &OrchestratorSnapshot) {
    let prev = load_previous_snapshot(paths, &current.meta.run_id);
    let content = drift_markdown(current, prev.as_ref());
    let out = paths.reports.join("drift_summary.md");
    if let Some(parent) = out.parent() {
        let _ = fs::create_dir_all(parent);
    }
    if let Ok(mut f) = fs::File::create(&out) {
        let _ = f.write_all(content.as_bytes());
    }
}

fn load_previous_snapshot(paths: &OrchestratorPaths, current_run_id: &str) -> Option<OrchestratorSnapshot> {
    let mut entries: Vec<_> = fs::read_dir(&paths.history)
        .ok()?
        .filter_map(Result::ok)
        .filter(|e| e.path().extension().and_then(|s| s.to_str()) == Some("json"))
        .collect();
    entries.sort_by_key(|e| e.file_name());
    for entry in entries.into_iter().rev() {
        let name = entry.file_name().to_string_lossy().to_string();
        if name.contains(current_run_id) {
            continue;
        }
        if let Ok(text) = fs::read_to_string(entry.path()) {
            if let Ok(snap) = serde_json::from_str::<OrchestratorSnapshot>(&text) {
                return Some(snap);
            }
        }
    }
    None
}

fn issue_key(i: &DiagnosticIssue) -> String {
    format!("{}:{}:{}", i.file, i.line, i.message)
}

fn drift_markdown(current: &OrchestratorSnapshot, previous: Option<&OrchestratorSnapshot>) -> String {
    let mut out = String::from("# Drift summary\n\n");
    out.push_str(&format!("- **Current run:** `{}`\n", current.meta.run_id));
    let Some(prev) = previous else {
        out.push_str("- **Previous:** _none_\n");
        return out;
    };
    out.push_str(&format!("- **Previous run:** `{}`\n\n", prev.meta.run_id));

    let cur_map: HashMap<_, _> = current
        .issues
        .iter()
        .map(|i| (issue_key(i), i))
        .collect();
    let prev_map: HashMap<_, _> = prev
        .issues
        .iter()
        .map(|i| (issue_key(i), i))
        .collect();

    let mut resolved = Vec::new();
    let mut new = Vec::new();
    let mut changed: BTreeMap<String, (String, String)> = BTreeMap::new();

    for (k, p) in &prev_map {
        if !cur_map.contains_key(k) {
            resolved.push(k.clone());
        } else if let Some(c) = cur_map.get(k) {
            let ps = format!("{:?}", p.state);
            let cs = format!("{:?}", c.state);
            if ps != cs {
                changed.insert(k.clone(), (ps, cs));
            }
        }
    }
    for k in cur_map.keys() {
        if !prev_map.contains_key(k) {
            new.push(k.clone());
        }
    }

    out.push_str(&format!("| Metric | Count |\n|--------|------:|\n"));
    out.push_str(&format!("| Resolved | {} |\n", resolved.len()));
    out.push_str(&format!("| New | {} |\n", new.len()));
    out.push_str(&format!("| State changed | {} |\n\n", changed.len()));

    if !resolved.is_empty() {
        out.push_str("## Resolved\n\n");
        for k in resolved.iter().take(40) {
            out.push_str(&format!("- `{k}`\n"));
        }
        out.push('\n');
    }
    if !new.is_empty() {
        out.push_str("## New\n\n");
        for k in new.iter().take(40) {
            out.push_str(&format!("- `{k}`\n"));
        }
        out.push('\n');
    }
    if !changed.is_empty() {
        out.push_str("## State changes\n\n");
        for (k, (from, to)) in &changed {
            out.push_str(&format!("- `{k}`: {from} → {to}\n"));
        }
    }
    out
}
