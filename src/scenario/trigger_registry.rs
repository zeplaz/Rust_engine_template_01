//! **VSS-T2-002** — resolve `TriggerEffect(effect_id)` → trigger RON → SimEffect enqueue.
//!
//! **ECS-12** (gea4v2 17.8.11.1–2 *Data pathway communication*) · LEGACY `events_message_passing`
//! (gea_v2_e4 p632): the registry is the set of *declared edges* between script steps and fire
//! intents. An edge exists only when `assets/scenarios/triggers/` holds a spec whose own
//! `id` equals the requested id; anything else (malformed id, missing file, id/file mismatch,
//! parse failure, no cells, unknown source) is a rejected edge and yields **no** step.
//! The registry only loads and resolves data into an intent step — it never touches sim state.
//!
//! **DEC-11** (aig3 5.8 *Rule-Based Systems*): [`TriggerFactMemory`] is shared fact memory;
//! [`arbitrate_trigger_rules`] picks one firing rule by priority when many match (not a full Rete).

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use super::scenario_steps::ScenarioStep;
use std::collections::VecDeque;
use super::trigger_spec::TriggerSpecV1;


/// DEC-11: one production rule over shared facts (alpha predicates are fact-key thresholds).
#[derive(Clone, Debug, PartialEq)]
pub struct TriggerRule {
    pub id: &'static str,
    /// Higher wins arbitration when several rules match.
    pub priority: i32,
    /// `(fact_key, min_value)` — all must hold (AND join in beta spirit).
    pub requires: &'static [(&'static str, f32)],
}

/// DEC-11: shared fact memory — many rules read one board of facts.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct TriggerFactMemory {
    pub tick: u64,
    pub facts: BTreeMap<&'static str, f32>,
}

impl TriggerFactMemory {
    pub fn set(&mut self, key: &'static str, value: f32) {
        self.facts.insert(key, value);
    }

    #[must_use]
    pub fn get(&self, key: &str) -> f32 {
        self.facts.get(key).copied().unwrap_or(0.0)
    }

    /// Alpha index: rule ids interested in `key` (for incremental updates).
    #[must_use]
    pub fn alpha_index(rules: &[TriggerRule]) -> BTreeMap<&'static str, Vec<&'static str>> {
        let mut idx: BTreeMap<&'static str, Vec<&'static str>> = BTreeMap::new();
        for r in rules {
            for (k, _) in r.requires {
                idx.entry(*k).or_default().push(r.id);
            }
        }
        for v in idx.values_mut() {
            v.sort_unstable();
            v.dedup();
        }
        idx
    }
}

#[must_use]
pub fn rule_matches(memory: &TriggerFactMemory, rule: &TriggerRule) -> bool {
    rule.requires
        .iter()
        .all(|(k, min)| memory.get(k) + f32::EPSILON >= *min)
}

/// DEC-11 arbitration: among matching rules, highest priority then lexical id.
#[must_use]
pub fn arbitrate_trigger_rules<'a>(
    memory: &TriggerFactMemory,
    rules: &'a [TriggerRule],
) -> Option<&'a TriggerRule> {
    rules
        .iter()
        .filter(|r| rule_matches(memory, r))
        .max_by(|a, b| a.priority.cmp(&b.priority).then(b.id.cmp(a.id)))
}

pub const TRIGGER_ASSET_DIR: &str = "assets/scenarios/triggers";

/// Known trigger ids whose on-disk filename differs from `{id}.trigger.ron`.
const TRIGGER_ID_ALIASES: &[(&str, &str)] = &[("demo_ignite_v0", "demo_ignite")];

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TriggerLoadError {
    NotFound,
    ReadFailed,
    ParseFailed,
    EmptyCells,
    UnsupportedSource,
    /// Id is not a plain `[A-Za-z0-9_-]+` token (path separators / traversal never name an edge).
    InvalidId,
}

fn triggers_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join(TRIGGER_ASSET_DIR)
}

/// True when `effect_id` is a plain token that may name a pathway edge.
#[must_use]
pub fn is_valid_trigger_id(effect_id: &str) -> bool {
    !effect_id.is_empty()
        && effect_id
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || b == b'_' || b == b'-')
}

fn trigger_path_for_id(effect_id: &str) -> PathBuf {
    let root = triggers_root();
    let direct = root.join(format!("{effect_id}.trigger.ron"));
    if direct.is_file() {
        return direct;
    }
    for (id, stem) in TRIGGER_ID_ALIASES {
        if *id == effect_id {
            let alias = root.join(format!("{stem}.trigger.ron"));
            if alias.is_file() {
                return alias;
            }
        }
    }
    direct
}

/// Parse trigger RON text and accept it only as the edge named `effect_id`.
pub fn parse_trigger_spec(effect_id: &str, text: &str) -> Result<TriggerSpecV1, TriggerLoadError> {
    let spec: TriggerSpecV1 = ron::from_str(text).map_err(|_| TriggerLoadError::ParseFailed)?;
    if spec.id != effect_id {
        return Err(TriggerLoadError::NotFound);
    }
    if spec.cells.is_empty() {
        return Err(TriggerLoadError::EmptyCells);
    }
    Ok(spec)
}

