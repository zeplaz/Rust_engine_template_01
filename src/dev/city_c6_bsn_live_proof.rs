//! **CITY-C6-BSN-001** — BSN street furniture witness.

pub use crate::strategic::settlement::{
    build_city_c6_bsn_witness_body, city_c6_bsn_witness_green, refresh_city_c6_bsn_witness,
    CITY_C6_BSN_LIVE_JSON,
};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn city_c6_bsn_live_witness_refresh_green() {
        assert!(refresh_city_c6_bsn_witness());
        let text = std::fs::read_to_string(CITY_C6_BSN_LIVE_JSON).expect("witness");
        let body: serde_json::Value = serde_json::from_str(&text).expect("json");
        let green = body
            .get("green")
            .or_else(|| body.get("payload").and_then(|p| p.get("green")))
            .and_then(|v| v.as_bool());
        assert_eq!(green, Some(true));
    }
}
