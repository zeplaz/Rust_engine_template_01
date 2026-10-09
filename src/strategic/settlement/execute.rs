//! Growth proposal → construction pending queue (PROC-OG-APPROVE-001).

use bevy::prelude::*;

use crate::construction::{
    PendingBuildBlueprint, PendingConstructionQueue, PendingEntryKind,
};
use crate::strategic::{BuildSiteTile, FootprintTiles, LayerType, SiteArchetype};

use super::actors::{growth_actor_may_enqueue, BuildingUsage, GrowthActorLayer};
use super::growth::{GrowthProposal, GrowthProposalQueue};
use super::ids::ArchetypeId;

#[must_use]
pub fn site_archetype_for_growth_proposal(proposal: &GrowthProposal) -> SiteArchetype {
    match proposal.usage {
        BuildingUsage::Residential => SiteArchetype::CivilHousing,
        BuildingUsage::Industrial | BuildingUsage::Logistics => SiteArchetype::Factory,
        BuildingUsage::Office | BuildingUsage::Commercial => SiteArchetype::Factory,
        _ => SiteArchetype::Factory,
    }
}

#[must_use]
pub fn default_footprint_for_archetype(archetype: &ArchetypeId) -> FootprintTiles {
    if archetype.0.contains("warehouse") {
        FootprintTiles {
            width: 4,
            depth: 2,
        }
    } else {
        FootprintTiles {
            width: 1,
            depth: 1,
        }
    }
}

#[must_use]
pub fn pending_blueprint_from_growth_proposal(proposal: &GrowthProposal) -> PendingBuildBlueprint {
    let origin = BuildSiteTile {
        x: proposal.anchor_tile.x.max(0) as u32,
        z: proposal.anchor_tile.y.max(0) as u32,
    };
    PendingBuildBlueprint {
        kind: PendingEntryKind::BuildSite,
        label: format!("growth:{}", proposal.archetype_id.0),
        archetype: site_archetype_for_growth_proposal(proposal),
        origin,
        footprint: default_footprint_for_archetype(&proposal.archetype_id),
        layer: LayerType::Surface,
        rotation_quarter_turns: 0,
        mirror_x: false,
        approved: true,
        catalog_id: Some(proposal.archetype_id.0.clone()),
    }
}

pub fn enqueue_approved_growth_proposal(
    proposal: GrowthProposal,
    pending: &mut PendingConstructionQueue,
) -> bool {
    if !growth_actor_may_enqueue(proposal.actor_layer) {
        return false;
    }
    pending.push(pending_blueprint_from_growth_proposal(&proposal));
    true
}

pub fn approve_growth_proposal_into_pending(
    queue: &mut GrowthProposalQueue,
    index: usize,
    pending: &mut PendingConstructionQueue,
) -> bool {
    if index >= queue.proposals.len() {
        return false;
    }
    let proposal = queue.proposals.remove(index);
    enqueue_approved_growth_proposal(proposal, pending)
}

/// Highest-priority district proposal with no block id. Staged lots stay for rollout.
pub fn commit_top_district_growth(
    queue: &mut GrowthProposalQueue,
    pending: &mut PendingConstructionQueue,
) -> bool {
    let Some(idx) = queue
        .proposals
        .iter()
        .enumerate()
        .filter(|(_, proposal)| {
            proposal.block_id.is_none() && growth_actor_may_enqueue(proposal.actor_layer)
        })
        .max_by(|(_, a), (_, b)| {
            a.priority
                .partial_cmp(&b.priority)
                .unwrap_or(std::cmp::Ordering::Equal)
                .then(b.seed.cmp(&a.seed))
                .then_with(|| a.archetype_id.0.cmp(&b.archetype_id.0))
        })
        .map(|(idx, _)| idx)
    else {
        return false;
    };
    approve_growth_proposal_into_pending(queue, idx, pending)
}

/// Approved growth blueprints enter the same commit message as player and AI placement.
#[must_use]
pub fn growth_pending_commit_event(
    owner: Entity,
    entry: &PendingBuildBlueprint,
) -> Option<crate::strategic::CommitConstructionSiteEvent> {
    if !entry.approved || entry.kind != PendingEntryKind::BuildSite {
        return None;
    }
    if !entry.label.starts_with("growth:") {
        return None;
    }
    Some(crate::strategic::CommitConstructionSiteEvent {
        site_id: crate::strategic::SiteId::UNASSIGNED,
        owner,
        archetype: entry.archetype,
        origin: entry.origin,
        footprint: entry.footprint,
        layer: entry.layer,
        catalog_id: entry.catalog_id.clone(),
        placement: None,
    })
}

