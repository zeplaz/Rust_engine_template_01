//! Construction sites as draw-demand consumers (plan §4, LO-8).
//!
//! A site under construction publishes one draw demand per material its
//! [`SiteResourceManifest`] still needs (`Concrete`, `Steel`, `Fuel`, `Machinery`). Materials on
//! hand at the site *are* the delivered amount: [`sync_construction_site_demands_system`] is the
//! sole production writer of `SiteResourceManifest::delivered_*`. When the site leaves
//! `UnderConstruction` its demands are retracted and the materials are consumed into the building.

use std::collections::BTreeMap;

use bevy::prelude::*;

use crate::strategic::{ConstructionSite, SiteConstructionPhase, SiteResourceManifest};

use super::model::{CargoKind, ConsumerKind, OwnerId, Priority, SiteId};
use super::plugin::{LogisticsOrderRequest, LogisticsOrderSite, LogisticsOrderSiteId};
use super::sim::LogisticsOrderCommand;
use super::snapshot::{LogisticsOrdersSnapshot, SiteRole};
use super::plugin::LogisticsOrdersSnapshotRes;

/// Manifest materials and their cargo tags. Same vocabulary as the industrial catalog
/// (`produces_resources`), so a plant making `Concrete` supplies the sites that need `Concrete`.
pub const CONSTRUCTION_MATERIALS: [&str; 4] = ["Concrete", "Steel", "Fuel", "Machinery"];

/// `(required, delivered)` for one material of a manifest.
#[must_use]
pub fn manifest_material(m: &SiteResourceManifest, tag: &str) -> (f32, f32) {
    match tag {
        "Concrete" => (m.concrete_required, m.delivered_concrete),
        "Steel" => (m.steel_required, m.delivered_steel),
        "Fuel" => (m.fuel_required, m.delivered_fuel),
        "Machinery" => (m.machinery_required, m.delivered_machinery),
        _ => (0.0, 0.0),
    }
}

/// Units of `tag` built into the building so far (construction writes, logistics applies).
#[must_use]
pub fn manifest_consumed(m: &SiteResourceManifest, tag: &str) -> f32 {
    match tag {
        "Concrete" => m.consumed_concrete,
        "Steel" => m.consumed_steel,
        "Fuel" => m.consumed_fuel,
        "Machinery" => m.consumed_machinery,
        _ => 0.0,
    }
}

fn set_delivered(m: &mut SiteResourceManifest, tag: &str, value: f32) {
    match tag {
        "Concrete" => m.delivered_concrete = value,
        "Steel" => m.delivered_steel = value,
        "Fuel" => m.delivered_fuel = value,
        "Machinery" => m.delivered_machinery = value,
        _ => {}
    }
}

/// Phases in which a site still accepts material deliveries.
#[must_use]
pub fn site_accepts_materials(phase: SiteConstructionPhase) -> bool {
    matches!(
        phase,
        SiteConstructionPhase::Planned
            | SiteConstructionPhase::Surveying
            | SiteConstructionPhase::Clearing
            | SiteConstructionPhase::Foundation
            | SiteConstructionPhase::UnderConstruction
    )
}

/// Demands to publish for a manifest: `(cargo, whole units required)` for materials with a bill.
#[must_use]
pub fn construction_demand_targets(m: &SiteResourceManifest) -> Vec<(&'static str, u32)> {
    CONSTRUCTION_MATERIALS
        .iter()
        .filter_map(|&tag| {
            let (required, _) = manifest_material(m, tag);
            (required > 0.0).then(|| (tag, required.ceil() as u32))
        })
        .collect()
}

fn on_hand(snap: &LogisticsOrdersSnapshot, site: SiteId, tag: &str) -> u32 {
    snap.site(site)
        .and_then(|s| s.stock.iter().find(|r| r.cargo == tag))
        .map_or(0, |r| r.on_hand)
}

