//! CDR-B-VEG-RESOLVER-PARITY-001 — catalog variant_key byte parity vs engine resolver (E5 gate).

pub const VEG_RESOLVER_PARITY_LIVE_JSON: &str =
    "debug_runs/art_pipeline/veg_resolver_parity_live.json";

const CATALOG_RON: &str = "assets/configs/landscape/_vegetation_variant_catalog.ron";
const NAMING_CHARTER: &str = "src/dev/plan_veg_variant_key_naming_v1.md";
const KNOWN_KEYS_DOC: &str = "src/dev/veg_resolver_known_keys_v1.md";

const ENGINE_VEG_RESOLVER_KEYS: &[&str] = &[
    "veg_clean_day",
    "veg_old_growth",
    "veg_damaged",
    "veg_regrowth_nuclei",
    "veg_regrowth_front",
    "veg_burn_00",
    "veg_burn_01",
    "veg_burn_02",
    "veg_burn_03",
    "veg_burn_04",
    "veg_burn_05",
    "veg_burn_06",
    "veg_burn_07",
];

const ENGINE_TOPOLOGY_STAMP_KEYS: &[&str] = &[
    "topology_patch",
    "topology_corridor",
    "topology_ring",
];

#[derive(serde::Deserialize)]
struct CatalogRon {
    entries: Vec<CatalogEntryRon>,
}

#[derive(serde::Deserialize)]
struct CatalogEntryRon {
    variant_key: String,
}

#[must_use]
fn repo_path(rel: &str) -> std::path::PathBuf {
    std::env::var_os("CARGO_MANIFEST_DIR")
        .map(std::path::PathBuf::from)
        .map(|root| root.join(rel))
        .unwrap_or_else(|| std::path::PathBuf::from(rel))
}

#[must_use]
pub fn veg_resolver_catalog_parity_green() -> bool {
    veg_resolver_parity_body()
        .get("green")
        .and_then(|v| v.as_bool())
        .unwrap_or(false)
}

#[must_use]
pub fn veg_resolver_parity_body() -> serde_json::Value {
    let path = repo_path(CATALOG_RON);
    let mut catalog_veg = std::collections::BTreeSet::new();
    let mut catalog_topology = std::collections::BTreeSet::new();
    if let Ok(text) = std::fs::read_to_string(&path) {
        if let Ok(parsed) = ron::from_str::<CatalogRon>(&text) {
            for entry in parsed.entries {
                if entry.variant_key.starts_with("veg_") {
                    catalog_veg.insert(entry.variant_key);
                } else if entry.variant_key.starts_with("topology_") {
                    catalog_topology.insert(entry.variant_key);
                }
            }
        }
    }
    let engine_veg: std::collections::BTreeSet<String> = ENGINE_VEG_RESOLVER_KEYS
        .iter()
        .map(|s| (*s).to_string())
        .collect();
    let engine_topo: std::collections::BTreeSet<String> = ENGINE_TOPOLOGY_STAMP_KEYS
        .iter()
        .map(|s| (*s).to_string())
        .collect();
    let missing_in_catalog: Vec<_> = engine_veg
        .difference(&catalog_veg)
        .cloned()
        .collect();
    let extra_in_catalog: Vec<_> = catalog_veg.difference(&engine_veg).cloned().collect();
    let stamp_missing: Vec<_> = engine_topo
        .difference(&catalog_topology)
        .cloned()
        .collect();
    let clamp_green = crate::systems::ecology::catalog_clamp_witness_green();
    let green = missing_in_catalog.is_empty() && extra_in_catalog.is_empty() && stamp_missing.is_empty();
    let _ = write_veg_resolver_known_keys_doc(catalog_topology.len(), catalog_veg.len(), green);
    serde_json::json!({
        "slice_id": "CDR-B-VEG-RESOLVER-PARITY-001",
        "green": green,
        "catalog_clamp_green": clamp_green,
        "engine_veg_keys": ENGINE_VEG_RESOLVER_KEYS,
        "engine_topology_stamp_keys": ENGINE_TOPOLOGY_STAMP_KEYS,
        "catalog_veg_count": catalog_veg.len(),
        "catalog_topology_count": catalog_topology.len(),
        "missing_in_catalog": missing_in_catalog,
        "extra_in_catalog": extra_in_catalog,
        "stamp_keys_missing_in_catalog": stamp_missing,
        "byte_parity": green,
        "charter": NAMING_CHARTER,
        "catalog_path": CATALOG_RON,
        "deliverable": KNOWN_KEYS_DOC,
        "extract_topo_pattern": "veg_topo_{topology_kind_ascii_lower}",
        "extract_topo_authority": "variant_key_for_extract_row (non-burn program rows)",
        "q4_signoff": {
            "burn_overrides_topology_tint": true,
            "empty_variant_key_pre_lg5": true,
            "catalog_ron_path": CATALOG_RON,
        },
    })
}