#[must_use]
pub fn archetype_for_zoning(zone: super::zoning::ZoningClass) -> &'static str {
    use super::zoning::ZoningClass;
    match zone {
        ZoningClass::Residential | ZoningClass::Rural => "house",
        ZoningClass::Industrial => "warehouse",
        ZoningClass::Commercial | ZoningClass::MixedUse => "corner_shop",
        ZoningClass::Civic => "civic_hall",
    }
}

#[must_use]
pub fn usage_for_zoning(zone: super::zoning::ZoningClass) -> BuildingUsage {
    use super::zoning::ZoningClass;
    match zone {
        ZoningClass::Residential | ZoningClass::Rural => BuildingUsage::Residential,
        ZoningClass::Industrial => BuildingUsage::Industrial,
        ZoningClass::Commercial | ZoningClass::MixedUse => BuildingUsage::Commercial,
        ZoningClass::Civic => BuildingUsage::Civic,
    }
}

pub fn growth_site_at_town_center(
    town: &super::town::TownRecord,
    district: &super::district::DistrictRecord,
) -> PendingBuildBlueprint {
        let proposal = GrowthProposal {
            district_id: district.id.clone(),
            block_id: None,
            archetype_id: ArchetypeId(archetype_for_zoning(district.zoning_default).into()),
            usage: usage_for_zoning(district.zoning_default),
            actor_layer: GrowthActorLayer::Growth,
            anchor_tile: town.center_tile,
        priority: 1.0,
        seed: town.center_tile.x.max(0) as u64,
        reason_codes: Vec::new(),
        saturation_at_submit: 0.0,
    };
    pending_blueprint_from_growth_proposal(&proposal)
}

/// One growth site per town that has a district. Sorted by town id.
#[must_use]
pub fn growth_sites_for_towns(
    towns: &super::town::TownBook,
    districts: &super::district::DistrictBook,
) -> Vec<PendingBuildBlueprint> {
    let mut ids: Vec<_> = towns.towns.keys().cloned().collect();
    ids.sort_by(|a, b| a.0.cmp(&b.0));
    let mut sites = Vec::new();
    for id in ids {
        let Some(town) = towns.towns.get(&id) else {
            continue;
        };
        let Some(district) = districts
            .districts
            .values()
            .find(|district| district.town_id == town.id)
        else {
            continue;
        };
        sites.push(growth_site_at_town_center(town, district));
    }
    sites
}

/// Pushes one growth site per town the first time it is called. Later calls add nothing.
/// Rows already in `pending` stay, including player blueprints.
pub fn enqueue_town_growth_once(
    towns: &super::town::TownBook,
    districts: &super::district::DistrictBook,
    pending: &mut PendingConstructionQueue,
    placed: &mut bool,
) -> usize {
    if *placed {
        return 0;
    }
    let sites = growth_sites_for_towns(towns, districts);
    let mut added = 0usize;
    for site in sites {
        let taken = pending.entries.iter().any(|entry| {
            entry.origin.x == site.origin.x && entry.origin.z == site.origin.z
        });
        if taken {
            continue;
        }
        pending.push(site);
        added += 1;
    }
    *placed = true;
    added
}

/// A second building sits two tiles east of the center. An origin already queued is skipped.
pub fn enqueue_second_town_growth(
    towns: &super::town::TownBook,
    districts: &super::district::DistrictBook,
    pending: &mut PendingConstructionQueue,
) -> usize {
    let mut ids: Vec<_> = towns.towns.keys().cloned().collect();
    ids.sort_by(|a, b| a.0.cmp(&b.0));
    let mut added = 0usize;
    for id in ids {
        let Some(town) = towns.towns.get(&id) else {
            continue;
        };
        let Some(district) = districts
            .districts
            .values()
            .find(|district| district.town_id == town.id)
        else {
            continue;
        };
        let mut site = growth_site_at_town_center(town, district);
        let ox = town.center_tile.x.saturating_add(2).max(0) as u32;
        let oz = town.center_tile.y.max(0) as u32;
        site.origin.x = ox;
        site.origin.z = oz;
        let taken = pending.entries.iter().any(|entry| {
            entry.origin.x == site.origin.x && entry.origin.z == site.origin.z
        });
        if taken {
            continue;
        }
        pending.push(site);
        added += 1;
    }
    added
}

#[derive(Resource, Default)]
pub struct TownGrowthEnqueueGate {
    pub placed: bool,
}

#[derive(Resource, Default)]
pub struct TownSecondGrowthGate {
    pub placed: bool,
}

