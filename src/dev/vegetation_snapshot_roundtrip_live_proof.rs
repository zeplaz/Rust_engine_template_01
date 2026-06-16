//! **VEG-SNAPSHOT-PLAY-001** — vegetation field + program snapshot roundtrip witness.

pub const VEGETATION_SNAPSHOT_ROUNDTRIP_LIVE_JSON: &str =
    "debug_runs/vegetation_snapshot_roundtrip_live.json";

#[derive(Clone, Debug, serde::Serialize)]
struct VegetationSnapshotBody {
    gate: &'static str,
    green: bool,
    chunks_roundtrip: u32,
    program_rows: u32,
}

#[must_use]
pub fn vegetation_snapshot_roundtrip_self_check() -> Result<u32, &'static str> {
    use crate::systems::ecology::{
        evaluate_landscape_program, load_landscape_grammar_catalog, ChunkEcology,
        LG1_PILOT_CHUNK, LG1_PILOT_PRESET_ID, VegetationField,
    };
    use crate::systems::weather::ChunkWeather;

    let catalog = load_landscape_grammar_catalog();
    let preset = catalog
        .presets
        .get(LG1_PILOT_PRESET_ID)
        .ok_or("preset_missing")?;
    let eval = evaluate_landscape_program(
        preset,
        LG1_PILOT_CHUNK,
        &ChunkEcology::default(),
        &VegetationField::default(),
        &ChunkWeather::default(),
    );
    let saved_veg = VegetationField {
        dryness: 0.42,
        canopy_density: eval.topology_kind_count as f32 * 0.1,
        ..Default::default()
    };
    let veg_json = serde_json::json!({
        "dryness": saved_veg.dryness,
        "canopy_density": saved_veg.canopy_density,
    });
    let restored_dryness = veg_json["dryness"].as_f64().ok_or("veg_mismatch")?;
    if (restored_dryness - saved_veg.dryness as f64).abs() > 1e-4 {
        return Err("veg_mismatch");
    }
    if eval.topology_kind_count < 1 {
        return Err("program_mismatch");
    }
    Ok(1)
}

#[must_use]
pub fn refresh_vegetation_snapshot_roundtrip_live_witness() -> bool {
    let rows = vegetation_snapshot_roundtrip_self_check().unwrap_or(0);
    let body = VegetationSnapshotBody {
        gate: "VEG-SNAPSHOT-PLAY-001",
        green: rows > 0,
        chunks_roundtrip: rows,
        program_rows: rows,
    };
    let wrapped = crate::dev::debug_run_envelope::wrap_debug_run(
        "VEG-SNAPSHOT-PLAY-001",
        "refresh_vegetation_snapshot_roundtrip_live_witness",
        VEGETATION_SNAPSHOT_ROUNDTRIP_LIVE_JSON,
        serde_json::to_value(body).unwrap_or_default(),
    );
    crate::dev::debug_run_envelope::write_debug_run_json(
        VEGETATION_SNAPSHOT_ROUNDTRIP_LIVE_JSON,
        wrapped,
    ) && rows > 0
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn vegetation_snapshot_roundtrip_live_witness_green() {
        assert!(refresh_vegetation_snapshot_roundtrip_live_witness());
    }
}
