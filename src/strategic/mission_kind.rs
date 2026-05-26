//! Stage-7 mission kinds (**S7B-M1-001**) — DTO only; no dispatch solver.

use serde::{Deserialize, Serialize};

/// Signed worksheet **D-S7-03 A** — move + secure corridor intents (v1).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum MissionKind {
    MoveCorridor,
    SecureCorridor,
}

impl MissionKind {
    pub const ALL: [Self; 2] = [Self::MoveCorridor, Self::SecureCorridor];

    #[must_use]
    pub const fn as_wire_str(self) -> &'static str {
        match self {
            Self::MoveCorridor => "MoveCorridor",
            Self::SecureCorridor => "SecureCorridor",
        }
    }
}

/// Stable list for witness JSON (`mission_kinds_supported`).
#[must_use]
pub fn mission_kinds_supported() -> Vec<&'static str> {
    MissionKind::ALL.iter().map(|k| k.as_wire_str()).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn mission_kind_ron_roundtrip() {
        for kind in MissionKind::ALL {
            let ron = ron::ser::to_string(&kind).expect("serialize");
            let back: MissionKind = ron::from_str(&ron).expect("deserialize");
            assert_eq!(kind, back);
        }
    }

    #[test]
    fn mission_kinds_supported_matches_worksheet() {
        assert_eq!(
            mission_kinds_supported(),
            vec!["MoveCorridor", "SecureCorridor"]
        );
    }
}
