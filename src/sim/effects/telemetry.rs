//! SimEffect telemetry ledger — P1 causal rows + JSONL export (SIM-EFFECT-TEL-001).

use std::fs::OpenOptions;
use std::io::{BufWriter, Write};
use std::path::Path;

use bevy::prelude::*;
use serde::{Deserialize, Serialize};

use super::event::SimEffectEvent;

pub const SIM_EFFECTS_JSONL: &str = "debug_runs/sim_effects/effects.jsonl";

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SimEffectTelemetryRecord {
    pub effect_id: u64,
    pub sim_tick: u64,
    pub source: super::event::SimEffectSource,
    pub cause_id: String,
    pub parent_effect_id: Option<u64>,
    pub kind_tag: u8,
    pub dispatch_ok: bool,
}

#[derive(Resource, Debug, Clone)]
pub struct SimEffectTelemetryLedger {
    pub rows: Vec<SimEffectTelemetryRecord>,
    pub effect_rows: u64,
    run_id: String,
    next_effect_id: u64,
}

impl Default for SimEffectTelemetryLedger {
    fn default() -> Self {
        Self {
            rows: Vec::new(),
            effect_rows: 0,
            run_id: String::new(),
            // Match reset_run_id — id 0 is skipped by PlayerEventLog projection cursor (<=).
            next_effect_id: 1,
        }
    }
}

impl SimEffectTelemetryLedger {
    pub fn reset_run_id(&mut self, run_id: &str) {
        self.run_id = run_id.to_string();
        self.rows.clear();
        self.effect_rows = 0;
        self.next_effect_id = 1;
    }

    pub fn record_drain(&mut self, tick: u64, event: &SimEffectEvent, ok: bool) -> u64 {
        let effect_id = self.next_effect_id;
        self.next_effect_id = self.next_effect_id.saturating_add(1);
        self.rows.push(SimEffectTelemetryRecord {
            effect_id,
            sim_tick: tick,
            source: event.source,
            cause_id: event.cause_id.clone(),
            parent_effect_id: event.parent_effect_id,
            kind_tag: event.kind.dedupe_tag(),
            dispatch_ok: ok,
        });
        self.effect_rows = self.effect_rows.saturating_add(1);
        effect_id
    }

    #[must_use]
    pub fn causal_chain_depth_max(&self) -> u64 {
        let mut depth = 0u64;
        for row in &self.rows {
            let mut chain = 1u64;
            let mut parent = row.parent_effect_id;
            let mut guard = 0u32;
            while let Some(pid) = parent {
                chain = chain.saturating_add(1);
                parent = self
                    .rows
                    .iter()
                    .find(|r| r.effect_id == pid)
                    .and_then(|r| r.parent_effect_id);
                guard = guard.saturating_add(1);
                if guard > 64 {
                    break;
                }
            }
            depth = depth.max(chain);
        }
        depth
    }

    pub fn export_jsonl(&self, path: &Path) -> std::io::Result<()> {
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let file = OpenOptions::new()
            .create(true)
            .write(true)
            .truncate(true)
            .open(path)?;
        let mut writer = BufWriter::new(file);
        for row in &self.rows {
            let line = serde_json::json!({
                "run_id": self.run_id,
                "effect_id": row.effect_id,
                "sim_tick": row.sim_tick,
                "source": row.source.as_str(),
                "cause_id": row.cause_id,
                "parent_effect_id": row.parent_effect_id,
                "kind_tag": row.kind_tag,
                "dispatch_ok": row.dispatch_ok,
            });
            writeln!(writer, "{line}")?;
        }
        Ok(())
    }
}
