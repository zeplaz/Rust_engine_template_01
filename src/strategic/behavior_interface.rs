//! **Layer 2 — behavior interface**: pluggable [`BehaviorModel`] + decision hook (no logic locked in yet).

use bevy::prelude::*;

use super::hybrid_fields::WorldFields;
use super::hybrid_brain::HybridSimPhaseClock;

/// Read-only slice of world state for evaluation (no `&mut World`).
#[derive(Clone, Debug)]
pub struct BehaviorContext<'a> {
    pub world_fields: &'a WorldFields,
    pub phase_tick: u64,
}

/// Opaque decision bundle — replace with real action schema when wiring fuzzy / statistical brains.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct DecisionSet {
    /// Placeholder count for tests / HUD (“did the hook run?”).
    pub placeholder_len: usize,
}

/// Pluggable brain: statistical, fuzzy, scripted overrides, faction constraints — implementors stay outside hot path until registered.
pub trait BehaviorModel: Send + Sync + 'static {
    fn evaluate(&self, ctx: &BehaviorContext<'_>) -> DecisionSet;
}

/// Default no-op — deterministic empty output.
#[derive(Clone, Copy, Debug, Default)]
pub struct NoopBehaviorModel;

impl BehaviorModel for NoopBehaviorModel {
    fn evaluate(&self, _ctx: &BehaviorContext<'_>) -> DecisionSet {
        DecisionSet::default()
    }
}

/// Active model slot (swap at runtime for tooling / scenarios).
#[derive(Resource)]
pub struct ActiveBehaviorModel {
    inner: Box<dyn BehaviorModel>,
}

impl Default for ActiveBehaviorModel {
    fn default() -> Self {
        Self {
            inner: Box::new(NoopBehaviorModel),
        }
    }
}

impl ActiveBehaviorModel {
    pub fn set(&mut self, model: Box<dyn BehaviorModel>) {
        self.inner = model;
    }

    #[inline]
    pub fn evaluate(&self, ctx: &BehaviorContext<'_>) -> DecisionSet {
        self.inner.evaluate(ctx)
    }
}

/// Last hook output — control plane / HUD can read without touching agents.
#[derive(Resource, Clone, Debug, Default)]
pub struct DecisionPipelineSink {
    pub last: Option<DecisionSet>,
    pub last_phase_tick: u64,
    /// Mean of [`super::behavior_pipeline::compose_decision_score`] across sampled agents (diagnostics).
    pub last_mean_composed_score: f32,
    pub last_agent_samples: usize,
}

/// Runs **before** hybrid intent systems — later: inject derived weights here.
pub fn behavior_model_evaluation_hook_system(
    model: Res<ActiveBehaviorModel>,
    world_fields: Res<WorldFields>,
    clock: Res<HybridSimPhaseClock>,
    mut sink: ResMut<DecisionPipelineSink>,
) {
    let ctx = BehaviorContext {
        world_fields: world_fields.as_ref(),
        phase_tick: clock.tick,
    };
    let decisions = model.evaluate(&ctx);
    sink.last = Some(decisions.clone());
    sink.last_phase_tick = clock.tick;
}

#[cfg(test)]
mod tests {
    use super::*;

    struct AlwaysOne;
    impl BehaviorModel for AlwaysOne {
        fn evaluate(&self, _: &BehaviorContext<'_>) -> DecisionSet {
            DecisionSet {
                placeholder_len: 1,
            }
        }
    }

    #[test]
    fn noop_returns_empty_decision_set() {
        let m = NoopBehaviorModel;
        let wf = WorldFields::default();
        let ctx = BehaviorContext {
            world_fields: &wf,
            phase_tick: 0,
        };
        assert_eq!(m.evaluate(&ctx), DecisionSet::default());
    }

    #[test]
    fn active_model_can_be_swapped() {
        let mut active = ActiveBehaviorModel::default();
        active.set(Box::new(AlwaysOne));
        let wf = WorldFields::default();
        let ctx = BehaviorContext {
            world_fields: &wf,
            phase_tick: 0,
        };
        assert_eq!(active.evaluate(&ctx).placeholder_len, 1);
    }
}
