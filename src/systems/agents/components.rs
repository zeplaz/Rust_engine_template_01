
#[derive(Component)]
struct AgentEmotionalState {
    stress_level: f32,       // Overall level of stress, affects decision making
    confidence_level: f32,   // Confidence in their decisions and actions
    urgency: f32,            // Perception of how urgent situations are
    trust: f32,              // Willingness to rely on others
    fear: f32,               // Fear of potential threats or outcomes
    satisfaction: f32,       // Satisfaction with current state and progress
    aggression: f32,         // Tendency to engage in conflict
    openness: f32,           // Openness to new experiences and alliances
    adaptability: f32,       // Ability to adapt strategies based on circumstances
    assertiveness: f32,      // Tendency to push for their own agenda
    empathy: f32,            // Ability to understand and share feelings of others
}

#[derive(Component)]
struct AgentPerception {
    perceived_emotional_states: HashMap<Entity, AgentEmotionalState>,
}



#[derive(Component)]
struct AgentStrategicAttributes {
    economic_strength: f32,
    logistic_capability: f32,
    frontline_efficiency: f32,
    diplomatic_influence: f32,
    technological_advancement: f32,  // Represents the level of technological development
    intelligence_network: f32,       // Efficiency and extent of espionage and information gathering
    social_stability: f32,           // Stability and contentment within the agent's territory
    // ... potentially more attributes ...
}


#[derive(Component)]
struct DecisionFactors {
    resource_availability: f32,
    strategic_value: f32,
    threat_level: f32,
    // ... other factors influencing decisions
}

#[derive(Component)]
struct AgentRelationships {
    interaction_history: HashMap<Entity, InteractionHistory>,
    political_stance: HashMap<Entity, PoliticalStance>,
    economic_relations: HashMap<Entity, EconomicRelation>,
    // ... additional factors
}