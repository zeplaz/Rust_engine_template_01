//! Site / plant info card (Bevy UI): for the focused site, shows build progress and the material
//! bill while it is being built, and once operational its production rate, workforce and power
//! (with the town / grid they come from), input and output buffers and every reason output is
//! held back. Presentation only: the text comes from `economy::facility_report`.
//!
//! Focus: the logistics focus (`HudLogisticsFocus`: map pick or the F9 cycle).

use bevy::prelude::*;
use bevy::ui::FocusPolicy;

use crate::economy::activation::BuildingDefinitionRef;
use crate::economy::facility_inputs::FacilityInputs;
use crate::economy::facility_report::facility_report_lines;
use crate::economy::resource_flow::{FacilityFlowState, ResourceFlowNode};
use crate::gui::logistics_focus::HudLogisticsFocus;
use crate::gui::style::{CmdUiMonoFont, UiPalette};
use crate::construction::SiteStageProgress;
use crate::strategic::{ConstructionSite, SiteMaterialStarved, SiteResourceManifest};

#[derive(Component)]
pub struct FacilityInfoCardRoot;

#[derive(Component)]
pub struct FacilityInfoCardText;

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
    roots: Query<Entity, With<FacilityInfoCardRoot>>,
    mut texts: Query<&mut Text, With<FacilityInfoCardText>>,
    palette: Option<Res<UiPalette>>,
    mono: Option<Res<CmdUiMonoFont>>,
) {
    let lines = focus
        .tracked_entity
        .and_then(|e| sites.get(e).ok())
        .map(|(site, def, stage, manifest, starved, node, state, inputs)| {
            let id = def.map_or("site", |d| d.catalog_id.as_str());
            facility_report_lines(id, site, stage, manifest, starved, node, state, inputs)
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
    commands
        .spawn((
            FacilityInfoCardRoot,
            Node {
                position_type: PositionType::Absolute,
                top: Val::Px(120.0),
                right: Val::Px(12.0),
                width: Val::Px(380.0),
                padding: UiRect::all(Val::Px(8.0)),
                border: UiRect::all(Val::Px(1.0)),
                border_radius: BorderRadius::ZERO,
                ..default()
            },
            BackgroundColor(palette.bevy_hud_panel_fill()),
            BorderColor::all(palette.bevy_wire_magenta()),
            FocusPolicy::Pass,
            Pickable::IGNORE,
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
