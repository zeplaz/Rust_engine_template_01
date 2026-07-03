//! **CITY-G1-C4-001** — seed chain live witness refresh.

pub use crate::strategic::settlement::{
    build_city_g1_c4_001_witness_body, city_g1_c4_001_seed_chain_witness_green,
    refresh_city_g1_c4_001_seed_chain_witness, CITY_G1_C4_LIVE_JSON,
};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn city_g1_c4_001_live_witness_refresh_green() {
        assert!(refresh_city_g1_c4_001_seed_chain_witness());
        let text = std::fs::read_to_string(CITY_G1_C4_LIVE_JSON).expect("witness");
        let body: serde_json::Value = serde_json::from_str(&text).expect("json");
        let green = body
            .get("green")
            .or_else(|| body.get("payload").and_then(|p| p.get("green")))
            .and_then(|v| v.as_bool());
        assert_eq!(green, Some(true));
    }
}
