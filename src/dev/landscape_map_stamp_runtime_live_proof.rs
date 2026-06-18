//! **VEG-LG5-STAMP-RUNTIME-001** — runtime LG-5 chunk UV stamp witness.

pub const LANDSCAPE_MAP_STAMP_RUNTIME_LIVE_JSON: &str =
    "debug_runs/landscape_map_stamp_runtime_live.json";

#[must_use]
pub fn landscape_map_stamp_runtime_stamped_chunks() -> usize {
    use bevy::prelude::IVec2;

    use crate::gui::landscape_chunk_atlas_stamp::stamp_request_for_topology;
    use crate::systems::ecology::{landscape_lg5_registry_stamped, load_landscape_atlas_registry};

    if !landscape_lg5_registry_stamped() {
        return 0;
    }
    let registry = load_landscape_atlas_registry();
    if !registry.load_errors.is_empty() {
        return 0;
    }
    ["Patch", "Corridor", "Ring"]
        .iter()
        .enumerate()
        .filter_map(|(i, kind)| {
            stamp_request_for_topology(
                &registry,
                IVec2::new(i as i32, 0),
                &[(*kind).to_string()],
            )
        })
        .count()
}

#[must_use]
pub fn landscape_map_stamp_runtime_witness_green() -> bool {
    landscape_map_stamp_runtime_stamped_chunks() >= 1
}

#[must_use]
pub fn refresh_landscape_map_stamp_runtime_live_witness() -> bool {
    let stamped_chunks = landscape_map_stamp_runtime_stamped_chunks();
    let green = stamped_chunks >= 1;
    let body = serde_json::json!({
        "gate": "VEG-LG5-STAMP-RUNTIME-001",
        "slice_id": "VEG-LG5-STAMP-RUNTIME-001",
        "green": green,
        "stamped_chunks": stamped_chunks,
        "index_path": "assets/configs/landscape/_landscape_atlas_index.ron",
        "variant_key_source": "topology_kind_to_variant_key | veg_topo_* extract",
        "engine_authority": "landscape_chunk_atlas_stamp",
        "parent_contract": "debug_runs/landscape_map_stamp_contract_live.json",
    });
    let wrapped = crate::dev::debug_run_envelope::wrap_debug_run(
        "VEG-LG5-STAMP-RUNTIME-001",
        "refresh_landscape_map_stamp_runtime_live_witness",
        LANDSCAPE_MAP_STAMP_RUNTIME_LIVE_JSON,
        body,
    );
    crate::dev::debug_run_envelope::write_debug_run_json(LANDSCAPE_MAP_STAMP_RUNTIME_LIVE_JSON, wrapped)
        && green
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn landscape_map_stamp_runtime_live_witness_green() {
        if !crate::systems::ecology::landscape_lg5_registry_stamped() {
            eprintln!("skip: landscape lg5 index not stamped");
            return;
        }
        assert!(refresh_landscape_map_stamp_runtime_live_witness());
    }
}
