//! **CITY-P1-001** — block static scene witness.

pub use crate::strategic::settlement::{
    build_city_p1_witness_body, city_p1_witness_green, refresh_city_p1_witness, CITY_P1_LIVE_JSON,
};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn city_p1_live_witness_refresh_green() {
        assert!(refresh_city_p1_witness());
        let text = std::fs::read_to_string(CITY_P1_LIVE_JSON).expect("witness");
        let body: serde_json::Value = serde_json::from_str(&text).expect("json");
        let green = body
            .get("green")
            .or_else(|| body.get("payload").and_then(|p| p.get("green")))
            .and_then(|v| v.as_bool());
        assert_eq!(green, Some(true));
    }
}
