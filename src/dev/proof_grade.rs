//! **G-PROOF-01** — witness proof grades (PLAY-TRUTH-002).
//!
//! `VisualCapture` (`--test visual` / `full_capture_active`) must not use witness shortcuts,
//! `patch_*_witness*`, or `qualified_close` as a green substitute.

/// How a witness JSON row was produced.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum ProofGrade {
    /// Lib-only fixture writers (`refresh_*_live_witness`, projection fixtures).
    LibFixture,
    /// Headless sim / harness without full visual capture proof commit.
    HeadlessSim,
    /// Live `--test visual` proof commit (`full_capture_active`).
    VisualCapture,
}

impl ProofGrade {
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::LibFixture => "lib_fixture",
            Self::HeadlessSim => "headless_sim",
            Self::VisualCapture => "visual_capture",
        }
    }

    /// **G-PROOF-01:** atomic LOG-* shortcuts and `patch_*_witness_for_play_proof` are lib/harness only.
    #[must_use]
    pub const fn allows_witness_shortcuts(self) -> bool {
        !matches!(self, Self::VisualCapture)
    }

    /// **G-PROOF-01:** `qualified_close` may close LOG-E01 on lib fixture only.
    #[must_use]
    pub const fn allows_qualified_close_green(self) -> bool {
        matches!(self, Self::LibFixture)
    }

    #[must_use]
    pub const fn from_full_capture_active(full_capture_active: bool) -> Self {
        if full_capture_active {
            Self::VisualCapture
        } else {
            Self::HeadlessSim
        }
    }
}
