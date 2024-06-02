fn dynamic_sanction_effect_system(
    mut sanctions: Query<(&mut Sanction, &AgentEmotionalState, &EconomicRelation, &AgentDecisionFactors)>,
    economic_data: Res<EconomicData>,
) {
    for (mut sanction, emotional_state, economic_relation, decision_factors) in sanctions.iter_mut() {
        adjust_sanction_severity_based_on_economic_state(&mut sanction, &economic_data);
        adjust_sanction_based_on_agent_behavior(&mut sanction, emotional_state, economic_relation, decision_factors);
    }
}

fn adjust_sanction_severity_based_on_economic_state(
    sanction: &mut Sanction,
    economic_data: &EconomicData,
) {
    // Logic to adjust sanction severity based on global and agent-specific economic data
    // Example: Modify sanction effects based on GDP, growth rate, resource availability, etc.
    // ...
}

fn adjust_sanction_based_on_agent_behavior(
    sanction: &mut Sanction,
    emotional_state: &AgentEmotionalState,
    economic_relation: &EconomicRelation,
    decision_factors: &AgentDecisionFactors,
) {
    // Logic to modify sanctions based on the agent's emotional state, economic relations, and decision-making factors
    // Example: Increase severity for agents showing high aggression or low diplomatic cooperation
    // ...
}