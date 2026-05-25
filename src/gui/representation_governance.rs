//! Convergent-growth governance — authority class + scaffold contracts for Stage 5.
//! Exit gate and agent workflow: `prompts/guides/stage5_convergence_directive_v1.md` §9–§15.

/// Whether a surface is authoritative, transitional, or legacy for representation work.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum RepresentationAuthorityClass {
    /// Real owner; downstream systems consume only.
    Authoritative,
    /// Approved temporary path with a documented migration target.
    Transitional,
    /// Superseded; no expansion allowed.
    Legacy,
}

/// Declared exit plan for a transitional scaffold (see `prompts/guides/stage5_convergence_directive_v1.md`).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ScaffoldContract {
    pub owner: &'static str,
    pub intended_replacement: &'static str,
    pub exit_condition: &'static str,
    pub removal_trigger: &'static str,
}

impl ScaffoldContract {
    #[must_use]
    pub const fn is_declared(self) -> bool {
        !self.owner.is_empty()
            && !self.intended_replacement.is_empty()
            && !self.exit_condition.is_empty()
            && !self.removal_trigger.is_empty()
    }
}

/// Runtime proofs required before Stage 5 exit (`stage5_convergence_directive_v1.md` §9).
pub const STAGE5_FULL_APP_EXIT_PROOFS: &[&str] = &[
    "one authoritative representation policy",
    "one fire extraction spine",
    "one preview authority path",
    "measurable LOD affecting GPU cost",
    "VT-4 / VT-5 agreement in FULL_APP",
    "Phase F instanced draw active",
    "no hidden parallel visual pipelines",
];

/// Mandatory closure checklist A–F (directive §13).
pub const STAGE5_MANDATORY_CLOSURES: &[&str] = &[
    "A resolver authority",
    "B VT-4 / VT-5 FULL_APP",
    "C Phase D preview authority",
    "D Phase F draw",
    "E overlay ownership",
    "F FULL_APP metrics HUD",
];

/// Fix priority when building a cycle TODO queue from FULL_APP failures (directive §12).
pub const STAGE5_FIX_PRIORITY_ORDER: &[&str] = &[
    "duplicate authority paths",
    "FULL_APP readiness failures",
    "VT mismatches",
    "Phase F proof gaps",
    "preview authority gaps",
    "resolver integration gaps",
    "metrics / HUD visibility",
];

/// Tier 1 convergence lanes — trend green each cycle (directive §5).
pub const TIER1_CONVERGENCE_LANES: &[&str] = &[
    "resolver_authority",
    "full_app_vt_readiness",
    "gpu_preview_authority",
    "phase_f_instanced_draw",
    "bq101_tile_storage_consumers",
    "docs_status_truth",
];

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn scaffold_contract_requires_all_fields() {
        let ok = ScaffoldContract {
            owner: "render/gpu_weather_fire_field",
            intended_replacement: "GPUBufferRegistry + RenderProjectionGraph",
            exit_condition: "FULL_APP readiness green without bypass",
            removal_trigger: "duplicate fire ECS scan in render prepare",
        };
        assert!(ok.is_declared());
        let bad = ScaffoldContract {
            owner: "owner",
            intended_replacement: "",
            exit_condition: "done",
            removal_trigger: "stale",
        };
        assert!(!bad.is_declared());
    }
}
