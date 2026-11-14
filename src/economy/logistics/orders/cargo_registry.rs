//! CargoKind registry (`assets/configs/economy/cargo_kinds_v1.ron`) — spine P0.

use std::collections::HashMap;
use std::path::{Path, PathBuf};

use bevy::prelude::*;
use serde::Deserialize;

use super::model::{CargoClass, CargoKind};

pub const CARGO_KINDS_RON: &str = "assets/configs/economy/cargo_kinds_v1.ron";

const EMBEDDED: &str = include_str!("../../../../assets/configs/economy/cargo_kinds_v1.ron");

#[derive(Debug, Clone, Deserialize)]
struct CargoKindsFile {
    #[serde(default)]
    schema_version: u32,
    #[serde(default)]
    table_id: String,
    kinds: Vec<CargoKindRow>,
}

#[derive(Debug, Clone, Deserialize)]
struct CargoKindRow {
    tag: String,
    class: String,
    #[serde(default)]
    consumers: Vec<String>,
    #[serde(default)]
    notes: String,
}

#[derive(Debug, Clone)]
pub struct RegisteredCargoKind {
    pub tag: String,
    pub class: CargoClass,
    pub consumers: Vec<String>,
    pub notes: String,
}

/// Loaded cargo registry (sole writer: load on startup / Default).
#[derive(Resource, Debug, Clone)]
pub struct CargoKindRegistry {
    pub by_tag: HashMap<String, RegisteredCargoKind>,
    pub schema_version: u32,
    pub table_id: String,
}

impl Default for CargoKindRegistry {
    fn default() -> Self {
        load_cargo_kind_registry().0
    }
}

impl CargoKindRegistry {
    #[must_use]
    pub fn get(&self, tag: &str) -> Option<&RegisteredCargoKind> {
        self.by_tag.get(tag)
    }

    #[must_use]
    pub fn contains(&self, tag: &str) -> bool {
        self.by_tag.contains_key(tag)
    }

    /// Registry class if known; else [`CargoKind::class`] heuristic.
    #[must_use]
    pub fn class_of(&self, cargo: &CargoKind) -> CargoClass {
        self.get(cargo.as_str())
            .map(|r| r.class)
            .unwrap_or_else(|| cargo.class())
    }
}

fn parse_class(s: &str) -> CargoClass {
    match s.trim().to_ascii_lowercase().as_str() {
        "bulk" => CargoClass::Bulk,
        "liquid" => CargoClass::Liquid,
        "personnel" => CargoClass::Personnel,
        "munitions" => CargoClass::Munitions,
        "deployable" => CargoClass::Deployable,
        _ => CargoClass::Bulk,
    }
}

fn parse_file(text: &str) -> Result<CargoKindRegistry, String> {
    let file: CargoKindsFile = ron::de::from_str(text).map_err(|e| e.to_string())?;
    let mut by_tag = HashMap::new();
    for row in file.kinds {
        by_tag.insert(
            row.tag.clone(),
            RegisteredCargoKind {
                tag: row.tag,
                class: parse_class(&row.class),
                consumers: row.consumers,
                notes: row.notes,
            },
        );
    }
    Ok(CargoKindRegistry {
        by_tag,
        schema_version: file.schema_version,
        table_id: file.table_id,
    })
}

#[must_use]
pub fn load_cargo_kind_registry() -> (CargoKindRegistry, Option<String>) {
    let path = PathBuf::from(CARGO_KINDS_RON);
    if path.is_file() {
        match std::fs::read_to_string(&path) {
            Ok(text) => match parse_file(&text) {
                Ok(reg) => return (reg, None),
                Err(e) => {
                    return (
                        parse_file(EMBEDDED).expect("embedded cargo_kinds"),
                        Some(format!("disk parse failed ({e}); using embedded")),
                    );
                }
            },
            Err(e) => {
                return (
                    parse_file(EMBEDDED).expect("embedded cargo_kinds"),
                    Some(format!("disk read failed ({e}); using embedded")),
                );
            }
        }
    }
    (
        parse_file(EMBEDDED).expect("embedded cargo_kinds"),
        Some(format!("missing {}; using embedded", path.display())),
    )
}

pub fn load_cargo_kind_registry_on_startup(mut commands: Commands) {
    let (reg, err) = load_cargo_kind_registry();
    if let Some(err) = err {
        warn!("cargo_kinds registry: {err}");
    }
    commands.insert_resource(reg);
}

/// True when `tag` is registered or matches a known heuristic class prefix (legacy).
#[must_use]
pub fn cargo_tag_known(reg: &CargoKindRegistry, tag: &str) -> bool {
    if reg.contains(tag) {
        return true;
    }
    // Heuristic escape hatch for dynamic tags not yet migrated.
    let c = CargoKind::new(tag).class();
    !matches!(c, CargoClass::Bulk) || tag.chars().next().is_some_and(|ch| ch.is_ascii_uppercase())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn embedded_has_drone_airframe_and_fuel() {
        let reg = parse_file(EMBEDDED).unwrap();
        assert!(reg.contains("drone_airframe"));
        assert!(reg.contains("Fuel"));
        assert!(reg.contains("personnel"));
        assert!(reg.contains("missile_body"));
        assert_eq!(reg.get("Oil").unwrap().class, CargoClass::Liquid);
        assert_eq!(reg.get("ammo").unwrap().class, CargoClass::Munitions);
        assert_eq!(reg.get("missile_body").unwrap().class, CargoClass::Munitions);
        assert_eq!(reg.table_id, "cargo_kinds_v1");
    }

    #[test]
    fn path_constant_exists_or_embedded() {
        let _ = Path::new(CARGO_KINDS_RON);
        let (reg, _) = load_cargo_kind_registry();
        assert!(!reg.by_tag.is_empty());
    }
}
