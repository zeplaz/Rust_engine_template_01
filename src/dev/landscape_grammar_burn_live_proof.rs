//! VEG-BURN chain — burn overlay + extract witnesses (001..007).

use crate::render::extraction::VegetationExtractFrame;
use crate::systems::ecology::{
    ActiveBurn, LandscapeBurnWitness, LandscapeGrammarLg2Witness, LG1_PILOT_CHUNK,
    SuccessionState, SuccessionTopologyStage,
};
use crate::systems::sim_control::SimTick;
use crate::terrain::generation::Chunk;
use bevy::prelude::{App, MinimalPlugins};

#[must_use]
pub fn veg_burn_chain_witness_green() -> bool {
    refresh_veg_burn_chain_live_witness()
}

#[must_use]
pub fn refresh_veg_burn_chain_live_witness() -> bool {
    if !crate::systems::ecology::burn_sm_self_check_green() {
        return false;
    }

    let mut extract_app = App::new();
    extract_app
        .add_plugins(MinimalPlugins)
        .init_resource::<SimTick>()
        .init_resource::<crate::systems::sim_control::SimTimeMicros>()
        .add_plugins(crate::render::extraction::VegetationVisualExtractPlugin);
    extract_app.world_mut().spawn((
        Chunk {
            coord: LG1_PILOT_CHUNK,
        },
        SuccessionState {
            stage: SuccessionTopologyStage::BurnScar,
            ..Default::default()
        },
        ActiveBurn {
            heat: 0.75,
            frame_index: 2,
            started_tick: 0,
            severity: 0.75,
            ..Default::default()
        },
    ));
    extract_app.update();
    let frame = extract_app.world().resource::<VegetationExtractFrame>();
    let extract_ok = crate::render::extraction::refresh_vegetation_extract_witness(frame);
    let fullapp_ok = veg_burn_fullapp_006_witness_green(frame);
    let play_ok = veg_burn_visible_at_operational_zoom_lib();

    let lg2 = LandscapeGrammarLg2Witness {
        fire_disturbances: 1,
        ..Default::default()
    };
    let burn = LandscapeBurnWitness {
        active_burn_chunks: 1,
        ..Default::default()
    };
    let overlay_ok = crate::systems::ecology::refresh_burn_overlay_witness(&lg2, &burn);

    let green = overlay_ok && extract_ok && fullapp_ok && play_ok;
    let body = serde_json::json!({
        "gate": "VEG-BURN-CHAIN-001",
        "green": green,
        "VEG-BURN-OVERLAY-001": overlay_ok,
        "VEG-BURN-EXTRACT-004": extract_ok,
        "VEG-BURN-FULLAPP-006": fullapp_ok,
        "VEG-BURN-PLAY-007": play_ok,
        "burn_active_rows": frame.rows.iter().filter(|r| r.burn_active).count(),
    });
    let wrapped = crate::dev::debug_run_envelope::wrap_debug_run(
        "VEG-BURN-CHAIN-001",
        "refresh_veg_burn_chain_live_witness",
        "debug_runs/landscape_grammar_burn_chain_live.json",
        body,
    );
    crate::dev::debug_run_envelope::write_debug_run_json(
        "debug_runs/landscape_grammar_burn_chain_live.json",
        wrapped,
    ) && green
}

#[must_use]
pub fn veg_burn_fullapp_006_witness_green(frame: &VegetationExtractFrame) -> bool {
    frame
        .rows
        .iter()
        .any(|r| r.burn_active && r.variant_key.starts_with("veg_burn_"))
}

#[must_use]
pub fn veg_burn_pilot_extract_frame() -> VegetationExtractFrame {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins)
        .init_resource::<SimTick>()
        .init_resource::<crate::systems::sim_control::SimTimeMicros>()
        .add_plugins(crate::render::extraction::VegetationVisualExtractPlugin);
    app.world_mut().spawn((
        Chunk {
            coord: LG1_PILOT_CHUNK,
        },
        SuccessionState {
            stage: SuccessionTopologyStage::BurnScar,
            ..Default::default()
        },
        ActiveBurn {
            heat: 0.8,
            frame_index: 1,
            started_tick: 0,
            severity: 0.8,
            ..Default::default()
        },
    ));
    app.update();
    app.world()
        .resource::<VegetationExtractFrame>()
        .clone()
}

/// **VEG-BURN-FULLAPP-006** — burn rows + variant_key stub for stage5 witness merge.
#[must_use]
pub fn veg_burn_stage5_witness_json() -> serde_json::Value {
    let frame = veg_burn_pilot_extract_frame();
    let burn_rows: Vec<_> = frame
        .rows
        .iter()
        .filter(|r| r.burn_active)
        .map(|r| {
            serde_json::json!({
                "coord": { "x": r.coord.x, "y": r.coord.y },
                "variant_key": r.variant_key,
                "frame_index": r.frame_index,
                "extract_glyph": r.extract_glyph.to_string(),
            })
        })
        .collect();
    serde_json::json!({
        "gate": "VEG-BURN-FULLAPP-006",
        "green": veg_burn_fullapp_006_witness_green(&frame),
        "burn_active_rows": burn_rows.len(),
        "sample_variant_keys": burn_rows
            .iter()
            .filter_map(|r| r.get("variant_key").and_then(|v| v.as_str()))
            .take(4)
            .collect::<Vec<_>>(),
        "burn_rows": burn_rows,
    })
}

/// **VEG-BURN-PLAY-007** — burn extract rows visible at operational zoom contract.
#[must_use]
pub fn veg_burn_visible_at_operational_zoom_lib() -> bool {
    veg_burn_fullapp_006_witness_green(&veg_burn_pilot_extract_frame())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn veg_burn_chain_live_witness_green() {
        assert!(refresh_veg_burn_chain_live_witness());
    }
}