/// Load a trigger spec by logical effect id (filename or alias).
pub fn load_trigger_spec(effect_id: &str) -> Result<TriggerSpecV1, TriggerLoadError> {
    if !is_valid_trigger_id(effect_id) {
        return Err(TriggerLoadError::InvalidId);
    }
    let path = trigger_path_for_id(effect_id);
    let text = std::fs::read_to_string(&path).map_err(|_| TriggerLoadError::NotFound)?;
    parse_trigger_spec(effect_id, &text)
}

/// Resolve trigger id to an emit step (validation / script drain).
pub fn trigger_effect_to_emit_step(effect_id: &str) -> Result<ScenarioStep, TriggerLoadError> {
    let spec = load_trigger_spec(effect_id)?;
    spec.to_emit_step().ok_or(TriggerLoadError::UnsupportedSource)
}

/// Production trigger rules for DEC-11 arbitration (ids name registered assets).
pub const SCENARIO_TRIGGER_RULES: &[TriggerRule] = &[
    TriggerRule {
        id: "demo_ignite_v0",
        priority: 1,
        requires: &[("drought", 0.35)],
    },
    TriggerRule {
        id: "demo_ignite_v0",
        priority: 5,
        requires: &[("drought", 0.75), ("fuel", 0.2)],
    },
];

/// Build fact memory for scenario trigger arbitration from live weather (read-only).
#[must_use]
pub fn build_trigger_fact_memory(tick: u64, drought: f32, fuel: f32, rain: f32) -> TriggerFactMemory {
    let mut mem = TriggerFactMemory::default();
    mem.tick = tick;
    mem.set("drought", drought.clamp(0.0, 1.0));
    mem.set("fuel", fuel.clamp(0.0, 1.0));
    mem.set("rain", rain.clamp(0.0, 1.0));
    mem
}

