use bevy::prelude::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Deployable prefab recipes (**COD-DEPLOYABLE-RECIPE-001**) — Custom domain only.
pub const MFG_DRAGON_TEETH_V1: &str = "mfg_dragon_teeth_v1";
pub const MFG_MINE_UNIT_V1: &str = "mfg_mine_unit_v1";

/// Freight / staging buffer tags — hauled via `FreightLot.buffer_tag` → [`crate::economy::logistics::SiteStagingStock`].
pub const BUFFER_TAG_DRAGON_TEETH_UNIT: &str = "dragon_teeth_unit";
pub const BUFFER_TAG_MINE_UNIT: &str = "mine_unit";

/// Tags that arrive into [`crate::economy::logistics::SiteStagingStock`] (not `ResourceFlowNode`).
#[must_use]
pub fn is_deployable_staging_buffer_tag(tag: &str) -> bool {
    tag == BUFFER_TAG_DRAGON_TEETH_UNIT || tag == BUFFER_TAG_MINE_UNIT
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ManufacturingDomain {
    Concrete,
    Aluminum,
    Power,
    Custom,
}

/// Serializable domain-level blueprint for modular factories.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ManufacturingBlueprint {
    pub id: String,
    pub domain: ManufacturingDomain,
    pub process_tags: Vec<String>,
    pub throughput_target: f32,
    /// When set, `tick_manufacturing_nodes` credits this tag on [`ManufacturingOutputBuffers`].
    /// Concrete/Aluminum/Power keep `None` until their chains wire string tags explicitly.
    #[serde(default)]
    pub output_buffer_tag: Option<String>,
}

/// Plant-local tagged output stock.
///
/// **Writers:** `tick_manufacturing_nodes` (credit) · `dispatch_deployable_from_manufacturing_system`
/// (debit → `InTransitLedger`). UI / place must never touch this component.
#[derive(Component, Debug, Clone, Default)]
pub struct ManufacturingOutputBuffers {
    pub amounts: HashMap<String, f32>,
}

impl ManufacturingOutputBuffers {
    #[must_use]
    pub fn get(&self, tag: &str) -> f32 {
        self.amounts.get(tag).copied().unwrap_or(0.0)
    }

    pub fn credit(&mut self, tag: &str, amount: f32) {
        if amount <= 0.0 {
            return;
        }
        *self.amounts.entry(tag.to_string()).or_insert(0.0) += amount;
    }

    /// Remove up to `amount`; returns taken. Used by deployable freight dispatch only.
    pub fn debit(&mut self, tag: &str, amount: f32) -> f32 {
        let take = self.get(tag).min(amount.max(0.0));
        if take <= 0.0 {
            return 0.0;
        }
        let left = self.get(tag) - take;
        if left <= 1e-6 {
            self.amounts.remove(tag);
        } else {
            self.amounts.insert(tag.to_string(), left);
        }
        take
    }
}

/// ECS runtime marker to attach a blueprint id to an entity.
#[derive(Component, Debug, Clone)]
pub struct ManufacturingNode {
    pub blueprint_id: String,
    pub local_efficiency: f32,
    /// Live rate this tick (`throughput_target × local_efficiency`).
    pub current_throughput: f32,
}
