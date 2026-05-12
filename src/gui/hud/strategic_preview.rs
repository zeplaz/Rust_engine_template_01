//! L3 **strategic preview** — “if you commit, what changes?” copy from placement scores (interpreted).

use crate::strategic::SitePlacementValidation;

/// Operational sentences derived from ghost validation scores (no raw scalars).
pub fn format_projected_commit_effects(report: &SitePlacementValidation) -> Option<String> {
    if !report.valid {
        return None;
    }
    let mut parts: Vec<&'static str> = Vec::new();
    if report.logistics_score >= 0.65 {
        parts.push("improves logistics anchoring for this tile");
    } else if report.logistics_score <= 0.35 {
        parts.push("leaves the site supply-fragile unless corridors catch up");
    }
    if report.terrain_score <= 0.35 {
        parts.push("terrain or hydrology stress — expect higher remedial work");
    }
    if report.strategic_score <= 0.45 {
        parts.push("sits in a more contested / visible band of the theater");
    } else if report.strategic_score >= 0.75 {
        parts.push("fits a comparatively defensible / low-visibility pocket");
    }
    if parts.is_empty() {
        return None;
    }
    Some(format!("If committed: {}.", parts.join("; ")))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn low_logistics_emits_fragile_copy() {
        let r = SitePlacementValidation {
            valid: true,
            allows_commit: true,
            logistics_score: 0.2,
            terrain_score: 0.8,
            strategic_score: 0.6,
            ..Default::default()
        };
        let s = format_projected_commit_effects(&r).expect("copy");
        assert!(s.contains("fragile"), "{s}");
    }
}
