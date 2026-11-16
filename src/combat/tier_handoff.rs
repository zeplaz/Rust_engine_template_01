//! **UM-10 / TAC-10** — multi-tier AI coordination ownership (aig3 6.4.1).
//!
//! Explicit tiers (no soft hierarchy / no cross-write):
//!
//! ```text
//! ⊚Faction decide ═▶ ⊨ orders enqueue only (AiWorld: Formation RO)
//! ⊚DeploymentOrderQueue.enqueue ═▶ ⊨order ingress
//! ⊚execute_deployment_orders_system ═▶ ⊨Formation.posture / target
//! ⊚advance_formations_system ═▶ ⊨Formation.tile
//! ⊚assign_member_slots / steer / overwatch ═▶ ⊨FormationMember only
//! ⊚MissionIntentQueue → MissionBook ═▶ ⊨sortie intents (separate from formation tile)
//! ```

#[cfg(test)]
mod tests {
    #[test]
    fn tac10() {
        // Tripwire: faction AI world must not mutably borrow Formation.
        // `AiWorld` is defined with `(Entity, &'static Formation)` — RO only.
        let faction_mod = include_str!("../ai/faction/mod.rs");
        let decide = include_str!("../ai/faction/decide.rs");
        for src in [faction_mod, decide] {
            let stripped: String = src
                .lines()
                .filter(|l| {
                    let t = l.trim_start();
                    !t.starts_with("//") && !t.starts_with("///") && !t.starts_with("//!")
                })
                .collect::<Vec<_>>()
                .join("\n");
            assert!(
                !stripped.contains("Query<&mut Formation>")
                    && !stripped.contains("Query<'_, '_, &mut Formation>")
                    && !stripped.contains("&'static mut Formation")
                    && !stripped.contains("ResMut<Formation>"),
                "TAC-10: faction decide must not Query<&mut Formation>"
            );
        }
        // Positive: RO Formation query form present in AiWorld.
        assert!(
            faction_mod.contains("&'static Formation")
                || faction_mod.contains("& Formation"),
            "expected RO Formation in AiWorld"
        );
    }
}
