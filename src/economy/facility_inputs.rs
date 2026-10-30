//! Workforce and power inputs for operational facilities.
//!
//! `Labour` and `Electricity` are not stockpiled goods, so they never sit in a flow-node buffer.
//! Instead each facility carries a [`FacilityInputs`] ratio per input, and
//! `propagate_resource_flow_system` scales output by the scarcer of the two.
//!
//! - **Workforce** comes from the settlement authority ([`TownBook`]): a facility draws
//!   `workers_required` (catalog) from the nearest town within [`COMMUTE_RANGE_TILES`]; a town's
//!   industrial labour pool is [`INDUSTRIAL_LABOUR_SHARE`] of its population minus the jobs it
//!   already has (never below a quarter of the pool). Facilities that share a town share its pool.
//! - **Power** comes from the utility authority: [`UtilityConnection`] (graph reachability to a
//!   power plant, written by `sync_utility_connection_power_system`) and the electrical grid the
//!   facility is a member of (`total_capacity / total_load` when overloaded).
//!
//! A model that is not loaded does not constrain: no towns in the book means workforce is
//! unmodelled (ratio 1), and no power graph means power is unmodelled (ratio 1).

use bevy::prelude::*;

use crate::construction::BuildingDefinitionRegistry;
use crate::economy::resource_flow::ResourceFlowNode;
use crate::entities::production::power::ElectricalGrid;
use crate::infrastructure::utility::UtilityGraph;
use crate::infrastructure::{UtilityConnection, UtilityNetworkKind};
use crate::strategic::{TownBook, TownRecord};

/// Tiles a worker commutes to a facility.
pub const COMMUTE_RANGE_TILES: f32 = 64.0;
/// Share of a town's population available to industry before existing jobs are subtracted.
pub const INDUSTRIAL_LABOUR_SHARE: f32 = 0.3;
const METRES_PER_TILE: f32 = 16.0;

/// Where a facility's power stands, for the info panel.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum PowerState {
    /// No power graph or no utility connection is modelled: not a constraint.
    #[default]
    Unmodelled,
    /// Not reachable from any power plant.
    Disconnected,
    Connected,
    /// On a grid whose load exceeds its capacity.
    Overloaded,
}

/// Per-facility labour and power supply, each `0..=1` (1 = fully supplied / not modelled).
#[derive(Component, Clone, Debug, PartialEq)]
pub struct FacilityInputs {
    pub workforce: f32,
    pub power: f32,
    /// The town the workers commute from and its distance in metres (`None`: unmodelled or no
    /// town within commute range).
    pub town: Option<(String, f32)>,
    pub power_state: PowerState,
}

impl Default for FacilityInputs {
    fn default() -> Self {
        Self {
            workforce: 1.0,
            power: 1.0,
            town: None,
            power_state: PowerState::Unmodelled,
        }
    }
}

impl FacilityInputs {
    /// Output multiplier: the scarcer input limits the plant.
    #[must_use]
    pub fn factor(&self) -> f32 {
        self.workforce.min(self.power).clamp(0.0, 1.0)
    }
}

/// Tags that are inputs-by-supply (workforce, grid) rather than hauled or buffered goods.
#[must_use]
pub fn is_supply_input_tag(tag: &str) -> bool {
    matches!(
        tag.to_ascii_lowercase().as_str(),
        "labour" | "electricity" | "energy" | "knowledge"
    )
}

fn town_labour_pool(population: u32, jobs: u32) -> f32 {
    let pool = population as f32 * INDUSTRIAL_LABOUR_SHARE;
    (pool - jobs as f32).max(pool * 0.25)
}

fn nearest_town<'a>(towns: &'a TownBook, at: Option<&Transform>) -> Option<&'a TownRecord> {
    let t = at?;
    let tile = Vec2::new(t.translation.x, t.translation.z) / METRES_PER_TILE;
    towns
        .towns
        .values()
        .map(|town| (town, town.center_tile.as_vec2().distance(tile)))
        .filter(|(_, d)| *d <= COMMUTE_RANGE_TILES)
        .min_by(|a, b| a.1.total_cmp(&b.1).then(a.0.id.0.cmp(&b.0.id.0)))
        .map(|(town, _)| town)
}

/// Refresh every flow node's [`FacilityInputs`] from the settlement and utility authorities.
#[allow(clippy::type_complexity)]
pub fn update_facility_inputs_system(
    mut commands: Commands,
    towns: Option<Res<TownBook>>,
    utility: Option<Res<UtilityGraph>>,
    registry: Option<Res<BuildingDefinitionRegistry>>,
    grids: Query<&ElectricalGrid>,
    mut nodes: Query<(
        Entity,
        &ResourceFlowNode,
        Option<&Transform>,
        Option<&UtilityConnection>,
        Option<&mut FacilityInputs>,
    )>,
) {
    let towns = towns.as_deref().filter(|t| !t.towns.is_empty());
    let workers_of = |catalog_id: &str| -> f32 {
        registry
            .as_deref()
            .and_then(|r| r.get(catalog_id))
            .map_or(0.0, |d| d.workers_required as f32)
    };

    // Workforce demand per nearest in-range town (facilities share their town's pool).
    let mut demand_by_town: Vec<(String, f32)> = Vec::new();
    if let Some(book) = towns {
        for (_, node, t, _, _) in &nodes {
            if let Some(town) = nearest_town(book, t) {
                let w = workers_of(&node.catalog_id);
                match demand_by_town.iter_mut().find(|(id, _)| *id == town.id.0) {
                    Some((_, d)) => *d += w,
                    None => demand_by_town.push((town.id.0.clone(), w)),
                }
            }
        }
    }
    let power_modelled = utility.as_deref().is_some_and(|u| !u.nodes.is_empty());

    for (entity, node, t, conn, current) in &mut nodes {
        let workers = workers_of(&node.catalog_id);
        let town = towns.and_then(|book| nearest_town(book, t)).map(|town| {
            let at = t.map_or(Vec2::ZERO, |t| Vec2::new(t.translation.x, t.translation.z));
            (town.name.clone(), (town.center_tile.as_vec2() * METRES_PER_TILE).distance(at))
        });
        let workforce = match towns {
            None => 1.0,
            Some(_) if workers <= 0.0 => 1.0,
            Some(book) => nearest_town(book, t).map_or(0.0, |town| {
                let demand = demand_by_town
                    .iter()
                    .find(|(id, _)| *id == town.id.0)
                    .map_or(workers, |(_, d)| *d)
                    .max(workers);
                (town_labour_pool(town.population, town.jobs) / demand).clamp(0.0, 1.0)
            }),
        };
        let (power, power_state) = if !power_modelled {
            (1.0, PowerState::Unmodelled)
        } else {
            match conn.filter(|c| c.kind == UtilityNetworkKind::Power) {
                None => (1.0, PowerState::Unmodelled),
                Some(c) if !c.connected => (0.0, PowerState::Disconnected),
                Some(_) => {
                    let ratio = grids
                        .iter()
                        .filter(|g| g.members.contains(&entity) && g.total_load > f32::EPSILON)
                        .map(|g| (g.total_capacity / g.total_load).clamp(0.0, 1.0))
                        .fold(1.0, f32::min);
                    (ratio, if ratio < 1.0 { PowerState::Overloaded } else { PowerState::Connected })
                }
            }
        };
        let next = FacilityInputs { workforce, power, town, power_state };
        match current {
            Some(mut c) => {
                if *c != next {
                    *c = next;
                }
            }
            None => {
                commands.entity(entity).insert(next);
            }
        }
    }
}
