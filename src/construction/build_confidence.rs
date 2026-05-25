//! Placement confidence gradient (not land/housing value).

use crate::strategic::SitePlacementValidation;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BuildConfidence {
    Perfect,
    Good,
    Risky,
    Invalid,
}

impl BuildConfidence {
    #[must_use]
    pub const fn label(self) -> &'static str {
        match self {
            Self::Perfect => "perfect",
            Self::Good => "good",
            Self::Risky => "risky",
            Self::Invalid => "invalid",
        }
    }
}

#[must_use]
pub fn confidence_from_validation(report: &SitePlacementValidation) -> BuildConfidence {
    if report.allows_commit && report.errors.is_empty() {
        return BuildConfidence::Perfect;
    }
    if report.allows_commit {
        return BuildConfidence::Good;
    }
    let risky = report.errors.iter().any(|e| {
        let lower = e.to_ascii_lowercase();
        lower.contains("overlap") || lower.contains("access") || lower.contains("terrain")
    });
    if risky {
        BuildConfidence::Risky
    } else {
        BuildConfidence::Invalid
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::strategic::SitePlacementValidation;

    #[test]
    fn perfect_when_commit_with_no_errors() {
        let r = SitePlacementValidation {
            allows_commit: true,
            errors: vec![],
            ..Default::default()
        };
        assert_eq!(confidence_from_validation(&r), BuildConfidence::Perfect);
    }
}
