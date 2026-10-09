//! Plain need ranking for the growth step. No resource and no enqueue.

use super::actors::GrowthReasonCode;

const HOUSING_WEIGHT: f32 = 1.0;
const STORAGE_WEIGHT: f32 = 1.0;
const POWER_WEIGHT: f32 = 1.0;
const NEED_ZERO: f32 = 0.0;

/// One existing catalog token and one existing reason code.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct RankedTownNeed {
    pub token: &'static str,
    pub reason: GrowthReasonCode,
}

/// Housing, then storage, then power. Scores at or below zero drop out.
/// Ring threat and artillery stay with the defend arm, so they do not add a fourth score.
#[must_use]
pub fn rank_town_need(
    housing: u32,
    population: u32,
    stockpile_fill_ratio: f32,
    industrial_output_proxy: f32,
    mean_power_flow: f32,
    plant_feeds: bool,
    ring_threat: f32,
    ring_artillery: f32,
) -> Option<RankedTownNeed> {
    let _defend_reads = ring_threat.is_finite() && ring_artillery.is_finite();
    let housing_score = housing_need(housing, population) * HOUSING_WEIGHT;
    let storage_score = storage_need(stockpile_fill_ratio, industrial_output_proxy) * STORAGE_WEIGHT;
    let power_score = power_need(mean_power_flow, plant_feeds) * POWER_WEIGHT;
    let ranked = [
        (
            housing_score,
            "house",
            GrowthReasonCode::HousingDeficit,
        ),
        (
            storage_score,
            "warehouse",
            GrowthReasonCode::TransportHigh,
        ),
        (
            power_score,
            "warehouse",
            GrowthReasonCode::UtilityMissing,
        ),
    ];
    let mut best: Option<RankedTownNeed> = None;
    let mut best_score = NEED_ZERO;
    for (score, token, reason) in ranked {
        if score <= NEED_ZERO {
            continue;
        }
        if best.is_none() || score > best_score {
            best = Some(RankedTownNeed { token, reason });
            best_score = score;
        }
    }
    best
}

fn housing_need(housing: u32, population: u32) -> f32 {
    if population == 0 {
        return 0.0;
    }
    let gap = population.saturating_sub(housing) as f32 / population as f32;
    gap.clamp(0.0, 1.0)
}

fn storage_need(fill: f32, industrial_output: f32) -> f32 {
    let gap = (1.0 - fill.clamp(0.0, 1.0)).max(0.0);
    let output = industrial_output.clamp(0.0, 1.0);
    gap * (1.0 + output)
}

fn power_need(mean_power_flow: f32, plant_feeds: bool) -> f32 {
    let gap = (1.0 - mean_power_flow.clamp(0.0, 1.0)).max(0.0);
    if plant_feeds {
        gap
    } else {
        gap.max(1.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn mai_need_utility_from_stock() {
        let low_stock = rank_town_need(20, 20, 0.1, 0.8, 1.0, true, 0.0, 0.0).expect("stock");
        assert_eq!(low_stock.token, "warehouse");
        assert_eq!(low_stock.reason, GrowthReasonCode::TransportHigh);

        let short_housing =
            rank_town_need(4, 20, 1.0, 0.9, 1.0, true, 0.4, 0.4).expect("housing");
        assert_eq!(short_housing.token, "house");
        assert_eq!(short_housing.reason, GrowthReasonCode::HousingDeficit);

        assert!(rank_town_need(0, 0, 1.0, 1.0, 1.0, true, 1.0, 1.0).is_none());

        let dark = rank_town_need(20, 20, 1.0, 0.0, 1.0, false, 0.0, 0.0).expect("power");
        assert_eq!(dark.token, "warehouse");
        assert_eq!(dark.reason, GrowthReasonCode::UtilityMissing);

        let tied = rank_town_need(0, 10, 0.0, 0.0, 1.0, true, 0.0, 0.0).expect("tie");
        assert_eq!(tied.token, "house");
        assert_eq!(tied.reason, GrowthReasonCode::HousingDeficit);
    }
}
