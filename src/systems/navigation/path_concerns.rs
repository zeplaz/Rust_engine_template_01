//! NAV-14 — per-order-type concern weights (time vs environmental risk) for grid pathfinding.
//!
//! Weights load from [`PATH_CONCERNS_RON`]; embedded copy is the fallback when disk is missing.

use std::collections::HashMap;
use std::path::PathBuf;

use bevy::prelude::*;
use serde::Deserialize;

pub const PATH_CONCERNS_RON: &str = "assets/configs/navigation/logistics_path_concerns_v1.ron";

const EMBEDDED: &str =
    include_str!("../../../assets/configs/navigation/logistics_path_concerns_v1.ron");

/// Blends summed edge cost (time proxy) with path max [`LogisticsTile::env_risk`](super::LogisticsTile::env_risk).
#[derive(Clone, Copy, Debug, PartialEq, Deserialize)]
pub struct LogisticsPathConcern {
    pub time_weight: f32,
    pub risk_weight: f32,
}

impl Default for LogisticsPathConcern {
    fn default() -> Self {
        Self::DEFAULT
    }
}

impl LogisticsPathConcern {
    pub const DEFAULT: Self = Self {
        time_weight: 1.0,
        risk_weight: 0.0,
    };

    #[must_use]
    pub fn blend_cost(self, time_cost: f32, max_risk: f32) -> f32 {
        self.time_weight * time_cost + self.risk_weight * max_risk.clamp(0.0, 1.0)
    }
}

#[derive(Clone, Copy, Debug, Deserialize)]
struct ConcernRow {
    time_weight: f32,
    risk_weight: f32,
}

impl From<ConcernRow> for LogisticsPathConcern {
    fn from(r: ConcernRow) -> Self {
        Self {
            time_weight: r.time_weight,
            risk_weight: r.risk_weight,
        }
    }
}

fn default_concern_row() -> ConcernRow {
    ConcernRow {
        time_weight: 1.0,
        risk_weight: 0.0,
    }
}

#[derive(Debug, Deserialize)]
struct PathConcernsFile {
    #[serde(default)]
    schema_version: u32,
    #[serde(default)]
    table_id: String,
    #[serde(default = "default_concern_row")]
    default: ConcernRow,
    #[serde(default)]
    by_consumer: HashMap<String, ConcernRow>,
    #[serde(default)]
    by_order_kind: HashMap<String, ConcernRow>,
    #[serde(default)]
    profiles: HashMap<String, ConcernRow>,
}

#[derive(Resource, Debug, Clone)]
pub struct LogisticsPathConcernRegistry {
    pub default: LogisticsPathConcern,
    pub by_consumer: HashMap<String, LogisticsPathConcern>,
    pub by_order_kind: HashMap<String, LogisticsPathConcern>,
    pub profiles: HashMap<String, LogisticsPathConcern>,
    pub schema_version: u32,
    pub table_id: String,
}

impl Default for LogisticsPathConcernRegistry {
    fn default() -> Self {
        load_logistics_path_concerns().0
    }
}

/// NAV-14 — concern weights for tactical grid paths that feed a front (RON `by_consumer.front`).
#[must_use]
pub fn front_haul_path_concern(reg: &LogisticsPathConcernRegistry) -> LogisticsPathConcern {
    reg.get_consumer("front")
}

impl LogisticsPathConcernRegistry {
    #[must_use]
    pub fn get_consumer(&self, label: &str) -> LogisticsPathConcern {
        self.by_consumer
            .get(label)
            .copied()
            .unwrap_or(self.default)
    }

    #[must_use]
    pub fn get_order_kind(&self, label: &str) -> LogisticsPathConcern {
        self.by_order_kind
            .get(label)
            .copied()
            .unwrap_or(self.default)
    }

    #[must_use]
    pub fn profile(&self, name: &str) -> LogisticsPathConcern {
        self.profiles.get(name).copied().unwrap_or(self.default)
    }
}

fn parse_file(text: &str) -> Result<LogisticsPathConcernRegistry, String> {
    let file: PathConcernsFile = ron::de::from_str(text).map_err(|e| e.to_string())?;
    let map = |m: HashMap<String, ConcernRow>| -> HashMap<String, LogisticsPathConcern> {
        m.into_iter().map(|(k, v)| (k, v.into())).collect()
    };
    Ok(LogisticsPathConcernRegistry {
        default: file.default.into(),
        by_consumer: map(file.by_consumer),
        by_order_kind: map(file.by_order_kind),
        profiles: map(file.profiles),
        schema_version: file.schema_version,
        table_id: file.table_id,
    })
}

#[must_use]
pub fn load_logistics_path_concerns() -> (LogisticsPathConcernRegistry, Option<String>) {
    let path = PathBuf::from(PATH_CONCERNS_RON);
    if path.is_file() {
        match std::fs::read_to_string(&path) {
            Ok(text) => match parse_file(&text) {
                Ok(reg) => return (reg, None),
                Err(e) => {
                    return (
                        parse_file(EMBEDDED).expect("embedded path concerns"),
                        Some(format!("disk parse failed ({e}); using embedded")),
                    );
                }
            },
            Err(e) => {
                return (
                    parse_file(EMBEDDED).expect("embedded path concerns"),
                    Some(format!("disk read failed ({e}); using embedded")),
                );
            }
        }
    }
    (
        parse_file(EMBEDDED).expect("embedded path concerns"),
        Some(format!("missing {}; using embedded", path.display())),
    )
}

pub fn load_logistics_path_concerns_on_startup(mut commands: Commands) {
    let (reg, err) = load_logistics_path_concerns();
    if let Some(err) = err {
        warn!("logistics_path_concerns: {err}");
    }
    commands.insert_resource(reg);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn embedded_risk_averse_profile_exists() {
        let reg = parse_file(EMBEDDED).unwrap();
        let c = reg.profile("risk_averse");
        assert!(c.risk_weight > 5.0, "risk_averse should weight fire heavily");
        assert_eq!(reg.get_consumer("front").risk_weight, 2.5);
    }
}