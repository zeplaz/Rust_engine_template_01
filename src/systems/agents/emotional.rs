
use bevy::prelude;

use components::AgentEmotionalState;

const EMPATHY_ACCURACY_FACTOR: f32 = 0.5;



fn empathy_understanding_system(
    agents: Query<(Entity, &AgentEmotionalState)>,
    mut perceptions: Query<(Entity, &mut AgentPerception)>,
) {
    for (agent_id, agent_state) in agents.iter() {
        if let Ok((_, mut perception)) = perceptions.get_mut(agent_id) {
            for (other_agent_id, perceived_state) in perception.perceived_emotional_states.iter_mut() {
                if let Ok(other_agent_real_state) = agents.get(*other_agent_id) {
                    update_perceived_emotional_states(agent_state, perceived_state, other_agent_real_state);
                }
            }
        }
    }
}

fn update_perceived_emotional_states(
    agent_state: &AgentEmotionalState,
    perceived_state: &mut AgentEmotionalState,
    other_agent_real_state: &AgentEmotionalState,
) {
    let accuracy = calculate_perception_accuracy(agent_state.empathy);
    adjust_perception_based_on_empathy(perceived_state, other_agent_real_state, accuracy);
}

fn calculate_perception_accuracy(empathy: f32) -> f32 {
    // Logic to determine the accuracy of perception based on empathy
    // High empathy results in higher accuracy
    empathy * EMPATHY_ACCURACY_FACTOR
}

fn adjust_perception_based_on_empathy(
    perceived_state: &mut AgentEmotionalState,
    real_state: &AgentEmotionalState,
    accuracy: f32,
) {
    // Adjust each emotional attribute based on the accuracy factor
    perceived_state.stress_level = blend(real_state.stress_level, perceived_state.stress_level, accuracy);
    perceived_state.confidence_level = blend(real_state.confidence_level, perceived_state.confidence_level, accuracy);
    perceived_state.urgency = blend(real_state.urgency, perceived_state.urgency, accuracy);
    perceived_state.trust = blend(real_state.trust, perceived_state.trust, accuracy);
    perceived_state.fear = blend(real_state.fear, perceived_state.fear, accuracy);
    perceived_state.satisfaction = blend(real_state.satisfaction, perceived_state.satisfaction, accuracy);
    perceived_state.aggression = blend(real_state.aggression, perceived_state.aggression, accuracy);
    perceived_state.openness = blend(real_state.openness, perceived_state.openness, accuracy);
    perceived_state.adaptability = blend(real_state.adaptability, perceived_state.adaptability, accuracy);
    perceived_state.assertiveness = blend(real_state.assertiveness, perceived_state.assertiveness, accuracy);
    perceived_state.empathy = blend(real_state.empathy, perceived_state.empathy, accuracy);

}

fn blend(real_value: f32, perceived_value: f32, accuracy: f32) -> f32 {
    // Blends the real and perceived values based on the accuracy (empathy level)
    perceived_value * (1.0 - accuracy) + real_value * accuracy
}