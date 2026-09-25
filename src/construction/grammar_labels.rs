//! Human labels for building grammar ids — mirrors `grammar_labels_v1.json` / APS `human_label()`.

use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use std::sync::{Mutex, OnceLock};

const GRAMMAR_LABELS_JSON: &str = "assets/configs/buildings/grammars/grammar_labels_v1.json";

#[derive(Debug, Clone, Default)]
struct GrammarLabelsFile {
    archetypes: HashMap<String, String>,
    massing: HashMap<String, String>,
    district_styles: HashMap<String, String>,
    age: HashMap<String, String>,
}

fn repo_asset_path(rel: &str) -> PathBuf {
    std::env::var_os("CARGO_MANIFEST_DIR")
        .map(PathBuf::from)
        .map(|root| root.join(rel))
        .unwrap_or_else(|| PathBuf::from(rel))
}

fn load_labels_file() -> GrammarLabelsFile {
    let path = repo_asset_path(GRAMMAR_LABELS_JSON);
    let Ok(raw) = fs::read_to_string(&path) else {
        return GrammarLabelsFile::default();
    };
    let Ok(doc) = serde_json::from_str::<serde_json::Value>(&raw) else {
        return GrammarLabelsFile::default();
    };
    let mut out = GrammarLabelsFile::default();
    if let Some(obj) = doc.get("archetypes").and_then(|v| v.as_object()) {
        for (k, v) in obj {
            if let Some(label) = v.get("label").and_then(|l| l.as_str()) {
                out.archetypes.insert(k.clone(), label.to_string());
            }
        }
    }
    for (section, dest) in [
        ("massing", &mut out.massing),
        ("district_styles", &mut out.district_styles),
        ("age", &mut out.age),
    ] {
        if let Some(obj) = doc.get(section).and_then(|v| v.as_object()) {
            for (k, v) in obj {
                if let Some(label) = v.as_str() {
                    dest.insert(k.clone(), label.to_string());
                }
            }
        }
    }
    out
}

fn labels_cache() -> &'static Mutex<GrammarLabelsFile> {
    static CACHE: OnceLock<Mutex<GrammarLabelsFile>> = OnceLock::new();
    CACHE.get_or_init(|| Mutex::new(load_labels_file()))
}

fn lookup(map: &HashMap<String, String>, id: &str) -> Option<String> {
    map.get(id).cloned()
}

fn title_case_fallback(id: &str) -> String {
    id.split('_')
        .map(|part| {
            let mut c = part.chars();
            match c.next() {
                None => String::new(),
                Some(f) => {
                    let mut s = f.to_uppercase().to_string();
                    s.push_str(c.as_str());
                    s
                }
            }
        })
        .collect::<Vec<_>>()
        .join(" ")
}

/// Player-facing label for grammar archetype id (PascalCase catalog id → lowercase phrase).
#[must_use]
pub fn human_archetype_label(id: &str) -> String {
    let cache = labels_cache().lock().expect("grammar labels");
    lookup(&cache.archetypes, id)
        .map(|s| s.to_ascii_lowercase())
        .unwrap_or_else(|| title_case_fallback(id).to_ascii_lowercase())
}

#[must_use]
pub fn human_district_label(id: &str) -> String {
    let cache = labels_cache().lock().expect("grammar labels");
    lookup(&cache.district_styles, id).unwrap_or_else(|| title_case_fallback(id))
}

#[must_use]
pub fn human_massing_label(id: &str) -> String {
    let cache = labels_cache().lock().expect("grammar labels");
    lookup(&cache.massing, id).unwrap_or_else(|| title_case_fallback(id))
}

#[must_use]
pub fn human_age_label(id: &str) -> String {
    let cache = labels_cache().lock().expect("grammar labels");
    lookup(&cache.age, id).unwrap_or_else(|| title_case_fallback(id))
}

