//! Capability TOE table (`assets/configs/combat/capability_toe_v1.ron`) — spine P0 / P3 / P5 / P6.

use std::collections::HashMap;
use std::path::PathBuf;

use bevy::prelude::*;
use serde::Deserialize;

use super::drone_missions::DroneRole;

pub const CAPABILITY_TOE_RON: &str = "assets/configs/combat/capability_toe_v1.ron";

/// Missile / AD magazine class (RON identity).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum MissileClass {
    GenericCruise = 0,
    SamShort = 1,
    SamMedium = 2,
}

impl MissileClass {
    pub const ALL: [MissileClass; 3] = [
        MissileClass::GenericCruise,
        MissileClass::SamShort,
        MissileClass::SamMedium,
    ];

    #[must_use]
    pub fn index(self) -> usize {
        self as usize
    }

    #[must_use]
    pub fn as_str(self) -> &'static str {
        match self {
            MissileClass::GenericCruise => "GenericCruise",
            MissileClass::SamShort => "SamShort",
            MissileClass::SamMedium => "SamMedium",
        }
    }

    #[must_use]
    pub fn parse(s: &str) -> Option<Self> {
        match s {
            "GenericCruise" => Some(Self::GenericCruise),
            "SamShort" => Some(Self::SamShort),
            "SamMedium" => Some(Self::SamMedium),
            _ => None,
        }
    }
}

const EMBEDDED: &str = include_str!("../../assets/configs/combat/capability_toe_v1.ron");

#[derive(Debug, Clone, Deserialize)]
struct CapabilityToeFile {
    #[serde(default)]
    schema_version: u32,
    #[serde(default)]
    table_id: String,
    #[serde(default)]
    drone_roles: Vec<DroneRoleRow>,
    #[serde(default)]
    missiles: Vec<MissileRow>,
}

#[derive(Debug, Clone, Deserialize)]
struct DroneRoleRow {
    role: String,
    toe_count: u32,
    #[serde(default)]
    lines: Vec<ToeLineRow>,
}

#[derive(Debug, Clone, Deserialize)]
struct MissileRow {
    class: String,
    #[serde(default)]
    toe_count: u32,
    #[serde(default)]
    lines: Vec<ToeLineRow>,
    #[serde(default)]
    notes: String,
}

#[derive(Debug, Clone, Deserialize)]
struct ToeLineRow {
    tag: String,
    qty: u32,
}

#[derive(Debug, Clone)]
pub struct ToeLine {
    pub tag: String,
    pub qty: u32,
}

#[derive(Debug, Clone)]
pub struct DroneRoleToe {
    pub toe_count: u32,
    pub lines: Vec<ToeLine>,
}

#[derive(Debug, Clone)]
pub struct MissileClassToe {
    pub toe_count: u32,
    pub lines: Vec<ToeLine>,
}

/// Loaded capability TOE (sole writer: startup / Default).
#[derive(Resource, Debug, Clone)]
pub struct CapabilityToeRegistry {
    pub drone: HashMap<DroneRole, DroneRoleToe>,
    pub missile: HashMap<MissileClass, MissileClassToe>,
    pub schema_version: u32,
    pub table_id: String,
}

impl Default for CapabilityToeRegistry {
    fn default() -> Self {
        load_capability_toe_registry().0
    }
}

impl CapabilityToeRegistry {
    #[must_use]
    pub fn drone(&self, role: DroneRole) -> DroneRoleToe {
        self.drone.get(&role).cloned().unwrap_or(DroneRoleToe {
            toe_count: 4,
            lines: vec![
                ToeLine {
                    tag: "drone_airframe".into(),
                    qty: 1,
                },
                ToeLine {
                    tag: "drone_battery".into(),
                    qty: 1,
                },
            ],
        })
    }

    #[must_use]
    pub fn drone_toe_count(&self, role: DroneRole) -> u32 {
        self.drone(role).toe_count.max(1)
    }

    #[must_use]
    pub fn missile(&self, class: MissileClass) -> MissileClassToe {
        self.missile.get(&class).cloned().unwrap_or_else(|| default_missile_toe(class))
    }
}