/// Make sites that still need materials logistics endpoints.
#[allow(clippy::type_complexity)]
pub fn tag_construction_sites_as_order_sites(
    mut commands: Commands,
    sites: Query<
        (Entity, &ConstructionSite, &SiteResourceManifest),
        (With<Transform>, Without<LogisticsOrderSite>),
    >,
    slots: Query<&super::plugin::FactionSlot>,
) {
    for (entity, site, manifest) in &sites {
        if !site_accepts_materials(site.phase) || construction_demand_targets(manifest).is_empty() {
            continue;
        }
        commands.entity(entity).insert(LogisticsOrderSite {
            name: format!("Site {}", site.site_id),
            // The site's owner entity carries its faction slot; no slot means the player.
            owner: slots.get(site.owner).map_or(OwnerId::PLAYER, |s| OwnerId(s.0)),
            role: SiteRole::Construction,
            position_m: None,
        });
    }
}

/// What this system last published per site, so it only sends changes.
#[derive(Resource, Debug, Default)]
pub struct ConstructionDemandBook {
    pub published: BTreeMap<SiteId, Vec<(&'static str, u32)>>,
    /// Units already removed from each site's stock as built-in material.
    pub consumed_applied: BTreeMap<(SiteId, &'static str), u32>,
}

/// Publish / retract construction demands and mirror on-site stock into the manifest.
pub fn sync_construction_site_demands_system(
    snapshot: Res<LogisticsOrdersSnapshotRes>,
    mut book: ResMut<ConstructionDemandBook>,
    mut sites: Query<(&LogisticsOrderSiteId, &ConstructionSite, &mut SiteResourceManifest)>,
    mut requests: MessageWriter<LogisticsOrderRequest>,
) {
    let snap = &snapshot.0;
    let mut live: Vec<SiteId> = Vec::new();
    for (sid, site, mut manifest) in &mut sites {
        let id = sid.0;
        if !site_accepts_materials(site.phase) {
            continue;
        }
        live.push(id);
        let targets = construction_demand_targets(&manifest);
        for &(tag, required) in &targets {
            // Delivered is cumulative: stock still on site plus what was built in already.
            let applied = book.consumed_applied.get(&(id, tag)).copied().unwrap_or(0);
            let on_site = on_hand(snap, id, tag);
            let delivered = (on_site + applied).min(required) as f32;
            if (manifest_material(&manifest, tag).1 - delivered).abs() > f32::EPSILON {
                set_delivered(&mut manifest, tag, delivered);
            }
            // Progressive consumption: remove what the stage tick has built in so far.
            let want = manifest_consumed(&manifest, tag).ceil() as u32;
            let delta = want.saturating_sub(applied).min(on_site);
            if delta > 0 {
                requests.write(LogisticsOrderRequest(LogisticsOrderCommand::ConsumeStock {
                    site: id,
                    cargo: CargoKind::new(tag),
                    amount: delta,
                }));
                book.consumed_applied.insert((id, tag), applied + delta);
            }
        }
        if book.published.get(&id) != Some(&targets) {
            for &(tag, required) in &targets {
                requests.write(LogisticsOrderRequest(LogisticsOrderCommand::PublishDemand {
                    site: id,
                    cargo: CargoKind::new(tag),
                    consumer: ConsumerKind::Construction,
                    buffer_target: required,
                    reorder_point: required,
                    draw_rate_per_min: 0,
                    priority: Priority::Normal,
                }));
            }
            book.published.insert(id, targets);
        }
    }
    // Sites that finished building (or vanished): retract and consume the materials.
    let done: Vec<SiteId> = book
        .published
        .keys()
        .copied()
        .filter(|id| !live.contains(id))
        .collect();
    for id in done {
        let targets = book.published.remove(&id).unwrap_or_default();
        book.consumed_applied.retain(|(s, _), _| *s != id);
        if snap.site(id).is_none() {
            continue;
        }
        // The building is done: close supply orders still pointed at it.
        for o in snap.orders.iter().filter(|o| o.sink == id && !o.status.is_terminal()) {
            requests.write(LogisticsOrderRequest(LogisticsOrderCommand::Cancel { order: o.id }));
        }
        for (tag, _) in targets {
            requests.write(LogisticsOrderRequest(LogisticsOrderCommand::RetractDemand {
                site: id,
                cargo: CargoKind::new(tag),
            }));
            requests.write(LogisticsOrderRequest(LogisticsOrderCommand::SetStock {
                site: id,
                cargo: CargoKind::new(tag),
                on_hand: 0,
            }));
        }
    }
}