fn write_veg_resolver_known_keys_doc(
    catalog_topology_count: usize,
    _catalog_veg_count: usize,
    parity_pass: bool,
) -> std::io::Result<()> {
    let mut lines = vec![
        "# veg_resolver_known_keys_v1 — VegetationExtractFrame authority".to_string(),
        String::new(),
        "| Field | Value |".to_string(),
        "|:---|:---|".to_string(),
        "| **Slice** | `CDR-B-VEG-RESOLVER-PARITY-001` |".to_string(),
        "| **Charter** | `src/dev/plan_veg_variant_key_naming_v1.md` |".to_string(),
        "| **Engine (burn)** | `variant_key_for_burn_row` · `src/systems/ecology/landscape_grammar_burn.rs` |".to_string(),
        "| **Engine (stamp)** | `topology_kind_to_variant_key` · `src/systems/ecology/landscape_atlas_registry.rs` |".to_string(),
        "| **Engine (extract)** | `variant_key_for_extract_row` · `src/render/extraction/vegetation_visual_extract.rs` |".to_string(),
        "| **Catalog** | `assets/configs/landscape/_vegetation_variant_catalog.ron` |".to_string(),
        format!(
            "| **Parity** | {} |",
            if parity_pass { "PASS" } else { "FAIL" }
        ),
        format!(
            "| **Witness** | `debug_runs/art_pipeline/veg_resolver_parity_live.json` |"
        ),
        String::new(),
        "## Veg resolver keys (`veg_*`)".to_string(),
        String::new(),
        "Byte-parity set — emitted by `variant_key_for_burn_row`:".to_string(),
        String::new(),
    ];
    for key in ENGINE_VEG_RESOLVER_KEYS {
        lines.push(format!("- `{key}`"));
    }
    lines.extend([
        String::new(),
        "## Topology stamp keys (`topology_*`)".to_string(),
        String::new(),
        "LG-5 stamp resolver — emitted by `topology_kind_to_variant_key`:".to_string(),
        String::new(),
    ]);
    for key in ENGINE_TOPOLOGY_STAMP_KEYS {
        lines.push(format!("- `{key}`"));
    }
    lines.extend([
        String::new(),
        "## Extract topology keys (`veg_topo_*`)".to_string(),
        String::new(),
        "Non-catalog dynamic keys from program topology when `ActiveBurn.heat <= ε`:".to_string(),
        String::new(),
        "- Pattern: `veg_topo_{topology_kind_ascii_lower}` (non-alphanumeric stripped)".to_string(),
        "- Examples: `veg_topo_patch`, `veg_topo_corridor`, `veg_topo_barrier`".to_string(),
        "- Fallback (no program): `veg_topo_patch`".to_string(),
        "- **Not** in byte-parity catalog set".to_string(),
        String::new(),
        "Witness: `debug_runs/landscape_grammar_extract_live.json` (`sprite_variant_from_program: true`).".to_string(),
        String::new(),
        "## Expanded atlas topology rows".to_string(),
        String::new(),
        "LG-5 expanded cells (`topology_*_scar`, `topology_*_burn_*`, regrowth suffixes) are catalog + tile_batch authority — not burn resolver output.".to_string(),
        String::new(),
        format!("Catalog topology row count: **{catalog_topology_count}**."),
        String::new(),
        "## Parity rule".to_string(),
        String::new(),
        "Authored catalog `veg_*` keys must match engine resolver keys **byte-for-byte** (no extras, no omissions). Stamp resolver base keys (`topology_patch`, `topology_corridor`, `topology_ring`) must exist in catalog.".to_string(),
        String::new(),
        "## Q4 sign-off (@coder B)".to_string(),
        String::new(),
        "- **Q4a:** Burn `veg_burn_*` wins over topology tint when `ActiveBurn.heat > ε`.".to_string(),
        "- **Q4b:** Empty/missing UV allowed pre-LG5 ship; parity scope is resolver-known `veg_*` only.".to_string(),
        "- **Q4c:** Ship catalog path is `assets/configs/landscape/_vegetation_variant_catalog.ron`.".to_string(),
        String::new(),
    ]);
    std::fs::write(repo_path(KNOWN_KEYS_DOC), lines.join("\n"))
}

#[must_use]
pub fn refresh_veg_resolver_parity_live_witness() -> bool {
    let body = veg_resolver_parity_body();
    let green = body.get("green").and_then(|v| v.as_bool()).unwrap_or(false);
    let mut payload = body;
    if let Some(obj) = payload.as_object_mut() {
        obj.insert(
            "deliverable_written".into(),
            serde_json::json!(KNOWN_KEYS_DOC),
        );
    }
    let wrapped = crate::dev::debug_run_envelope::wrap_debug_run(
        "CDR-B-VEG-RESOLVER-PARITY-001",
        "refresh_veg_resolver_parity_live_witness",
        VEG_RESOLVER_PARITY_LIVE_JSON,
        payload,
    );
    crate::dev::debug_run_envelope::write_debug_run_json(VEG_RESOLVER_PARITY_LIVE_JSON, wrapped)
        && green
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn veg_resolver_catalog_byte_parity_green() {
        assert!(veg_resolver_catalog_parity_green(), "{:?}", veg_resolver_parity_body());
    }

    #[test]
    fn veg_resolver_parity_live_witness_refresh_green() {
        assert!(refresh_veg_resolver_parity_live_witness());
        let doc = std::fs::read_to_string(repo_path(KNOWN_KEYS_DOC)).expect("known keys doc");
        assert!(doc.contains("CDR-B-VEG-RESOLVER-PARITY-001"));
        assert!(doc.contains("veg_topo_"));
        assert!(doc.contains("topology_patch"));
    }
}
