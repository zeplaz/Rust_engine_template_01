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

/// Player-facing label for grammar archetype id (`IndustrialWarehouse` → `Industrial warehouse` per design).
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
    cache.archetypes.contains_key("IndustrialWarehouse")
        && cache.archetypes.contains_key("FactoryCluster")
        && cache.archetypes.contains_key("RailEdge")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn g1_archetype_labels_present() {
        assert!(grammar_labels_loaded_green());
        assert_eq!(
            human_archetype_label("FactoryCluster"),
            "factory cluster"
        );
        assert_eq!(human_massing_label("long_hall"), "Long Hall");
        assert!(!human_archetype_label("IndustrialWarehouse").contains("IndustrialWarehouse"));
    }
}
