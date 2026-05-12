//! L2 cause chain row — logistics focus + [`OperationalDependencyLink`](crate::strategic::OperationalDependencyLink).

use bevy::prelude::*;

use crate::gui::logistics_focus::HudLogisticsFocus;
use crate::strategic::{
    composite_operational_stress_note, ConstructionSite, OperationalDependencyLink,
    OperationalSupplyAnchor, SiteConstructionPhase, SiteNetworkAttachment, SiteOperationalStats,
};

/// Parent node for the developmental **CAUSE** strip (toggles visibility when empty).
#[derive(Component)]
pub struct DevelopmentalCauseStripRoot;

#[derive(Component)]
pub struct DevelopmentalCauseStripLine;

fn walk_dependency_hops(
    start: Entity,
    links: &Query<&OperationalDependencyLink>,
    names: &Query<&Name>,
    anchors: &Query<&OperationalSupplyAnchor>,
) -> String {
    let mut parts = Vec::new();
    let mut cursor = start;
    for _ in 0..5 {
        let Ok(link) = links.get(cursor) else {
            break;
        };
        let up_label = names
            .get(link.upstream)
            .map(|n| n.as_str().to_string())
            .unwrap_or_else(|_| "upstream supply".into());
        parts.push(format!("{} ← {}", link.kind.operational_label(), up_label));
        cursor = link.upstream;
        if anchors.get(cursor).is_ok() {
            break;
        }
    }
    parts.join(" → ")
}

pub fn update_developmental_cause_strip_system(
    focus: Res<HudLogisticsFocus>,
    sites: Query<
        (
            Entity,
            &ConstructionSite,
            &SiteOperationalStats,
            Option<&SiteNetworkAttachment>,
        ),
        With<ConstructionSite>,
    >,
    links: Query<&OperationalDependencyLink>,
    names: Query<&Name>,
    anchors: Query<&OperationalSupplyAnchor>,
    mut text_q: Query<&mut Text, With<DevelopmentalCauseStripLine>>,
    mut vis_q: Query<&mut Visibility, With<DevelopmentalCauseStripRoot>>,
) {
    let Some(tracked) = focus.tracked_entity else {
        for mut v in &mut vis_q {
            *v = Visibility::Visible;
        }
        for mut t in &mut text_q {
            *t = Text::new(
                "CAUSE — Pick or cycle logistics focus (F9) to read live dependency chains on a site.",
            );
        }
        return;
    };

    let row = sites.iter().find(|(e, ..)| *e == tracked);
    let Some((_, site, stats, net)) = row else {
        for mut v in &mut vis_q {
            *v = Visibility::Visible;
        }
        for mut t in &mut text_q {
            *t = Text::new(
                "CAUSE — Focus is not a territorial site — switch to a hub row or map-picked depot.",
            );
        }
        return;
    };

    let chain = walk_dependency_hops(tracked, &links, &names, &anchors);
    let composite = composite_operational_stress_note(stats, net);

    let mut line = String::new();

    if matches!(
        site.phase,
        SiteConstructionPhase::Planned
            | SiteConstructionPhase::Surveying
            | SiteConstructionPhase::Clearing
            | SiteConstructionPhase::Foundation
            | SiteConstructionPhase::UnderConstruction
            | SiteConstructionPhase::Abandoned
    ) {
        line.push_str(
            "CAUSE — Site still building — provisioning chain activates after construction.",
        );
    } else if matches!(
        site.phase,
        SiteConstructionPhase::Damaged | SiteConstructionPhase::Offline
    ) {
        line.push_str("CAUSE — Site impaired — ");
        if !chain.is_empty() {
            line.push_str(&chain);
        } else {
            line.push_str("check power, corridors, and crew pools.");
        }
        if let Some(c) = composite {
            line.push_str(" · ");
            line.push_str(&c);
        }
    } else if chain.is_empty() && composite.is_none() {
        line.push_str("CAUSE — No upstream stress registered on this site (thresholds clean).");
    } else {
        line.push_str("CAUSE CHAIN — ");
        if !chain.is_empty() {
            line.push_str(&chain);
        }
        if let Some(c) = composite {
            if !chain.is_empty() {
                line.push_str(" · ");
            }
            line.push_str(&c);
        }
    }

    let show = Visibility::Visible;
    for mut v in &mut vis_q {
        *v = show;
    }
    for mut t in &mut text_q {
        *t = Text::new(line.clone());
    }
}