pub fn enqueue_generated_town_growth_once_system(
    towns: Option<Res<super::town::TownBook>>,
    districts: Option<Res<super::district::DistrictBook>>,
    pending: Option<ResMut<PendingConstructionQueue>>,
    gate: Option<ResMut<TownGrowthEnqueueGate>>,
) {
    let (Some(towns), Some(districts), Some(mut pending), Some(mut gate)) =
        (towns, districts, pending, gate)
    else {
        return;
    };
    enqueue_town_growth_once(&towns, &districts, &mut pending, &mut gate.placed);
}

pub fn enqueue_second_town_growth_system(
    towns: Option<Res<super::town::TownBook>>,
    districts: Option<Res<super::district::DistrictBook>>,
    pending: Option<ResMut<PendingConstructionQueue>>,
    gate: Option<ResMut<TownSecondGrowthGate>>,
) {
    let (Some(towns), Some(districts), Some(mut pending), Some(mut gate)) =
        (towns, districts, pending, gate)
    else {
        return;
    };
    if gate.placed {
        return;
    }
    enqueue_second_town_growth(&towns, &districts, &mut pending);
    gate.placed = true;
}

pub fn commit_growth_pending_to_funnel_system(
    pending: Option<ResMut<PendingConstructionQueue>>,
    mut events: MessageWriter<crate::strategic::CommitConstructionSiteEvent>,
) {
    let Some(mut pending) = pending else {
        return;
    };
    let owner = Entity::PLACEHOLDER;
    let mut kept = Vec::new();
    for entry in pending.entries.drain(..) {
        if let Some(event) = growth_pending_commit_event(owner, &entry) {
            events.write(event);
        } else {
            kept.push(entry);
        }
    }
    pending.entries = kept;
}

pub fn approve_all_growth_proposals_into_pending(
    queue: &mut GrowthProposalQueue,
    pending: &mut PendingConstructionQueue,
) -> usize {
    let mut count = 0usize;
    while let Some(proposal) = queue.proposals.pop() {
        if enqueue_approved_growth_proposal(proposal, pending) {
            count += 1;
        }
    }
    count
}

