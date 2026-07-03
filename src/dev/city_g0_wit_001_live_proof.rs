//! **CITY-G0-WIT-001** — grammar determinism witness (G0c): fixed seed ⇒ stable `AssemblySnapshot` hash.

pub use crate::construction::procedural::{
    build_city_g0_wit_001_witness_body, city_g0_wit_001_determinism_witness_green,
    refresh_city_g0_wit_001_grammar_determinism_witness, CITY_G0_WIT_LIVE_JSON,
};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn city_g0_wit_001_live_witness_refresh_green() {
        assert!(refresh_city_g0_wit_001_grammar_determinism_witness());
        let text = std::fs::read_to_string(CITY_G0_WIT_LIVE_JSON).expect("witness file");
        let body: serde_json::Value = serde_json::from_str(&text).expect("json");
        let green = body
            .get("green")
            .or_else(|| body.get("payload").and_then(|p| p.get("green")))
            .and_then(|v| v.as_bool());
        assert_eq!(green, Some(true));
        let gate = body
            .get("gate")
            .or_else(|| body.get("payload").and_then(|p| p.get("gate")))
            .and_then(|v| v.as_str());
        assert_eq!(gate, Some("CITY-G0-WIT-001"));
    }
}