/// When several `TriggerEffect` steps are queued, run the highest-priority matching rule first.
pub fn reorder_pending_trigger_effects(
    pending: &mut VecDeque<ScenarioStep>,
    memory: &TriggerFactMemory,
    rules: &[TriggerRule],
) {
    let trigger_count = pending
        .iter()
        .take_while(|s| matches!(s, ScenarioStep::TriggerEffect { .. }))
        .count();
    if trigger_count < 2 {
        return;
    }
    let winner = arbitrate_trigger_rules(memory, rules).map(|r| r.id);
    let Some(win_id) = winner else {
        return;
    };
    let mut prefix: Vec<ScenarioStep> = pending.drain(..trigger_count).collect();
    prefix.sort_by(|a, b| {
        let pri = |s: &ScenarioStep| match s {
            ScenarioStep::TriggerEffect { effect_id } if effect_id == win_id => 10_000,
            ScenarioStep::TriggerEffect { effect_id } => rules
                .iter()
                .find(|r| r.id == effect_id)
                .map_or(0, |r| r.priority),
            _ => 0,
        };
        pri(b).cmp(&pri(a))
    });
    for step in prefix.into_iter().rev() {
        pending.push_front(step);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bevy::input::InputPlugin;
    use bevy::prelude::*;

    use crate::gui::InputBindings;
    use crate::scenario::scenario_plugin::ScenarioScriptingPlugin;
    use crate::scenario::scenario_types::ScenarioFileV1;
    use crate::scenario::script_host::EngineScriptHost;
    use crate::sim::effects::SimEffectQueue;
    use crate::systems::sim_control::SimControlPlugin;

    const GOOD: &str = r#"(
        schema_version: 1, id: "edge_a", when_tick_min: None, source: "ecology",
        cause_id: "CAUSE-a", parent_effect_id: None,
        cells: [(chunk_x: 1, chunk_y: 1, cell: 0, spark: 0.2)],
    )"#;

    #[test]
    fn trigger_effect_resolves_demo_ignite_alias() {
        let step = trigger_effect_to_emit_step("demo_ignite_v0").expect("demo trigger");
        assert!(step.routes_fire_via_sim_effect());
    }

    #[test]
    fn ecs12_registered_pathway_resolves_to_emit_intent_only() {
        let step = trigger_effect_to_emit_step("demo_ignite_v0").expect("registered edge");
        assert!(matches!(step, ScenarioStep::EmitSimEffect { .. }));
    }

    #[test]
    fn ecs12_unregistered_or_malformed_edges_do_not_resolve() {
        assert_eq!(
            trigger_effect_to_emit_step("no_such_trigger").unwrap_err(),
            TriggerLoadError::NotFound
        );
        assert_eq!(load_trigger_spec("").unwrap_err(), TriggerLoadError::InvalidId);
        for bad in ["../triggers/demo_ignite", "a/b", "a\\b", "demo ignite", "demo_ignite.v0"] {
            assert_eq!(
                load_trigger_spec(bad).unwrap_err(),
                TriggerLoadError::InvalidId,
                "{bad}"
            );
        }
    }

    #[test]
    fn ecs12_file_stem_is_not_an_edge_unless_spec_id_matches() {
        // `demo_ignite.trigger.ron` exists, but its declared id is `demo_ignite_v0`.
        assert_eq!(
            load_trigger_spec("demo_ignite").unwrap_err(),
            TriggerLoadError::NotFound
        );
    }

    #[test]
    fn ecs12_parse_rejects_mismatch_garbage_and_empty_cells() {
        assert!(parse_trigger_spec("edge_a", GOOD).is_ok());
        assert_eq!(
            parse_trigger_spec("edge_b", GOOD).unwrap_err(),
            TriggerLoadError::NotFound
        );
        assert_eq!(
            parse_trigger_spec("edge_a", "not ron").unwrap_err(),
            TriggerLoadError::ParseFailed
        );
        let empty = GOOD.replace("cells: [(chunk_x: 1, chunk_y: 1, cell: 0, spark: 0.2)]", "cells: []");
        assert_eq!(
            parse_trigger_spec("edge_a", &empty).unwrap_err(),
            TriggerLoadError::EmptyCells
        );
    }

    #[test]
    fn ecs12_unknown_source_is_rejected_edge() {
        let bad = GOOD.replace("\"ecology\"", "\"mystery\"");
        let spec = parse_trigger_spec("edge_a", &bad).expect("parses");
        assert!(spec.to_emit_step().is_none());
    }

    /// Pathway modules stay data → intent only: no ECS handles, no sim-effect queue access.
    #[test]
    fn ecs12_pathway_modules_hold_no_sim_write_surface() {
        for (name, src) in [
            ("trigger_registry.rs", include_str!("trigger_registry.rs")),
            ("trigger_spec.rs", include_str!("trigger_spec.rs")),
        ] {
            let prod = src.split_once("#[cfg(test)]").map_or(src, |(p, _)| p);
            for forbidden in [
                "ResMut", "Commands", "Query<", "&mut World", "SimEffectQueue", "SimControlState",
                // `.push(` is allowed for pure index builders (e.g. alpha_index); ban sim queues only.
            ] {
                assert!(
                    !prod.contains(forbidden),
                    "{name} production code must stay intent-only (found `{forbidden}`)"
                );
            }
        }
    }

    fn run_script(steps_ron: &str) -> u64 {
        let text = format!(
            r#"(schema_version: 2,
                metadata: (id: "ecs12", display_name: "ecs12", author: None, description: None),
                steps: [{steps_ron}])"#
        );
        let file: ScenarioFileV1 = ron::from_str(&text).expect("script ron");
        let mut app = App::new();
        app.add_plugins((MinimalPlugins, InputPlugin));
        app.init_resource::<InputBindings>();
        app.init_resource::<SimEffectQueue>();
        app.add_plugins((SimControlPlugin, ScenarioScriptingPlugin));
        app.world_mut()
            .resource_mut::<EngineScriptHost>()
            .load_script(file);
        for _ in 0..4 {
            app.update();
        }
        app.world().resource::<SimEffectQueue>().pushed_total as u64
    }

    #[test]
    fn ecs12_registered_pathway_fires_and_unregistered_does_not_broadcast() {
        let fired = run_script(r#"TriggerEffect(effect_id: "demo_ignite_v0")"#);
        assert!(fired >= 1, "registered edge must enqueue a sim effect");
        let missing = run_script(r#"TriggerEffect(effect_id: "no_such_trigger")"#);
        assert_eq!(missing, 0, "unregistered edge must not broadcast");
        let traversal = run_script(r#"TriggerEffect(effect_id: "../triggers/demo_ignite")"#);
        assert_eq!(traversal, 0, "malformed id must not broadcast");
    }
    #[test]
    fn dec11_rete_fact_memory_arbitrates_by_priority() {
        let rules = [
            TriggerRule {
                id: "ignite_dry",
                priority: 1,
                requires: &[("drought", 0.5), ("fuel", 0.2)],
            },
            TriggerRule {
                id: "ignite_urgent",
                priority: 5,
                requires: &[("drought", 0.5)],
            },
            TriggerRule {
                id: "rain_out",
                priority: 9,
                requires: &[("rain", 0.8)],
            },
        ];
        let idx = TriggerFactMemory::alpha_index(&rules);
        assert!(idx.get("drought").unwrap().contains(&"ignite_dry"));
        let mut mem = TriggerFactMemory::default();
        mem.set("drought", 0.7);
        mem.set("fuel", 0.3);
        let win = arbitrate_trigger_rules(&mem, &rules).expect("match");
        assert_eq!(win.id, "ignite_urgent");
        mem.set("rain", 0.9);
        let win2 = arbitrate_trigger_rules(&mem, &rules).expect("rain");
        assert_eq!(win2.id, "rain_out");
    }

}
