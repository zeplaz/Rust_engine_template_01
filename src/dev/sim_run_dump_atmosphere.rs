//! `atmosphere` block of the sim run dump (counts only, no verdicts).
//!
//! Plan: `src/dev/plan_shared_atmosphere_weather_fire_v1.md` §6. Wired keys are read from
//! [`AtmosphereDiagnostics`] (sampled from `AtmosphereClipmapStack`); keys owned by later waves
//! are emitted as `0` and listed under `stubs` so a zero is never mistaken for a measurement.
//! `l0_advect_writers` is the number of distinct systems that advected L0 (reviewer expects ≤ 1).

use serde_json::{json, Value};

use crate::systems::atmosphere::{AtmosphereDiagnostics, AtmosphereFrame};

/// §6 counters not yet measured. Emitted as `0`; removed from this list as each wave wires them.
/// W2–W5 wired earlier. W6 wired: `gpu_field_upload_paths`, `presentation_frame_revision`.
pub const ATMOSPHERE_DUMP_STUBS: &[&str] = &[];

/// `atmosphere` block (`samples: 0` when atmosphere diagnostics are absent).
#[must_use]
pub fn atmosphere_json(
    diag: Option<&AtmosphereDiagnostics>,
    frame: Option<&AtmosphereFrame>,
) -> Value {
    let mut out = serde_json::Map::new();
    out.insert("samples".into(), json!(if diag.is_some() { 1 } else { 0 }));

    if let Some(d) = diag {
        out.insert("l0_advect_writers".into(), json!(d.l0_advect_writers));
        out.insert("l0_revision".into(), json!(d.l0_revision));
        out.insert("l0_smoke_max".into(), json!(d.l0_smoke_max));
        out.insert("l0_smoke_mass".into(), json!(d.l0_smoke_mass));
        out.insert("advect_runs".into(), json!(d.advect_runs));
        out.insert("field_fill_runs".into(), json!(d.field_fill_runs));
        out.insert("smoke_cloud_overlap_cells".into(), json!(d.smoke_cloud_overlap_cells));
        out.insert("firestorm_plumes".into(), json!(d.firestorm_plumes));
        out.insert("plume_updraft_peak_mps".into(), json!(d.plume_updraft_peak_mps));
        out.insert("pyro_moisture_injected".into(), json!(d.pyro_moisture_injected));
        out.insert("pyro_cloud_delta".into(), json!(d.pyro_cloud_delta));
        out.insert("rain_washout_mass".into(), json!(d.rain_washout_mass));
        out.insert("gpu_field_upload_paths".into(), json!(d.gpu_field_upload_paths));
        out.insert(
            "presentation_frame_revision".into(),
            json!(d.presentation_frame_revision),
        );
        out.insert(
            "second_writer_violations".into(),
            json!(d.second_writer_violations),
        );
    } else {
        out.insert("l0_advect_writers".into(), json!(0));
        out.insert("l0_revision".into(), json!(0));
        out.insert("l0_smoke_max".into(), json!(0.0));
        out.insert("l0_smoke_mass".into(), json!(0.0));
        out.insert("advect_runs".into(), json!(0));
        out.insert("field_fill_runs".into(), json!(0));
        out.insert("smoke_cloud_overlap_cells".into(), json!(0));
        out.insert("firestorm_plumes".into(), json!(0));
        out.insert("plume_updraft_peak_mps".into(), json!(0.0));
        out.insert("pyro_moisture_injected".into(), json!(0.0));
        out.insert("pyro_cloud_delta".into(), json!(0.0));
        out.insert("rain_washout_mass".into(), json!(0.0));
        out.insert("gpu_field_upload_paths".into(), json!(0));
        out.insert("presentation_frame_revision".into(), json!(0));
        out.insert("second_writer_violations".into(), json!(0));
    }

    if let Some(f) = frame {
        let st = f.stats();
        out.insert("frame_revision".into(), json!(f.revision()));
        out.insert("frame_builds".into(), json!(st.builds));
        out.insert(
            "legacy_weather_direct_reads".into(),
            json!(st.legacy_weather_direct_reads),
        );
        out.insert("vis_weather_min_m".into(), json!(st.vis_weather_min_m));
        out.insert("vis_combined_min_m".into(), json!(st.vis_combined_min_m));
        out.insert("vis_smoke_delta_m".into(), json!(st.vis_smoke_delta_max_m));
        out.insert("icing_risk_max".into(), json!(st.icing_risk_max));
        out.insert(
            "icing_grounded_samples".into(),
            json!(st.icing_grounded_samples),
        );
        out.insert(
            "storm_developing_cells".into(),
            json!(st.storm_developing_cells),
        );
        out.insert("storm_thunder_cells".into(), json!(st.storm_thunder_cells));
        out.insert(
            "hurricane_class_cells".into(),
            json!(st.storm_hurricane_cells),
        );
        // Prefer live frame revision for presentation parity when diag has not synced yet.
        if out.get("presentation_frame_revision").and_then(|v| v.as_u64()) == Some(0)
            && f.revision() > 0
        {
            out.insert("presentation_frame_revision".into(), json!(f.revision()));
        }
    } else {
        // Wired keys are measured (not stubs) even when the frame resource is absent this sample.
        out.insert("frame_revision".into(), json!(0));
        out.insert("frame_builds".into(), json!(0));
        out.insert("legacy_weather_direct_reads".into(), json!(0));
        out.insert("vis_weather_min_m".into(), json!(0.0));
        out.insert("vis_combined_min_m".into(), json!(0.0));
        out.insert("vis_smoke_delta_m".into(), json!(0.0));
        out.insert("icing_risk_max".into(), json!(0.0));
        out.insert("icing_grounded_samples".into(), json!(0));
        out.insert("storm_developing_cells".into(), json!(0));
        out.insert("storm_thunder_cells".into(), json!(0));
        out.insert("hurricane_class_cells".into(), json!(0));
    }

    let mut stubs: Vec<&str> = ATMOSPHERE_DUMP_STUBS
        .iter()
        .copied()
        .filter(|key| !out.contains_key(*key))
        .collect();
    stubs.sort_unstable();
    for key in &stubs {
        out.insert((*key).into(), json!(0));
    }
    out.insert("stubs".into(), json!(stubs));
    Value::Object(out)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn absent_diagnostics_yield_zero_samples_and_empty_stubs() {
        let v = atmosphere_json(None, None);
        assert_eq!(v["samples"], 0);
        assert!(v["stubs"].as_array().is_some_and(|a| a.is_empty()));
        assert_eq!(v["gpu_field_upload_paths"], 0);
        assert_eq!(v["presentation_frame_revision"], 0);
        assert_eq!(v["second_writer_violations"], 0);
    }

    #[test]
    fn block_documents_l0_advect_writers_and_w6_keys() {
        let d = AtmosphereDiagnostics {
            l0_advect_writers: 1,
            l0_revision: 7,
            gpu_field_upload_paths: 1,
            presentation_frame_revision: 7,
            second_writer_violations: 0,
            ..Default::default()
        };
        let v = atmosphere_json(Some(&d), None);
        assert_eq!(v["l0_advect_writers"], 1);
        assert_eq!(v["l0_revision"], 7);
        assert_eq!(v["gpu_field_upload_paths"], 1);
        assert_eq!(v["presentation_frame_revision"], 7);
        assert_eq!(v["second_writer_violations"], 0);
        assert!(v["stubs"].as_array().unwrap().is_empty());
        assert!(v.get("green").is_none());
    }

    #[test]
    fn frame_stats_merge_into_block() {
        let mut frame = AtmosphereFrame::default();
        frame.rebuild_for_tests_with_smoke(0, 0.4);
        let v = atmosphere_json(None, Some(&frame));
        assert!(v["frame_builds"].as_u64().unwrap_or(0) > 0);
        assert!(v.get("frame_revision").is_some());
        assert_eq!(v["legacy_weather_direct_reads"], 0);
        assert!(v["vis_smoke_delta_m"].as_f64().unwrap_or(0.0) > 0.0);
        assert!(v.get("icing_risk_max").is_some());
        assert!(v.get("icing_grounded_samples").is_some());
        assert!(v.get("storm_developing_cells").is_some());
        assert!(v.get("storm_thunder_cells").is_some());
        let stubs = v["stubs"].as_array().unwrap();
        for wired in [
            "frame_builds",
            "frame_revision",
            "legacy_weather_direct_reads",
            "vis_weather_min_m",
            "vis_combined_min_m",
            "vis_smoke_delta_m",
            "icing_risk_max",
            "icing_grounded_samples",
            "storm_developing_cells",
            "storm_thunder_cells",
            "hurricane_class_cells",
            "smoke_cloud_overlap_cells",
            "firestorm_plumes",
            "plume_updraft_peak_mps",
            "pyro_moisture_injected",
            "pyro_cloud_delta",
            "rain_washout_mass",
            "gpu_field_upload_paths",
            "presentation_frame_revision",
        ] {
            assert!(
                !stubs.iter().any(|k| k.as_str() == Some(wired)),
                "{wired} must not remain a stub"
            );
        }
        assert_eq!(v["presentation_frame_revision"], frame.revision());
    }

    #[test]
    fn w4_diag_counters_appear_in_block() {
        let d = AtmosphereDiagnostics {
            firestorm_plumes: 2,
            pyro_cloud_delta: 0.4,
            pyro_moisture_injected: 0.2,
            rain_washout_mass: 0.15,
            smoke_cloud_overlap_cells: 3,
            plume_updraft_peak_mps: 9.0,
            ..Default::default()
        };
        let v = atmosphere_json(Some(&d), None);
        assert_eq!(v["firestorm_plumes"], 2);
        assert!((v["pyro_cloud_delta"].as_f64().unwrap() - 0.4).abs() < 1e-6);
        assert!((v["rain_washout_mass"].as_f64().unwrap() - 0.15).abs() < 1e-6);
        assert_eq!(v["smoke_cloud_overlap_cells"], 3);
    }

    #[test]
    fn presentation_revision_matches_sim_frame_when_diag_synced() {
        let mut frame = AtmosphereFrame::default();
        frame.rebuild_for_tests_with_smoke(1, 0.2);
        let d = AtmosphereDiagnostics {
            gpu_field_upload_paths: 1,
            presentation_frame_revision: frame.revision(),
            ..Default::default()
        };
        let v = atmosphere_json(Some(&d), Some(&frame));
        assert_eq!(v["gpu_field_upload_paths"], 1);
        assert_eq!(v["presentation_frame_revision"], v["frame_revision"]);
    }
}
