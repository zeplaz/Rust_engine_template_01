//! **Layer 1 — raw entities** (`base_behav_a` scaffold): agent / faction / membership link.
//!
//! - **Simulation-native agents** ([`AgentMode::Free`]) vs **narrative-biased** ([`AgentMode::Scripted`] / [`AgentMode::Hybrid`]).
//! - Runtime identity is the ECS [`Entity`]. Stored `id` fields are kept in sync for graph tooling and saves.

use bevy::prelude::*;

use super::hybrid_brain::{HybridAgentEmotions, HybridAgentTraits};

// --- Agent kernel ---

/// Cognitive scratch — focus, salience; expand for memory / theory-of-mind later.
#[derive(Clone, Copy, Debug)]
pub struct CognitiveState {
    pub focus: f32,
    /// Mission / narrative pressure coupling (objectives raise salience elsewhere).
    pub narrative_salience: f32,
}

impl Default for CognitiveState {
    fn default() -> Self {
        Self {
            focus: 0.5,
            narrative_salience: 0.0,
        }
    }
}

/// Emergent vs authorable control (authority injects **pressure**, never direct ECS mutations for outcomes).
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum AgentMode {
    /// Fully ECS + traits / fuzzy; no mission bias.
    Free,
    /// Mission-driven; [`super::behavior_script::ScriptInfluence`] expected most ticks.
    Scripted,
    /// Blend simulation traits with script bias: `script_weight` ∈ [0, 1].
    Hybrid { script_weight: f32 },
}

impl Default for AgentMode {
    fn default() -> Self {
        Self::Free
    }
}

#[derive(Component, Clone, Debug)]
pub struct Agent {
    pub id: Entity,
    pub traits: HybridAgentTraits,
    pub emotional_state: HybridAgentEmotions,
    pub cognition: CognitiveState,
    pub mode: AgentMode,
}

/// Internal cohesion lifecycle (stages 1–3 before optional split entity).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum FactionInternalStage {
    #[default]
    Unified,
    Divergence,
    Autonomy,
    /// Ready for topology change (new faction entity — handled by fracture hook, not immediate).
    Split,
}

/// Faction = pressure aggregator + resource / ideology / control — **not** a simple team label.
#[derive(Component, Clone, Debug)]
pub struct Faction {
    pub id: Entity,
    pub cohesion: f32,
    pub ideology: Vec<f32>,
    pub resources: f32,
    /// Power blocks (oligarchs, military clusters, …) — optional ECS links.
    pub internal_blocks: Vec<Entity>,
    pub control_strength: f32,
    pub internal_stage: FactionInternalStage,
    pub sub_factions: Vec<Entity>,
}

impl Default for Faction {
    fn default() -> Self {
        Self {
            id: Entity::PLACEHOLDER,
            cohesion: 0.75,
            ideology: Vec::new(),
            resources: 100.0,
            internal_blocks: Vec::new(),
            control_strength: 0.7,
            internal_stage: FactionInternalStage::Unified,
            sub_factions: Vec::new(),
        }
    }
}

/// **Critical link** — attach on the **agent** entity; `faction` points at the faction entity.
#[derive(Component, Clone, Debug)]
pub struct AgentFactionLink {
    pub agent: Entity,
    pub faction: Entity,
    pub loyalty: f32,
    pub influence: f32,
    pub autonomy: f32,
}

/// Keeps `Agent.id` / `Faction.id` / `AgentFactionLink.agent` aligned with owning [`Entity`].
pub fn behavior_sync_entity_ids_system(
    mut agents: Query<(Entity, &mut Agent), With<Agent>>,
    mut factions: Query<(Entity, &mut Faction), With<Faction>>,
    mut links: Query<(Entity, &mut AgentFactionLink), With<AgentFactionLink>>,
) {
    for (e, mut a) in agents.iter_mut() {
        a.id = e;
    }
    for (e, mut f) in factions.iter_mut() {
        f.id = e;
    }
    for (e, mut link) in links.iter_mut() {
        link.agent = e;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sync_fills_placeholder_ids() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.add_systems(PreUpdate, behavior_sync_entity_ids_system);

        let fe = app.world_mut().spawn(Faction::default()).id();
        let e = app
            .world_mut()
            .spawn((
                Agent {
                    id: Entity::PLACEHOLDER,
                    traits: HybridAgentTraits::default(),
                    emotional_state: HybridAgentEmotions::default(),
                    cognition: CognitiveState::default(),
                    mode: AgentMode::default(),
                },
                AgentFactionLink {
                    agent: Entity::PLACEHOLDER,
                    faction: fe,
                    loyalty: 0.7,
                    influence: 0.5,
                    autonomy: 0.2,
                },
            ))
            .id();

        app.update();

        let world = app.world();
        let agent = world.entity(e).get::<Agent>().expect("agent");
        assert_eq!(agent.id, e);
        let link = world.entity(e).get::<AgentFactionLink>().expect("link");
        assert_eq!(link.agent, e);
        assert_eq!(link.faction, fe);
        let fac = world.entity(fe).get::<Faction>().expect("faction");
        assert_eq!(fac.id, fe);
    }
}
