//! **CITY-G2-C5-001** — palette variation resolver witness.

pub use crate::construction::procedural::{
    build_city_g2_c5_001_witness_body, city_g2_c5_001_palette_witness_green,
    refresh_city_g2_c5_001_palette_witness, CITY_G2_C5_LIVE_JSON,
};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn city_g2_c5_001_live_witness_refresh_green() {
        assert!(refresh_city_g2_c5_001_palette_witness());
        let text = std::fs::read_to_string(CITY_G2_C5_LIVE_JSON).expect("witness");
        let body: serde_json::Value = serde_json::from_str(&text).expect("json");
        let green = body
            .get("green")
            .or_else(|| body.get("payload").and_then(|p| p.get("green")))
            .and_then(|v| v.as_bool());
        assert_eq!(green, Some(true));
    }
}
