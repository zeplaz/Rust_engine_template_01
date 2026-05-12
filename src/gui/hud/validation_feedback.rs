//! L1 failure explanation — map raw validation tokens to operational language.
//!
//! See `prompts/guides/operational_feedback_language_v1.md`.

use crate::strategic::{SiteArchetype, SitePlacementValidation};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ValidationSeverity {
    Info,
    Warning,
    Error,
}

#[derive(Clone, Debug)]
pub struct ValidationDiagnostic {
    pub severity: ValidationSeverity,
    pub message: String,
}

/// Human-readable diagnostics for ghost placement / commit gating.
pub fn diagnostics_from_site_validation(v: &SitePlacementValidation) -> Vec<ValidationDiagnostic> {
    let mut out = Vec::new();
    for e in &v.errors {
        out.push(ValidationDiagnostic {
            severity: ValidationSeverity::Error,
            message: map_error_token(e),
        });
    }
    for w in &v.warnings {
        out.push(ValidationDiagnostic {
            severity: ValidationSeverity::Warning,
            message: map_warning_token(w),
        });
    }
    if out.is_empty() && v.allows_commit {
        out.push(ValidationDiagnostic {
            severity: ValidationSeverity::Info,
            message: "Placement checks pass — commit when ready.".into(),
        });
    }
    out
}

/// First blocking reason, or strongest warning, for one-line HUD context.
pub fn primary_validation_message(v: &SitePlacementValidation) -> Option<String> {
    let diags = diagnostics_from_site_validation(v);
    diags
        .iter()
        .find(|d| d.severity == ValidationSeverity::Error)
        .map(|d| d.message.clone())
        .or_else(|| {
            diags
                .iter()
                .find(|d| d.severity == ValidationSeverity::Warning)
                .map(|d| d.message.clone())
        })
        .or_else(|| {
            if v.errors.is_empty() && v.warnings.is_empty() && !v.allows_commit {
                Some("Placement blocked — see terrain & logistics overlays.".into())
            } else {
                None
            }
        })
}

#[inline]
pub fn site_archetype_operational_name(a: SiteArchetype) -> &'static str {
    match a {
        SiteArchetype::CivilHousing => "Civil housing district",
        SiteArchetype::Factory => "Industrial plant",
        SiteArchetype::PowerPlant => "Power complex",
        SiteArchetype::RailDepot => "Rail / logistics hub",
        SiteArchetype::MilitaryBase => "Military base",
        SiteArchetype::RadarSite => "Radar site",
        SiteArchetype::SensorPost => "Sensor post",
        SiteArchetype::TrenchLine => "Trench line",
        SiteArchetype::BunkerComplex => "Bunker complex",
        SiteArchetype::FuelDepot => "Fuel depot",
        SiteArchetype::WaterPlant => "Water / utilities plant",
    }
}

fn map_error_token(raw: &str) -> String {
    match raw {
        "terrain" => "Terrain not viable — check slope, water, or geology for this structure.".into(),
        "network_access" => {
            "No logistics access from this tile — extend corridors or pick a better-connected site.".into()
        }
        x if x.starts_with("out_") => format!("Operational constraint: {}", x.replace('_', " ")),
        _ => format!("Cannot place: {}", raw.replace('_', " ")),
    }
}

fn map_warning_token(raw: &str) -> String {
    match raw {
        "sparse_logistics_reach" => {
            "Logistics reach is weak — site may be hard to supply once built.".into()
        }
        _ => format!("Note: {}", raw.replace('_', " ")),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn terrain_error_maps_to_operational_copy() {
        let v = SitePlacementValidation {
            valid: false,
            allows_commit: false,
            errors: vec!["terrain".into()],
            ..Default::default()
        };
        let d = diagnostics_from_site_validation(&v);
        assert!(d.iter().any(|x| x.severity == ValidationSeverity::Error));
        assert!(
            d[0].message.contains("Terrain"),
            "{}",
            d[0].message
        );
    }

    #[test]
    fn sparse_logistics_emits_warning_diagnostic() {
        let v = SitePlacementValidation {
            valid: true,
            allows_commit: true,
            warnings: vec!["sparse_logistics_reach".into()],
            ..Default::default()
        };
        let d = diagnostics_from_site_validation(&v);
        assert!(
            d.iter().any(|x| x.message.contains("Logistics reach")),
            "{d:?}"
        );
    }
}
