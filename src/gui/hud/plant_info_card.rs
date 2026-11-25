//! Site / plant info card (Bevy UI): for the focused site, shows build progress and the material
//! bill while it is being built, and once operational its production rate, workforce and power
//! (with the town / grid they come from), input and output buffers and every reason output is
//! held back. Presentation only: the text comes from `economy::facility_report`.
//!
//! Focus: the logistics focus (`HudLogisticsFocus`: map pick or the F9 cycle).
//!
//! One card per focused site: a dam's operations section is a child of this card
//! (`dam_ops_card`), not a second card on top of it. The card moves by dragging it anywhere and
//! reopens where the player left it ([`crate::gui::hud::panel_drag::HudCardPlacements`]).

use bevy::prelude::*;
use bevy::ui::FocusPolicy;

use crate::gui::hud::panel_drag::HudCardPlacements;

use crate::economy::activation::BuildingDefinitionRef;
use crate::economy::facility_inputs::FacilityInputs;
use crate::economy::facility_report::facility_report_lines;
use crate::economy::resource_flow::{FacilityFlowState, ResourceFlowNode};
use crate::gui::logistics_focus::HudLogisticsFocus;
use crate::gui::style::{CmdUiMonoFont, UiPalette};
use crate::construction::SiteStageProgress;
use crate::scenario::dam_flood::DamFloodPlayerOwner;
use crate::strategic::{ConstructionSite, SiteMaterialStarved, SiteResourceManifest};

#[derive(Component)]
pub struct FacilityInfoCardRoot;

#[derive(Component)]
pub struct FacilityInfoCardText;

/// Default distance of the card from the window's top-right corner (logical px).
const CARD_ANCHOR_RIGHT_TOP: Vec2 = Vec2::new(12.0, 120.0);

#[allow(clippy::type_complexity)]
pub fn sync_facility_info_card_system(
    mut commands: Commands,
    focus: Res<HudLogisticsFocus>,
    sites: Query<(
        &ConstructionSite,
        Option<&BuildingDefinitionRef>,
        Option<&SiteStageProgress>,
        Option<&SiteResourceManifest>,
        Option<&SiteMaterialStarved>,
        Option<&ResourceFlowNode>,
        Option<&FacilityFlowState>,
        Option<&FacilityInputs>,
    )>,
    player_owner: Option<Res<DamFloodPlayerOwner>>,
    other_sites: Query<&crate::economy::logistics::orders::LogisticsOrderSite, Without<ConstructionSite>>,
    roots: Query<Entity, With<FacilityInfoCardRoot>>,
    mut texts: Query<&mut Text, With<FacilityInfoCardText>>,
    style: (Option<Res<UiPalette>>, Option<Res<CmdUiMonoFont>>, Res<HudCardPlacements>),
) {
    let (palette, mono, placements) = style;
    let lines = focus
        .tracked_entity
        .and_then(|e| sites.get(e).ok())
        .map(|(site, def, stage, manifest, starved, node, state, inputs)| {
            let id = def
                .map(|d| d.catalog_id.as_str())
                .unwrap_or_else(|| match site.archetype {
                    crate::strategic::SiteArchetype::Dam => "dam",
                    crate::strategic::SiteArchetype::Factory => "factory",
                    crate::strategic::SiteArchetype::PowerPlant => "power_plant",
                    _ => "site",
                });
            let mut lines =
                facility_report_lines(id, site, stage, manifest, starved, node, state, inputs);
            // Intel floor: always report; mark foreign when local player owner is known and differs.
            if let Some(local) = player_owner.as_ref().and_then(|o| o.0) {
                if site.owner != local && site.owner != Entity::PLACEHOLDER {
                    lines.insert(0, "Intel: foreign / limited contact".into());
                }
            }
            lines
        })
        // Logistics sites that are not buildings (a scenario's freight yards, depots): a title
        // line so the card (and its yard / depot section) shows for them too.
        .or_else(|| {
            let s = other_sites.get(focus.tracked_entity?).ok()?;
            Some(vec![format!("{} — {}", s.name, s.role.label())])
        })
        .unwrap_or_default();

    if lines.is_empty() {
        for e in &roots {
            commands.entity(e).despawn();
        }
        return;
    }
    let body = lines.join("\n");
    if let Some(mut text) = texts.iter_mut().next() {
        if text.0 != body {
            text.0 = body;
        }
        return;
    }
    let (Some(palette), Some(mono)) = (palette, mono) else {
        return;
    };
    let (anchor, drag, right, top) = placements.card("site_card", CARD_ANCHOR_RIGHT_TOP);
    commands
        .spawn((
            FacilityInfoCardRoot,
            Node {
                position_type: PositionType::Absolute,
                top,
                right,
                width: Val::Px(380.0),
                flex_direction: FlexDirection::Column,
                padding: UiRect::all(Val::Px(8.0)),
                border: UiRect::all(Val::Px(1.0)),
                border_radius: BorderRadius::ZERO,
                ..default()
            },
            anchor,
            drag,
            BackgroundColor(palette.bevy_hud_panel_fill()),
            BorderColor::all(palette.bevy_wire_magenta()),
            FocusPolicy::Block,
            ZIndex(950),
        ))
        .with_children(|root| {
            root.spawn((
                FacilityInfoCardText,
                Text::new(body),
                TextFont::from_font_size(11.5).with_font(mono.0.clone()),
                TextColor(palette.bevy_primary_text()),
            ));
        });
}
