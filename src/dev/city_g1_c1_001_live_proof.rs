//! **CITY-G1-C1-001** — BlockArchetype threshold witness.

pub use crate::strategic::settlement::{
    build_city_g1_c1_001_witness_body, city_g1_c1_001_block_archetype_witness_green,
    refresh_city_g1_c1_001_block_archetype_witness, CITY_G1_C1_LIVE_JSON,
};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn city_g1_c1_001_live_witness_refresh_green() {
        assert!(refresh_city_g1_c1_001_block_archetype_witness());
        let text = std::fs::read_to_string(CITY_G1_C1_LIVE_JSON).expect("witness");
        let body: serde_json::Value = serde_json::from_str(&text).expect("json");
        let green = body
            .get("green")
            .or_else(|| body.get("payload").and_then(|p| p.get("green")))
            .and_then(|v| v.as_bool());
        assert_eq!(green, Some(true));
    }
}