#[must_use]
pub fn growth_approve_execute_pipeline_witness_green() -> bool {
    use super::actors::{BuildingUsage, GrowthActorLayer};
    use super::ids::DistrictId;

    let mut queue = GrowthProposalQueue::default();
    queue.enqueue(GrowthProposal {
        district_id: DistrictId("d".into()),
        block_id: None,
        archetype_id: ArchetypeId("corner_shop".into()),
        usage: BuildingUsage::Commercial,
        actor_layer: GrowthActorLayer::Growth,
        anchor_tile: IVec2::new(4, 4),
        priority: 0.5,
        seed: 1,
        reason_codes: Vec::new(),
        saturation_at_submit: 0.0,
    });
    let mut pending = PendingConstructionQueue::default();
    if !approve_growth_proposal_into_pending(&mut queue, 0, &mut pending) {
        return false;
    }
    queue.proposals.is_empty()
        && pending.entries.len() == 1
        && pending.entries[0].approved
        && pending.entries[0].kind == PendingEntryKind::BuildSite
        && pending.entries[0].catalog_id.as_deref() == Some("corner_shop")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn growth_approve_enqueues_pending_blueprint() {
        assert!(growth_approve_execute_pipeline_witness_green());
    }

    #[test]
    fn commit_top_district_growth_keeps_lower_priority_and_staged_lots() {
        use super::super::actors::{BuildingUsage, GrowthActorLayer};
        use super::super::ids::{ArchetypeId, BlockId, DistrictId};

        let mut queue = GrowthProposalQueue::default();
        queue.enqueue(GrowthProposal {
            district_id: DistrictId("d".into()),
            block_id: None,
            archetype_id: ArchetypeId("house".into()),
            usage: BuildingUsage::Residential,
            actor_layer: GrowthActorLayer::Growth,
            anchor_tile: IVec2::new(2, 2),
            priority: 0.2,
            seed: 2,
            reason_codes: Vec::new(),
            saturation_at_submit: 0.0,
        });
        queue.enqueue(GrowthProposal {
            district_id: DistrictId("d".into()),
            block_id: None,
            archetype_id: ArchetypeId("shop".into()),
            usage: BuildingUsage::Commercial,
            actor_layer: GrowthActorLayer::Growth,
            anchor_tile: IVec2::new(3, 3),
            priority: 0.8,
            seed: 1,
            reason_codes: Vec::new(),
            saturation_at_submit: 0.0,
        });
        queue.enqueue(GrowthProposal {
            district_id: DistrictId("d".into()),
            block_id: Some(BlockId("lot".into())),
            archetype_id: ArchetypeId("staged".into()),
            usage: BuildingUsage::Industrial,
            actor_layer: GrowthActorLayer::Growth,
            anchor_tile: IVec2::new(9, 9),
            priority: 0.99,
            seed: 9,
            reason_codes: Vec::new(),
            saturation_at_submit: 0.0,
        });
        let mut pending = PendingConstructionQueue::default();
        assert!(commit_top_district_growth(&mut queue, &mut pending));
        assert_eq!(pending.entries.len(), 1);
        assert_eq!(pending.entries[0].catalog_id.as_deref(), Some("shop"));
        assert_eq!(queue.proposals.len(), 2);
        assert!(queue.proposals.iter().any(|p| p.block_id.is_some()));
        assert!(queue.proposals.iter().any(|p| p.archetype_id.0 == "house"));
    }

    #[test]
    fn growth_pending_uses_commit_funnel() {
        use super::super::actors::{BuildingUsage, GrowthActorLayer};
        use super::super::ids::ArchetypeId;

        let owner = Entity::PLACEHOLDER;
        let pending = pending_blueprint_from_growth_proposal(&GrowthProposal {
            district_id: super::super::ids::DistrictId("d".into()),
            block_id: None,
            archetype_id: ArchetypeId("shop".into()),
            usage: BuildingUsage::Commercial,
            actor_layer: GrowthActorLayer::Growth,
            anchor_tile: IVec2::new(4, 4),
            priority: 0.5,
            seed: 1,
            reason_codes: Vec::new(),
            saturation_at_submit: 0.0,
        });
        let event = growth_pending_commit_event(owner, &pending).expect("growth commits");
        assert_eq!(event.catalog_id.as_deref(), Some("shop"));
        assert_eq!(event.origin.x, 4);
        let mut player = pending.clone();
        player.label = "player".into();
        assert!(growth_pending_commit_event(owner, &player).is_none());
    }

    #[test]
    fn generated_town_center_growth_uses_commit_funnel() {
        use super::super::district::generated_districts_for_towns;
        use super::super::town::generated_town_book;

        let towns = generated_town_book(99_001, 4);
        let districts = generated_districts_for_towns(&towns);
        let town = towns
            .towns
            .get(towns.default_town.as_ref().expect("default"))
            .expect("town");
        let district = districts
            .districts
            .values()
            .find(|d| d.town_id == town.id)
            .expect("district");
        let pending = growth_site_at_town_center(town, district);
        let event = growth_pending_commit_event(Entity::PLACEHOLDER, &pending).expect("commit");
        assert_eq!(event.origin.x, town.center_tile.x.max(0) as u32);
        assert_eq!(event.origin.z, town.center_tile.y.max(0) as u32);
        assert!(pending.label.starts_with("growth:"));
    }

    #[test]
    fn every_generated_town_gets_a_growth_site() {
        use super::super::district::generated_districts_for_towns;
        use super::super::town::generated_town_book;

        let towns = generated_town_book(99_001, 4);
        let districts = generated_districts_for_towns(&towns);
        let sites = growth_sites_for_towns(&towns, &districts);
        assert_eq!(sites.len(), 4);
        let mut origins = Vec::new();
        for site in &sites {
            let event = growth_pending_commit_event(Entity::PLACEHOLDER, site).expect("commit");
            origins.push((event.origin.x, event.origin.z));
        }
        origins.sort_unstable();
        origins.dedup();
        assert_eq!(origins.len(), 4);
    }

    #[test]
    fn town_growth_enqueues_once_and_keeps_player_rows() {
        use super::super::district::generated_districts_for_towns;
        use super::super::town::generated_town_book;

        let towns = generated_town_book(42, 4);
        let districts = generated_districts_for_towns(&towns);
        let mut pending = PendingConstructionQueue::default();
        let mut player = growth_sites_for_towns(&towns, &districts)[0].clone();
        player.label = "player".into();
        pending.push(player);
        let mut placed = false;
        assert_eq!(
            enqueue_town_growth_once(&towns, &districts, &mut pending, &mut placed),
            4
        );
        assert_eq!(
            enqueue_town_growth_once(&towns, &districts, &mut pending, &mut placed),
            0
        );
        assert_eq!(pending.entries.len(), 5);
        assert!(pending.entries.iter().any(|entry| entry.label == "player"));
    }

    #[test]
    fn zoning_picks_the_growth_archetype() {
        use super::super::district::generated_districts_for_towns;
        use super::super::town::generated_town_book;
        use super::super::zoning::ZoningClass;

        let towns = generated_town_book(7, 1);
        let mut districts = generated_districts_for_towns(&towns);
        let town = towns.towns.values().next().expect("town");
        let district = districts.districts.values_mut().next().expect("district");
        district.zoning_default = ZoningClass::Industrial;
        let site = growth_site_at_town_center(town, district);
        assert_eq!(site.catalog_id.as_deref(), Some("warehouse"));
        district.zoning_default = ZoningClass::Residential;
        let house = growth_site_at_town_center(town, district);
        assert_eq!(house.catalog_id.as_deref(), Some("house"));
    }

    #[test]
    fn civic_zoning_queues_a_hall() {
        use super::super::district::generated_districts_for_towns;
        use super::super::town::generated_town_book;
        use super::super::zoning::ZoningClass;

        let towns = generated_town_book(9, 1);
        let mut districts = generated_districts_for_towns(&towns);
        let town = towns.towns.values().next().expect("town");
        let district = districts.districts.values_mut().next().expect("district");
        district.zoning_default = ZoningClass::Civic;
        let hall = growth_site_at_town_center(town, district);
        district.zoning_default = ZoningClass::Commercial;
        let shop = growth_site_at_town_center(town, district);
        assert_eq!(hall.catalog_id.as_deref(), Some("civic_hall"));
        assert_eq!(shop.catalog_id.as_deref(), Some("corner_shop"));
    }

    #[test]
    fn second_growth_tick_uses_a_new_origin_and_does_not_repeat() {
        use super::super::district::generated_districts_for_towns;
        use super::super::town::generated_town_book;

        let towns = generated_town_book(11, 2);
        let districts = generated_districts_for_towns(&towns);
        let mut pending = PendingConstructionQueue::default();
        let mut placed = false;
        assert_eq!(enqueue_town_growth_once(&towns, &districts, &mut pending, &mut placed), 2);
        let first = pending.entries.len();
        assert_eq!(enqueue_second_town_growth(&towns, &districts, &mut pending), 2);
        assert_eq!(enqueue_second_town_growth(&towns, &districts, &mut pending), 0);
        assert_eq!(pending.entries.len(), first + 2);
        let mut origins = pending
            .entries
            .iter()
            .map(|entry| (entry.origin.x, entry.origin.z))
            .collect::<Vec<_>>();
        origins.sort_unstable();
        origins.dedup();
        assert_eq!(origins.len(), pending.entries.len());
    }

    fn labeled_growth(label: &str) -> PendingBuildBlueprint {
        use super::super::actors::{BuildingUsage, GrowthActorLayer};
        use super::super::ids::{ArchetypeId, DistrictId};

        let mut row = pending_blueprint_from_growth_proposal(&GrowthProposal {
            district_id: DistrictId("d".into()),
            block_id: None,
            archetype_id: ArchetypeId("shop".into()),
            usage: BuildingUsage::Commercial,
            actor_layer: GrowthActorLayer::Growth,
            anchor_tile: IVec2::new(4, 4),
            priority: 0.5,
            seed: 1,
            reason_codes: Vec::new(),
            saturation_at_submit: 0.0,
        });
        row.label = label.into();
        row
    }

    fn labels_after_funnel(label: &str) -> Vec<String> {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins)
            .init_resource::<PendingConstructionQueue>()
            .add_message::<crate::strategic::CommitConstructionSiteEvent>()
            .add_systems(Update, commit_growth_pending_to_funnel_system);
        app.world_mut()
            .resource_mut::<PendingConstructionQueue>()
            .push(labeled_growth(label));
        app.update();
        app.world()
            .resource::<PendingConstructionQueue>()
            .entries
            .iter()
            .map(|entry| entry.label.clone())
            .collect()
    }

    #[test]
    fn over_bare_growth_prefix_stays_queued() {
        let kept = labels_after_funnel("growth");
        assert_eq!(kept, vec!["growth".to_string()]);
        assert!(growth_pending_commit_event(Entity::PLACEHOLDER, &labeled_growth("growth")).is_none());
    }

    #[test]
    fn over_capital_growth_label_is_not_a_funnel_row() {
        let kept = labels_after_funnel("Growth:");
        assert_eq!(kept, vec!["Growth:".to_string()]);
        assert!(
            growth_pending_commit_event(Entity::PLACEHOLDER, &labeled_growth("Growth:shop")).is_none()
        );
    }
}
