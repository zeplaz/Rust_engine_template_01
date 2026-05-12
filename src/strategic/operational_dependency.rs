//! L2 **operational dependency** edges for developmental UX — upstream tracing + supply anchors.
//!
//! Dependants hold [`OperationalDependencyLink`] → immediate upstream entity. Upstream may be a
//! [`OperationalSupplyAnchor`] (logical grid / corridor / pool) or future real infrastructure entities.

use bevy::prelude::*;

use crate::strategic::{
    ConstructionSite, SiteConstructionPhase, SiteNetworkAttachment, SiteOperationalStats,
};

/// Kind of dependency (interpreted for players — not raw sim names).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum OperationalDependencyKind {
    PowerDelivery,
    LogisticsCorridor,
    FuelSupply,
    WaterHeadworks,
    CommandAndStaff,
}

impl OperationalDependencyKind {
    #[inline]
    pub const fn operational_label(self) -> &'static str {
        match self {
            Self::PowerDelivery => "power delivery",
            Self::LogisticsCorridor => "logistics corridor",
            Self::FuelSupply => "fuel supply",
            Self::WaterHeadworks => "water headworks",
            Self::CommandAndStaff => "command & workforce",
        }
    }
}

/// Marks a logical **supply root** (player-readable label via [`Name`]).
#[derive(Component, Clone, Copy, Debug)]
pub struct OperationalSupplyAnchor {
    pub kind: OperationalDependencyKind,
}

/// One hop toward an upstream supplier / grid segment.
#[derive(Component, Clone, Debug)]
pub struct OperationalDependencyLink {
    pub upstream: Entity,
    pub kind: OperationalDependencyKind,
}

/// Spawned once at startup — canonical upstream entities for stub tracing.
#[derive(Resource, Clone, Debug)]
pub struct OperationalCausalityAnchors {
    pub power_grid: Entity,
    pub logistics_spine: Entity,
    pub fuel_terminal: Entity,
    pub water_headworks: Entity,
    pub command_staff_pool: Entity,
}

pub fn startup_spawn_operational_causality_anchors(mut commands: Commands) {
    let power_grid = commands
        .spawn((
            OperationalSupplyAnchor {
                kind: OperationalDependencyKind::PowerDelivery,
            },
            Name::new("Regional power grid"),
        ))
        .id();
    let logistics_spine = commands
        .spawn((
            OperationalSupplyAnchor {
                kind: OperationalDependencyKind::LogisticsCorridor,
            },
            Name::new("Strategic rail / road spine"),
        ))
        .id();
    let fuel_terminal = commands
        .spawn((
            OperationalSupplyAnchor {
                kind: OperationalDependencyKind::FuelSupply,
            },
            Name::new("Bulk fuel transfer node"),
        ))
        .id();
    let water_headworks = commands
        .spawn((
            OperationalSupplyAnchor {
                kind: OperationalDependencyKind::WaterHeadworks,
            },
            Name::new("Regional water headworks"),
        ))
        .id();
    let command_staff_pool = commands
        .spawn((
            OperationalSupplyAnchor {
                kind: OperationalDependencyKind::CommandAndStaff,
            },
            Name::new("Command & labor pool"),
        ))
        .id();

    commands.insert_resource(OperationalCausalityAnchors {
        power_grid,
        logistics_spine,
        fuel_terminal,
        water_headworks,
        command_staff_pool,
    });
}

const STRESS_THRESHOLD: f32 = 0.85;

#[inline]
fn weakest_operational_kind(stats: &SiteOperationalStats) -> OperationalDependencyKind {
    let mut worst_k = OperationalDependencyKind::PowerDelivery;
    let mut worst_v = stats.power_ratio;
    if stats.supply_ratio < worst_v {
        worst_v = stats.supply_ratio;
        worst_k = OperationalDependencyKind::LogisticsCorridor;
    }
    if stats.workforce_ratio < worst_v {
        worst_k = OperationalDependencyKind::CommandAndStaff;
    }
    let _ = worst_v;
    worst_k
}

#[inline]
fn anchor_for_kind(anchors: &OperationalCausalityAnchors, kind: OperationalDependencyKind) -> Entity {
    match kind {
        OperationalDependencyKind::PowerDelivery => anchors.power_grid,
        OperationalDependencyKind::LogisticsCorridor => anchors.logistics_spine,
        OperationalDependencyKind::FuelSupply => anchors.fuel_terminal,
        OperationalDependencyKind::WaterHeadworks => anchors.water_headworks,
        OperationalDependencyKind::CommandAndStaff => anchors.command_staff_pool,
    }
}

