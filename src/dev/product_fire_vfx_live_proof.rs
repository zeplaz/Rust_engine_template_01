//! VFX-ABSENT / product fire-spark-smoke draw witness — EMIT≠DRAW instrumentation.
//!
//! Lib refresh writes an **honest pending** body (cannot claim pixels without GPU).
//! Live `--test vfx` / FULL_APP proof stamps `host_present` + `draw_ok` + `spark_rows`.

pub const PRODUCT_FIRE_VFX_LIVE_JSON: &str = "debug_runs/product_fire_vfx_live.json";

/// Honest product-lane body. Green only when particles actually raster.
#[must_use]
pub fn product_fire_vfx_witness_body(
    host_present: bool,
    spark_rows: u64,
    draw_ok: u64,
    fire_heat_overlay_on: bool,
) -> serde_json::Value {
    let particles_rendered = host_present && spark_rows > 0 && draw_ok > 0;
    let product_vfx_claim = true;
    let green = particles_rendered && fire_heat_overlay_on;
    serde_json::json!({
        "gate": "PRODUCT-FIRE-VFX-001",
        "scope": "render_particles",
        "product_vfx_claim": product_vfx_claim,
        "particles_rendered": particles_rendered,
        "green": green,
        "status": if green { "pending_operator_verify" } else { "pending_draw" },
        "rtt_core2d_overlay_host_present": host_present,
        "spark_rows": spark_rows,
        "fire_draw_ok": draw_ok,
        "fire_heat_overlay_on": fire_heat_overlay_on,
        "chrome_lod_note": "yellow/green edge = CameraFocusDebug / tile-debug — not VFX",
        "not_proof": [
            "fire_ecology_live f1_green",
            "effects_system_es7_live (Hanabi contract)",
            "steward_spark_vfx_001 lib zoom lock alone"
        ],
        "verify_cmd": "cargo run -p proc_A_dine01 --release -- --test vfx",
        "adversarial_packet": "debug_runs/vfx_absent_adversarial_packet.json",
    })
}

/// Live `--test vfx` / tactical debug flush — stamps honest GPU counters.
#[must_use]
pub fn stamp_product_fire_vfx_live_witness(
    host_present: bool,
    spark_rows: u64,
    draw_ok: u64,
    fire_heat_overlay_on: bool,
) -> bool {
    let body = product_fire_vfx_witness_body(
        host_present,
        spark_rows,
        draw_ok,
        fire_heat_overlay_on,
    );
    let wrapped = crate::dev::debug_run_envelope::wrap_debug_run(
        "PRODUCT-FIRE-VFX-001",
        "stamp_product_fire_vfx_live_witness",
        PRODUCT_FIRE_VFX_LIVE_JSON,
        body,
    );
    crate::dev::debug_run_envelope::write_debug_run_json(PRODUCT_FIRE_VFX_LIVE_JSON, wrapped)
}

/// Lib refresh: host expected from spawn policy; draw_ok unknown → not green.
#[must_use]
pub fn refresh_product_fire_vfx_pending_witness() -> bool {
    let host_expected = crate::gui::rtt_core2d_overlay_hosts_enabled();
    let body = product_fire_vfx_witness_body(host_expected, 0, 0, false);
    // Force honest pending — lib cannot claim pixels.
    let mut body = body;
    if let Some(obj) = body.as_object_mut() {
        obj.insert("green".into(), serde_json::json!(false));
        obj.insert("particles_rendered".into(), serde_json::json!(false));
        obj.insert("status".into(), serde_json::json!("pending_live_vfx"));
        obj.insert(
            "lib_note".into(),
            serde_json::json!("lib refresh stamps instrumentation schema only — run --test vfx for green"),
        );
    }
    let wrapped = crate::dev::debug_run_envelope::wrap_debug_run(
        "PRODUCT-FIRE-VFX-001",
        "refresh_product_fire_vfx_pending_witness",
        PRODUCT_FIRE_VFX_LIVE_JSON,
        body,
    );
    crate::dev::debug_run_envelope::write_debug_run_json(PRODUCT_FIRE_VFX_LIVE_JSON, wrapped)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn product_vfx_green_requires_host_rows_and_draw_ok() {
        assert!(!product_fire_vfx_witness_body(false, 12, 5, true)["green"]
            .as_bool()
            .unwrap());
        assert!(!product_fire_vfx_witness_body(true, 0, 5, true)["green"]
            .as_bool()
            .unwrap());
        assert!(!product_fire_vfx_witness_body(true, 12, 0, true)["green"]
            .as_bool()
            .unwrap());
        assert!(product_fire_vfx_witness_body(true, 12, 5, true)["green"]
            .as_bool()
            .unwrap());
    }

    #[test]
    fn pending_lib_refresh_never_greens() {
        assert!(refresh_product_fire_vfx_pending_witness());
        let text = std::fs::read_to_string(PRODUCT_FIRE_VFX_LIVE_JSON).unwrap();
        let v: serde_json::Value = serde_json::from_str(&text).unwrap();
        assert_eq!(v["green"], false);
        assert_eq!(v["particles_rendered"], false);
        assert_eq!(v["product_vfx_claim"], true);
    }

    #[test]
    fn live_stamp_body_matches_green_contract() {
        let v = product_fire_vfx_witness_body(true, 48, 12, true);
        assert_eq!(v["green"], true);
        assert_eq!(v["particles_rendered"], true);
        assert_eq!(v["status"], "pending_operator_verify");
        assert_eq!(v["spark_rows"], 48);
        assert_eq!(v["fire_draw_ok"], 12);
    }
}
