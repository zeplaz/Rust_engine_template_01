//! **CITY-P2-001** — block LOD impostor witness.

pub use crate::strategic::settlement::{
    build_city_p2_witness_body, city_p2_witness_green, refresh_city_p2_witness, CITY_P2_LIVE_JSON,
};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn city_p2_live_witness_refresh_green() {
        assert!(refresh_city_p2_witness());
        let text = std::fs::read_to_string(CITY_P2_LIVE_JSON).expect("witness");
        let body: serde_json::Value = serde_json::from_str(&text).expect("json");
        let green = body
            .get("green")
            .or_else(|| body.get("payload").and_then(|p| p.get("green")))
            .and_then(|v| v.as_bool());
        assert_eq!(green, Some(true));
    }
}