pub fn sync_site_operational_dependency_links_apply_system(
    anchors: Option<Res<OperationalCausalityAnchors>>,
    mut commands: Commands,
    sites: Query<
        (
            Entity,
            &ConstructionSite,
            &SiteOperationalStats,
            Option<&OperationalDependencyLink>,
        ),
        With<SiteOperationalStats>,
    >,
) {
    let Some(anchors) = anchors.as_ref().map(|r| r.as_ref()) else {
        return;
    };

    for (entity, site, stats, existing) in &sites {
        let phase_ok = matches!(
            site.phase,
            SiteConstructionPhase::Provisioning | SiteConstructionPhase::Operational
        );
        let stressed = stats.power_ratio < STRESS_THRESHOLD
            || stats.supply_ratio < STRESS_THRESHOLD
            || stats.workforce_ratio < STRESS_THRESHOLD;

        if !phase_ok || !stressed {
            if existing.is_some() {
                commands.entity(entity).remove::<OperationalDependencyLink>();
            }
            continue;
        }

        let kind = weakest_operational_kind(stats);
        let upstream = anchor_for_kind(anchors, kind);

        if let Some(link) = existing {
            if link.upstream == upstream && link.kind == kind {
                continue;
            }
        }
        commands.entity(entity).insert(OperationalDependencyLink {
            upstream,
            kind,
        });
    }
}

/// Multi-factor “story” when several buses sag together (no extra ECS hops).
pub fn composite_operational_stress_note(
    stats: &SiteOperationalStats,
    _net: Option<&SiteNetworkAttachment>,
) -> Option<String> {
    let p = stats.power_ratio < STRESS_THRESHOLD;
    let s = stats.supply_ratio < STRESS_THRESHOLD;
    let w = stats.workforce_ratio < STRESS_THRESHOLD;
    match (p, s, w) {
        (true, true, _) => Some("Composite stress: power sag is choking corridor clearances.".into()),
        (true, _, true) => Some("Composite stress: grid + staffing both short — recovery will be slow.".into()),
        (_, true, true) => Some("Composite stress: logistics + staffing — convoys idle without crews.".into()),
        _ => None,
    }
}

/// Walk [`OperationalDependencyLink`] up to `max_hops`, collecting operational labels.
pub fn trace_operational_cause_chain(world: &World, leaf: Entity, max_hops: usize) -> String {
    let mut parts = Vec::new();
    let mut cursor = leaf;
    for _ in 0..max_hops {
        let Some(link) = world.entity(cursor).get::<OperationalDependencyLink>() else {
            break;
        };
        let hop = format!("{} ← {}", link.kind.operational_label(), upstream_display(world, link.upstream));
        parts.push(hop);
        cursor = link.upstream;
        if world.entity(cursor).get::<OperationalSupplyAnchor>().is_some() {
            break;
        }
    }
    if parts.is_empty() {
        return String::new();
    }
    parts.join(" :: ")
}

#[inline]
fn upstream_display(world: &World, ent: Entity) -> String {
    if let Some(name) = world.entity(ent).get::<Name>() {
        return name.as_str().to_string();
    }
    if let Some(a) = world.entity(ent).get::<OperationalSupplyAnchor>() {
        return format!("{} (anchor)", a.kind.operational_label());
    }
    format!("{ent:?}")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::strategic::{ConstructionSite, SiteArchetype};
    use bevy::ecs::system::RunSystemOnce;

    #[test]
    fn stress_threshold_inserts_link_via_apply_system() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.add_systems(Startup, startup_spawn_operational_causality_anchors);
        app.update();

        let anchors = app.world().resource::<OperationalCausalityAnchors>().clone();
        let site_ent = app
            .world_mut()
            .spawn((
                ConstructionSite {
                    site_id: 1,
                    owner: Entity::PLACEHOLDER,
                    archetype: SiteArchetype::Factory,
                    phase: SiteConstructionPhase::Provisioning,
                    operational_readiness: 0.0,
                },
                SiteOperationalStats {
                    power_ratio: 0.2,
                    supply_ratio: 0.9,
                    workforce_ratio: 0.9,
                    integrity: 1.0,
                },
            ))
            .id();

        let _ = app.world_mut().run_system_once(sync_site_operational_dependency_links_apply_system);
        let link = app
            .world()
            .entity(site_ent)
            .get::<OperationalDependencyLink>()
            .expect("link");
        assert_eq!(link.upstream, anchors.power_grid);
        assert_eq!(link.kind, OperationalDependencyKind::PowerDelivery);
        let line = trace_operational_cause_chain(app.world(), site_ent, 4);
        assert!(line.contains("power"));
        assert!(line.contains("grid") || line.contains("Regional"));
    }
}