#[must_use]
pub fn grammar_labels_loaded_green() -> bool {
    let cache = labels_cache().lock().expect("grammar labels");
    cache.archetypes.len() >= 3
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn g1_archetype_labels_present() {
        assert!(grammar_labels_loaded_green());
        // Drop the cache guard before human_* lookups. `std::sync::Mutex` is not
        // reentrant; holding it across those calls deadlocks the lib harness.
        let (factory_key, warehouse_key) = {
            let cache = labels_cache().lock().expect("grammar labels");
            let factory_key = cache
                .archetypes
                .keys()
                .find(|k| k.contains("Factory"))
                .cloned()
                .expect("factory archetype label");
            let warehouse_key = cache
                .archetypes
                .keys()
                .find(|k| k.contains("Warehouse") || k.contains("Industrial"))
                .cloned()
                .expect("warehouse archetype label");
            (factory_key, warehouse_key)
        };
        assert_eq!(human_archetype_label(&factory_key), "factory cluster");
        assert_eq!(human_massing_label("long_hall"), "Long Hall");
        let warehouse_label = human_archetype_label(&warehouse_key);
        assert!(!warehouse_label.contains(&warehouse_key));
    }

    /// True when a `labels_cache().lock()` guard is still in scope at a call that locks again.
    ///
    /// `std::sync::Mutex` deadlocks on same-thread reentry, which hung `cargo test --lib`.
    fn lock_scope_relocks(src: &str) -> bool {
        let code = strip_comments_and_strings(src);
        let bytes = code.as_bytes();
        let mut depth = 0i32;
        let mut guards: Vec<i32> = Vec::new();
        let mut i = 0usize;
        while i < bytes.len() {
            match bytes[i] {
                b'{' => {
                    depth += 1;
                    i += 1;
                }
                b'}' => {
                    depth -= 1;
                    guards.retain(|&d| depth >= d);
                    i += 1;
                }
                _ => {
                    if starts_at(&code, i, "labels_cache().lock()") {
                        if guards.iter().any(|&d| depth >= d) {
                            return true;
                        }
                        guards.push(depth);
                        i += "labels_cache().lock()".len();
                        continue;
                    }
                    const RELOCK: &[&str] = &[
                        "human_archetype_label",
                        "human_district_label",
                        "human_massing_label",
                        "human_age_label",
                        "grammar_labels_loaded_green",
                        "labels_cache",
                    ];
                    if let Some(hit) = RELOCK.iter().find(|name| ident_at(&code, i, name)) {
                        if guards.iter().any(|&d| depth >= d) {
                            return true;
                        }
                        i += hit.len();
                        continue;
                    }
                    i += 1;
                }
            }
        }
        false
    }

    fn ident_at(code: &str, i: usize, name: &str) -> bool {
        if !code[i..].starts_with(name) {
            return false;
        }
        let before_ok = i == 0
            || {
                let prev = code.as_bytes()[i - 1];
                !prev.is_ascii_alphanumeric() && prev != b'_'
            };
        let end = i + name.len();
        let after_ok = end >= code.len()
            || {
                let next = code.as_bytes()[end];
                !next.is_ascii_alphanumeric() && next != b'_'
            };
        before_ok && after_ok
    }

    fn starts_at(code: &str, i: usize, needle: &str) -> bool {
        code[i..].starts_with(needle)
    }

    fn strip_comments_and_strings(src: &str) -> String {
        let b = src.as_bytes();
        let mut out = String::with_capacity(src.len());
        let mut i = 0usize;
        while i < b.len() {
            if b[i] == b'/' && i + 1 < b.len() && b[i + 1] == b'/' {
                while i < b.len() && b[i] != b'\n' {
                    i += 1;
                }
                continue;
            }
            if b[i] == b'r' {
                if let Some(end) = skip_raw_string(b, i) {
                    out.push(' ');
                    i = end;
                    continue;
                }
            }
            if b[i] == b'"' {
                i += 1;
                while i < b.len() && b[i] != b'"' {
                    if b[i] == b'\\' {
                        i = (i + 2).min(b.len());
                        continue;
                    }
                    i += 1;
                }
                if i < b.len() {
                    i += 1;
                }
                out.push(' ');
                continue;
            }
            out.push(b[i] as char);
            i += 1;
        }
        out
    }

    fn skip_raw_string(b: &[u8], i: usize) -> Option<usize> {
        let mut j = i + 1;
        let mut hashes = 0usize;
        while j < b.len() && b[j] == b'#' {
            hashes += 1;
            j += 1;
        }
        if j >= b.len() || b[j] != b'"' {
            return None;
        }
        j += 1;
        while j < b.len() {
            if b[j] == b'"' {
                let mut k = 0usize;
                while k < hashes && j + 1 + k < b.len() && b[j + 1 + k] == b'#' {
                    k += 1;
                }
                if k == hashes {
                    return Some(j + 1 + hashes);
                }
            }
            j += 1;
        }
        Some(b.len())
    }

    #[test]
    fn grammar_label_cache_must_not_relock() {
        let historical = r#"
            fn g1_archetype_labels_present() {
                let cache = labels_cache().lock().expect("grammar labels");
                let factory_key = cache.archetypes.keys().next().cloned().unwrap();
                assert_eq!(human_archetype_label(&factory_key), "factory cluster");
            }
        "#;
        assert!(
            lock_scope_relocks(historical),
            "detector must flag the CI deadlock shape"
        );
        let live = include_str!("grammar_labels.rs");
        assert!(
            !lock_scope_relocks(live),
            "grammar label cache re-locked: a lookup runs while the Mutex guard is live"
        );

        let (tx, rx) = std::sync::mpsc::channel();
        let worker = std::thread::spawn(move || {
            let key = {
                let cache = labels_cache().lock().expect("grammar labels");
                cache.archetypes.keys().next().cloned()
            };
            if let Some(key) = key {
                let _ = human_archetype_label(&key);
            }
            let _ = human_massing_label("long_hall");
            let _ = human_district_label("core");
            let _ = human_age_label("new");
            assert!(
                labels_cache().try_lock().is_ok(),
                "grammar label cache still held after lookup"
            );
            tx.send(()).expect("send");
        });
        match rx.recv_timeout(std::time::Duration::from_secs(2)) {
            Ok(()) => worker.join().expect("grammar label lookup thread"),
            Err(_) => panic!("grammar label cache re-locked (same-thread Mutex deadlock)"),
        }
    }
}
