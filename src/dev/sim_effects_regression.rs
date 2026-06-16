//! Regression filter anchor: `cargo test -p proc_A_dine01 --lib sim_effects …`

#[cfg(test)]
mod tests {
    use crate::dev::sim_effect_spine_live_proof::{
        refresh_sim_effect_spine_live_witness, sim_effect_spine_live_proof_body_green,
    };
    use crate::sim::effects::sim_effect_spine_lib_witness_green;

    #[test]
    fn sim_effects_spine_lib_witness_green() {
        assert!(sim_effect_spine_lib_witness_green());
    }

    #[test]
    fn sim_effects_spine_live_proof_body_green() {
        assert!(sim_effect_spine_live_proof_body_green());
    }

    #[test]
    fn sim_effects_spine_live_witness_writes_artifacts() {
        assert!(refresh_sim_effect_spine_live_witness());
    }

    #[test]
    fn sim_effects_faction_react_hook_rows_green() {
        use crate::dev::sim_effect_spine_live_proof::sim_effect_spine_proof_state;
        let (_, _, _, faction_react) = sim_effect_spine_proof_state();
        assert!(faction_react.wired);
        assert!(faction_react.hook_rows >= 1);
    }
}
