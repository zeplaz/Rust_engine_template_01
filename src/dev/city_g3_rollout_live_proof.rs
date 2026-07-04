//! **CITY-G3-ROLLOUT-001** — BlockRecipe plugin rollout witness.

pub use crate::strategic::settlement::{
    build_city_g3_rollout_witness_body, city_g3_rollout_witness_green,
    refresh_city_g3_rollout_witness, CITY_G3_ROLLOUT_LIVE_JSON,
};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn city_g3_rollout_live_witness_refresh_green() {
        assert!(refresh_city_g3_rollout_witness());
        let text = std::fs::read_to_string(CITY_G3_ROLLOUT_LIVE_JSON).expect("witness");
        let body: serde_json::Value = serde_json::from_str(&text).expect("json");
        let green = body
            .get("green")
            .or_else(|| body.get("payload").and_then(|p| p.get("green")))
            .and_then(|v| v.as_bool());
        assert_eq!(green, Some(true));
    }
}
