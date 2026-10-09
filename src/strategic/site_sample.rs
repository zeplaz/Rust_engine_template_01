//! One overlay cell scored through `site_score`. Nothing is stored.

use bevy::prelude::IVec2;

use super::runbook_rounds::city_planning::site_score;

/// Forward when the chunk is contested or friendly control does not lead.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SiteSampleLabel {
    Forward,
    Internal,
}

/// Scalar and label for the one cell the caller named.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct OverlaySiteSample {
    pub score: f32,
    pub label: SiteSampleLabel,
    pub friendly_minus_enemy: f32,
}

/// Scores one cell. `fire_risk` is the flood argument. `threat` is the threat argument.
/// The caller says whether this chunk is already in `FrontlineState.contested_chunks`.
#[must_use]
pub fn sample_overlay_site(
    logistics_strength: f32,
    fire_risk: f32,
    threat: f32,
    friendly_control: f32,
    enemy_control: f32,
    chunk_is_contested: bool,
) -> OverlaySiteSample {
    let friendly_minus_enemy = friendly_control - enemy_control;
    let label = if chunk_is_contested || friendly_minus_enemy <= 0.0 {
        SiteSampleLabel::Forward
    } else {
        SiteSampleLabel::Internal
    };
    OverlaySiteSample {
        score: site_score(logistics_strength, fire_risk, threat),
        label,
        friendly_minus_enemy,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::strategic::site_score;

    #[test]
    fn mai_sample_site_score() {
        let first = sample_overlay_site(0.4, 0.2, 0.1, 0.8, 0.1, false);
        let second = sample_overlay_site(0.4, 0.2, 0.1, 0.8, 0.1, false);
        assert_eq!(first.score, second.score);
        assert_eq!(first.score, site_score(0.4, 0.2, 0.1));
        assert_eq!(first.label, SiteSampleLabel::Internal);
        assert!((first.friendly_minus_enemy - 0.7).abs() < 1.0e-5);

        let contested = [IVec2::new(4, 4)];
        let chunk = IVec2::new(4, 4);
        let forward = sample_overlay_site(0.4, 0.2, 0.1, 0.8, 0.1, contested.contains(&chunk));
        assert_eq!(forward.score, site_score(0.4, 0.2, 0.1));
        assert_eq!(forward.label, SiteSampleLabel::Forward);

        let quiet = IVec2::new(1, 1);
        let internal = sample_overlay_site(0.4, 0.2, 0.1, 0.8, 0.0, contested.contains(&quiet));
        assert_eq!(internal.label, SiteSampleLabel::Internal);

        let tied = sample_overlay_site(0.4, 0.2, 0.1, 0.2, 0.2, false);
        assert!(tied.friendly_minus_enemy <= 0.0);
        assert_eq!(tied.label, SiteSampleLabel::Forward);
    }
}