fn default_missile_toe(class: MissileClass) -> MissileClassToe {
    let toe_count = match class {
        MissileClass::GenericCruise => 4,
        MissileClass::SamShort => 8,
        MissileClass::SamMedium => 6,
    };
    MissileClassToe {
        toe_count,
        lines: vec![
            ToeLine {
                tag: "missile_body".into(),
                qty: 1,
            },
            ToeLine {
                tag: "missile_seeker".into(),
                qty: 1,
            },
        ],
    }
}

fn parse_role(s: &str) -> Option<DroneRole> {
    match s {
        "FpvStrike" => Some(DroneRole::FpvStrike),
        "ReconLoiter" => Some(DroneRole::ReconLoiter),
        "AttritCruise" => Some(DroneRole::AttritCruise),
        "MedAttack" => Some(DroneRole::MedAttack),
        "Interceptor" => Some(DroneRole::Interceptor),
        "SwarmMember" => Some(DroneRole::SwarmMember),
        _ => None,
    }
}

fn parse_file(text: &str) -> Result<CapabilityToeRegistry, String> {
    let file: CapabilityToeFile = ron::de::from_str(text).map_err(|e| e.to_string())?;
    let mut drone = HashMap::new();
    for row in file.drone_roles {
        let Some(role) = parse_role(&row.role) else {
            continue;
        };
        drone.insert(
            role,
            DroneRoleToe {
                toe_count: row.toe_count.max(1),
                lines: row
                    .lines
                    .into_iter()
                    .map(|l| ToeLine {
                        tag: l.tag,
                        qty: l.qty.max(1),
                    })
                    .collect(),
            },
        );
    }
    let mut missile = HashMap::new();
    for row in file.missiles {
        let Some(class) = MissileClass::parse(&row.class) else {
            continue;
        };
        let toe_count = if row.toe_count == 0 {
            default_missile_toe(class).toe_count
        } else {
            row.toe_count.max(1)
        };
        let lines = if row.lines.is_empty() {
            default_missile_toe(class).lines
        } else {
            row.lines
                .into_iter()
                .map(|l| ToeLine {
                    tag: l.tag,
                    qty: l.qty.max(1),
                })
                .collect()
        };
        let _ = row.notes;
        missile.insert(class, MissileClassToe { toe_count, lines });
    }
    Ok(CapabilityToeRegistry {
        drone,
        missile,
        schema_version: file.schema_version,
        table_id: file.table_id,
    })
}

#[must_use]
pub fn load_capability_toe_registry() -> (CapabilityToeRegistry, Option<String>) {
    let path = PathBuf::from(CAPABILITY_TOE_RON);
    if path.is_file() {
        match std::fs::read_to_string(&path) {
            Ok(text) => match parse_file(&text) {
                Ok(reg) => return (reg, None),
                Err(e) => {
                    return (
                        parse_file(EMBEDDED).expect("embedded capability_toe"),
                        Some(format!("disk parse failed ({e}); using embedded")),
                    );
                }
            },
            Err(e) => {
                return (
                    parse_file(EMBEDDED).expect("embedded capability_toe"),
                    Some(format!("disk read failed ({e}); using embedded")),
                );
            }
        }
    }
    (
        parse_file(EMBEDDED).expect("embedded capability_toe"),
        Some(format!("missing {}; using embedded", path.display())),
    )
}

pub fn load_capability_toe_on_startup(mut commands: Commands) {
    let (reg, err) = load_capability_toe_registry();
    if let Some(err) = err {
        warn!("capability_toe registry: {err}");
    }
    commands.insert_resource(reg);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn embedded_covers_drones_and_missiles() {
        let reg = parse_file(EMBEDDED).unwrap();
        for role in DroneRole::ALL {
            let toe = reg.drone(role);
            assert!(toe.toe_count >= 1);
            assert!(toe.lines.iter().any(|l| l.tag == "drone_airframe"));
        }
        for class in MissileClass::ALL {
            let toe = reg.missile(class);
            assert!(toe.toe_count >= 1);
            assert!(toe.lines.iter().any(|l| l.tag == "missile_body"));
        }
        assert_eq!(reg.table_id, "capability_toe_v1");
    }
}
