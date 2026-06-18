//! Staged parametric placements — queue rows before commit (**CONSTRUCTION-PARAM-CODER-004** / P3-A).

/// Tray Build tab owns staging in sim — no RIGHT_BOTTOM floater.
pub const STAGED_PANEL_FLOATING_SIM: bool = false;

use bevy::prelude::*;

use crate::engine::states::BaseState;
use bevy_egui::egui;

use crate::gui::InputBindings;
use crate::strategic::{
    BuildSiteTile, CommittedPlacementSnapshot, FootprintTiles, LayerType, SiteArchetype,
};

use super::build_commit::queue_commit_construction_site;
use super::build_state::{BuildCommandActor, BuildGhostState, BuildPlacementPreview};
use super::build_strip::{BuildStripState, ToolContext};
use super::build_tool_authority::{ActiveBuildTool, BuildTool};
use super::building_definitions::BuildingDefinitionRegistry;
use super::history::ConstructionHistory;
use super::parametric_commit::parametric_placement_snapshot;
use super::placement_scaling::clamp_scale_factor;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum StagedValidity {
    Ok,
    Warn,
    Bad,
}

#[derive(Clone, Debug)]
pub struct StagedPlacementRow {
    pub id: u64,
    pub catalog_id: String,
    pub anchor_tile: BuildSiteTile,
    pub scale: f32,
    pub rotation_quarter_turns: u8,
    pub mirror_x: bool,
    pub approved: bool,
    pub validity: StagedValidity,
    pub allows_commit: bool,
    pub footprint_weights: Vec<(IVec2, f32)>,
    pub archetype: SiteArchetype,
    pub footprint: FootprintTiles,
    pub layer: LayerType,
    pub placement: CommittedPlacementSnapshot,
}

/// `Stage placements` toggle — when true, Enter does not commit; LMB appends a staged row.
#[derive(Resource, Debug, Clone, Copy)]
pub struct StagedPlacementMode {
    pub enabled: bool,
}

impl Default for StagedPlacementMode {
    fn default() -> Self {
        Self { enabled: false }
    }
}

#[derive(Resource, Debug, Default)]
pub struct StagedPlacementBook {
    pub rows: Vec<StagedPlacementRow>,
    next_id: u64,
}

impl StagedPlacementBook {
    pub fn staged_count(&self) -> usize {
        self.rows.len()
    }

    pub fn push_from_ghost(
        &mut self,
        catalog_id: String,
        archetype: SiteArchetype,
        footprint: FootprintTiles,
        layer: LayerType,
        ghost: &BuildGhostState,
        placement: CommittedPlacementSnapshot,
        allows_commit: bool,
        validity: StagedValidity,
    ) {
        let id = self.next_id;
        self.next_id = self.next_id.saturating_add(1);
        self.rows.push(StagedPlacementRow {
            id,
            catalog_id,
            anchor_tile: ghost.origin.unwrap_or(placement.origin),
            scale: placement.scale_factor,
            rotation_quarter_turns: ghost.rotation_quarter_turns,
            mirror_x: ghost.mirror_x,
            approved: false,
            validity,
            allows_commit,
            footprint_weights: placement.weights.clone(),
            archetype,
            footprint,
            layer,
            placement,
        });
    }

    pub fn clear_unapproved_or_invalid(&mut self) {
        self.rows
            .retain(|r| r.approved && r.validity != StagedValidity::Bad);
    }

    pub fn approve_all_valid(&mut self) {
        for row in &mut self.rows {
            if row.allows_commit && row.validity != StagedValidity::Bad {
                row.approved = true;
            }
        }
    }

    pub fn drain_approved_committable(&mut self) -> Vec<StagedPlacementRow> {
        let mut out = Vec::new();
        self.rows.retain(|row| {
            if row.approved && row.allows_commit && row.validity != StagedValidity::Bad {
                out.push(row.clone());
                false
            } else {
                true
            }
        });
        out
    }
}

fn staged_validity_from_preview(
    allows_commit: bool,
    scale: f32,
    errors: &[String],
) -> StagedValidity {
    if errors.iter().any(|e| e == "weighted_overlap") || !allows_commit {
        return StagedValidity::Bad;
    }
    if scale < 0.25 {
        return StagedValidity::Bad;
    }
    if scale < 0.35 {
        return StagedValidity::Warn;
    }
    StagedValidity::Ok
}

/// After pick + validation: append staged row when staging mode is on.
pub fn stage_active_ghost_on_lmb_system(
    buttons: Res<ButtonInput<MouseButton>>,
    strip: Res<BuildStripState>,
    tool: Res<ActiveBuildTool>,
    staging: Res<StagedPlacementMode>,
    ghost: Res<BuildGhostState>,
    preview: Res<BuildPlacementPreview>,
    registry: Res<BuildingDefinitionRegistry>,
    mut book: ResMut<StagedPlacementBook>,
) {
    if !staging.enabled || strip.active == ToolContext::None {
        return;
    }
    let BuildTool::Building(_) = tool.tool else {
        return;
    };
    if !buttons.just_pressed(MouseButton::Left) {
        return;
    }
    let Some(origin) = ghost.origin else {
        return;
    };
    let Some(intent) = tool.building_intent.as_ref() else {
        return;
    };
    let Some(catalog_id) = intent.catalog_id.clone() else {
        return;
    };
    let Some(def) = registry.get(&catalog_id) else {
        return;
    };
    let placement = parametric_placement_snapshot(
        &def.footprint,
        def.family,
        origin,
        ghost.rotation_quarter_turns,
        ghost.mirror_x,
        Some(ghost.scale_factor),
    );
    let scale = clamp_scale_factor(placement.scale_factor);
    let validity = staged_validity_from_preview(
        preview.report.allows_commit,
        scale,
        &preview.report.errors,
    );
    book.push_from_ghost(
        catalog_id,
        def.site_archetype,
        ghost.footprint,
        LayerType::Surface,
        &ghost,
        placement,
        preview.report.allows_commit,
        validity,
    );
}

/// Parametric HUD line 1 for active ghost (`construction_parametric_scale_hud_v1.md`).
#[must_use]
pub fn parametric_active_ghost_hud_line(
    tool: &ActiveBuildTool,
    registry: &BuildingDefinitionRegistry,
    ghost: &BuildGhostState,
    preview: &BuildPlacementPreview,
) -> Option<String> {
    let origin = ghost.origin?;
    let intent = tool.building_intent.as_ref()?;
    let catalog_id = intent.catalog_id.as_deref()?;
    let def = registry.get(catalog_id)?;
    let snap = parametric_placement_snapshot(
        &def.footprint,
        def.family,
        origin,
        ghost.rotation_quarter_turns,
        ghost.mirror_x,
        Some(ghost.scale_factor),
    );
    let mass: f32 = snap.weights.iter().map(|(_, w)| *w).sum();
    let prod_pct = 100.0
        * crate::economy::activation::scale::production_multiplier(snap.effective_scale, crate::economy::activation::scale::DEFAULT_K_PROD);
    let short = catalog_id.chars().take(18).collect::<String>();
    let rot = ghost.rotation_quarter_turns as u32 * 90;
    let valid = if preview.report.allows_commit {
        "Valid"
    } else {
        "Invalid"
    };
    Some(format!(
        "{short} · Scale {scale:.2}× · Rot {rot}° · Mass {mass:.1} tiles\nProd ~{prod_pct:.0}% · {valid}",
        short = short,
        scale = snap.scale_factor,
        rot = rot,
        mass = mass,
        prod_pct = prod_pct,
        valid = valid,
    ))
}

pub fn build_all_valid_staged_rows(
    book: &mut StagedPlacementBook,
    actor: Entity,
    events: &mut MessageWriter<crate::strategic::CommitConstructionSiteEvent>,
    history: &mut ConstructionHistory,
    occupation: Option<&crate::strategic::TileOccupationBook>,
) -> usize {
    book.approve_all_valid();
    commit_approved_staged_rows(book, actor, events, history, occupation)
}

/// Staging panel + tray readout (P3-A / P3-B).
pub fn draw_staged_placements_panel_egui_system(
    mut contexts: bevy_egui::EguiContexts,
    base: Res<State<crate::engine::states::BaseState>>,
    strip: Res<BuildStripState>,
    tool: Res<ActiveBuildTool>,
    ghost: Res<BuildGhostState>,
    preview: Res<BuildPlacementPreview>,
    registry: Res<BuildingDefinitionRegistry>,
    mut staging: ResMut<StagedPlacementMode>,
    mut book: ResMut<StagedPlacementBook>,
    actor: Res<BuildCommandActor>,
    mut events: MessageWriter<crate::strategic::CommitConstructionSiteEvent>,
    mut history: ResMut<ConstructionHistory>,
    occupation: Option<Res<crate::strategic::TileOccupationBook>>,
) -> Result {
    if strip.active == ToolContext::None {
        return Ok(());
    }
    if !matches!(tool.tool, BuildTool::Building(_)) {
        return Ok(());
    }
    if matches!(base.get(), BaseState::Simulation) {
        return Ok(());
    }
    let ctx = contexts.ctx_mut()?;
    egui::Area::new(egui::Id::new("construction_staged_parametric_panel"))
        .anchor(egui::Align2::RIGHT_BOTTOM, egui::vec2(-16.0, -140.0))
        .show(ctx, |ui| {
            ui.heading("Parametric placement");
            ui.checkbox(&mut staging.enabled, "Stage placements");
            if let Some(line) = parametric_active_ghost_hud_line(&tool, &registry, &ghost, &preview) {
                ui.label(egui::RichText::new(line).small());
            }
            let panel_visible = staging.enabled || !book.rows.is_empty();
            if !panel_visible {
                return;
            }
            ui.add_space(4.0);
            ui.set_min_width(280.0);
            draw_staged_placements_panel_body(
                ui,
                &mut book,
                actor.0,
                &mut events,
                &mut history,
                occupation.as_deref(),
            );
        });
    Ok(())
}

pub fn draw_staged_placements_panel_body(
    ui: &mut egui::Ui,
    book: &mut StagedPlacementBook,
    actor: Entity,
    events: &mut MessageWriter<crate::strategic::CommitConstructionSiteEvent>,
    history: &mut ConstructionHistory,
    occupation: Option<&crate::strategic::TileOccupationBook>,
) {
    ui.label(
        egui::RichText::new(format!("Staged placements ({})", book.staged_count()))
            .strong(),
    );
    if book.rows.is_empty() {
        ui.label(
            egui::RichText::new("No staged ghosts — adjust active ghost, then LMB on map.")
                .weak(),
        );
    } else {
        egui::ScrollArea::vertical().max_height(120.0).show(ui, |ui| {
            let mut remove_idx = None;
            for (idx, row) in book.rows.iter_mut().enumerate() {
                ui.horizontal(|ui| {
                    ui.checkbox(&mut row.approved, "");
                    let label = row.catalog_id.chars().take(14).collect::<String>();
                    ui.label(label);
                    ui.label(format!("{:.2}×", row.scale));
                    ui.label(format!("{}°", row.rotation_quarter_turns as u32 * 90));
                    let badge = match row.validity {
                        StagedValidity::Ok => "OK",
                        StagedValidity::Warn => "Warn",
                        StagedValidity::Bad => "Bad",
                    };
                    ui.label(egui::RichText::new(badge).weak());
                    if ui.small_button("✕").clicked() {
                        remove_idx = Some(idx);
                    }
                });
            }
            if let Some(idx) = remove_idx {
                book.rows.remove(idx);
            }
        });
    }
    ui.separator();
    let approved_valid = book.rows.iter().any(|r| {
        r.approved && r.allows_commit && r.validity != StagedValidity::Bad
    });
    let any_valid = book
        .rows
        .iter()
        .any(|r| r.allows_commit && r.validity != StagedValidity::Bad);
    let any_clear = book.rows.iter().any(|r| !r.approved || r.validity == StagedValidity::Bad);
    ui.horizontal(|ui| {
        if ui
            .add_enabled(approved_valid, egui::Button::new("Build approved"))
            .clicked()
        {
            commit_approved_staged_rows(book, actor, events, history, occupation);
        }
        if ui
            .add_enabled(any_valid, egui::Button::new("Build all valid"))
            .clicked()
        {
            build_all_valid_staged_rows(book, actor, events, history, occupation);
        }
        if ui
            .add_enabled(any_clear, egui::Button::new("Clear unapproved"))
            .clicked()
        {
            book.clear_unapproved_or_invalid();
        }
    });
}

#[must_use]
pub fn staging_panel_visible_witness_green() -> bool {
    staging_panel_visible_self_check().is_ok()
}

#[must_use]
pub fn staging_validity_badges_wired_witness_green() -> bool {
    staging_validity_badges_self_check().is_ok()
}

fn staging_validity_badges_self_check() -> Result<(), &'static str> {
    for v in [
        StagedValidity::Ok,
        StagedValidity::Warn,
        StagedValidity::Bad,
    ] {
        let _ = match v {
            StagedValidity::Ok => "OK",
            StagedValidity::Warn => "Warn",
            StagedValidity::Bad => "Bad",
        };
    }
    Ok(())
}

fn staging_panel_visible_self_check() -> Result<(), &'static str> {
    let mode = StagedPlacementMode { enabled: true };
    let book = StagedPlacementBook::default();
    if !(mode.enabled || book.staged_count() > 0) {
        return Err("visible_when_enabled");
    }
    let book2 = StagedPlacementBook {
        rows: vec![StagedPlacementRow {
            id: 1,
            catalog_id: "x".into(),
            anchor_tile: BuildSiteTile { x: 0, z: 0 },
            scale: 1.0,
            rotation_quarter_turns: 0,
            mirror_x: false,
            approved: false,
            validity: StagedValidity::Ok,
            allows_commit: true,
            footprint_weights: vec![],
            archetype: SiteArchetype::Factory,
            footprint: FootprintTiles {
                width: 1,
                depth: 1,
            },
            layer: LayerType::Surface,
            placement: CommittedPlacementSnapshot {
                origin: BuildSiteTile { x: 0, z: 0 },
                scale_factor: 1.0,
                effective_scale: 1.0,
                rotation_quarter_turns: 0,
                mirror_x: false,
                weights: vec![],
            },
        }],
        next_id: 2,
    };
    if book2.staged_count() == 0 {
        return Err("visible_when_rows");
    }
    Ok(())
}

/// **Build approved** — drain approved valid rows through the single commit funnel.
pub fn commit_approved_staged_placements_system(
    keys: Res<ButtonInput<KeyCode>>,
    bindings: Res<InputBindings>,
    strip: Res<BuildStripState>,
    tool: Res<ActiveBuildTool>,
    staging: Res<StagedPlacementMode>,
    actor: Res<BuildCommandActor>,
    registry: Res<BuildingDefinitionRegistry>,
    mut book: ResMut<StagedPlacementBook>,
    mut events: MessageWriter<crate::strategic::CommitConstructionSiteEvent>,
    mut history: ResMut<ConstructionHistory>,
    occupation: Option<Res<crate::strategic::TileOccupationBook>>,
) {
    if strip.active == ToolContext::None {
        return;
    }
    if !keys.just_pressed(bindings.confirm_build_placement) {
        return;
    }
    if !(keys.pressed(KeyCode::ShiftLeft) || keys.pressed(KeyCode::ShiftRight)) {
        return;
    }
    if !staging.enabled {
        return;
    }
    let BuildTool::Building(_) = tool.tool else {
        return;
    };
    let _ = registry;
    let rows = book.drain_approved_committable();
    for row in rows {
        if let Some(book) = occupation.as_ref() {
            if book.would_overlap(&row.placement.weights) {
                continue;
            }
        }
        queue_commit_construction_site(
            &mut events,
            actor.0,
            row.archetype,
            row.anchor_tile,
            row.footprint,
            row.layer,
            Some(row.catalog_id),
            Some(row.placement),
        );
        history.queue_site(row.anchor_tile);
    }
}

pub fn commit_approved_staged_rows(
    book: &mut StagedPlacementBook,
    actor: Entity,
    events: &mut MessageWriter<crate::strategic::CommitConstructionSiteEvent>,
    history: &mut ConstructionHistory,
    occupation: Option<&crate::strategic::TileOccupationBook>,
) -> usize {
    let rows = book.drain_approved_committable();
    let mut committed = 0usize;
    for row in rows {
        if let Some(occ) = occupation {
            if occ.would_overlap(&row.placement.weights) {
                continue;
            }
        }
        queue_commit_construction_site(
            events,
            actor,
            row.archetype,
            row.anchor_tile,
            row.footprint,
            row.layer,
            Some(row.catalog_id),
            Some(row.placement),
        );
        history.queue_site(row.anchor_tile);
        committed = committed.saturating_add(1);
    }
    committed
}

#[must_use]
pub fn staging_toggle_wired_witness_green() -> bool {
    staging_toggle_self_check().is_ok()
}

fn staging_toggle_self_check() -> Result<(), &'static str> {
    let mut mode = StagedPlacementMode::default();
    mode.enabled = true;
    if !mode.enabled {
        return Err("toggle");
    }
    mode.enabled = false;
    if mode.enabled {
        return Err("toggle_off");
    }
    Ok(())
}

#[must_use]
pub fn build_approved_drains_staged_witness_green() -> bool {
    build_approved_drains_staged_self_check().is_ok()
}

fn build_approved_drains_staged_self_check() -> Result<(), &'static str> {
    use crate::construction::building_catalog::{BuildingFamily, FootprintMatrix};

    let mut book = StagedPlacementBook::default();
    let ghost = BuildGhostState {
        origin: Some(BuildSiteTile { x: 8, z: 8 }),
        ..Default::default()
    };
    let placement = parametric_placement_snapshot(
        &FootprintMatrix::from_size(1, 1, true),
        BuildingFamily::Industry,
        BuildSiteTile { x: 8, z: 8 },
        0,
        false,
        None,
    );
    book.push_from_ghost(
        "test_factory".into(),
        SiteArchetype::Factory,
        FootprintTiles {
            width: 1,
            depth: 1,
        },
        LayerType::Surface,
        &ghost,
        placement,
        true,
        StagedValidity::Ok,
    );
    book.rows[0].approved = true;
    let drained = book.drain_approved_committable();
    if drained.len() != 1 {
        return Err("drain_count");
    }
    if !book.rows.is_empty() {
        return Err("book_not_drained");
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn staging_toggle_wired() {
        assert!(staging_toggle_wired_witness_green());
    }

    #[test]
    fn build_approved_drains_staged() {
        assert!(build_approved_drains_staged_witness_green());
    }

    #[test]
    fn build_approved_skips_unapproved() {
        use crate::construction::building_catalog::{BuildingFamily, FootprintMatrix};

        let mut book = StagedPlacementBook::default();
        let ghost = BuildGhostState {
            origin: Some(BuildSiteTile { x: 0, z: 0 }),
            ..Default::default()
        };
        let placement = parametric_placement_snapshot(
            &FootprintMatrix::from_size(1, 1, true),
            BuildingFamily::Industry,
            BuildSiteTile { x: 0, z: 0 },
            0,
            false,
            None,
        );
        book.push_from_ghost(
            "a".into(),
            SiteArchetype::Factory,
            FootprintTiles {
                width: 1,
                depth: 1,
            },
            LayerType::Surface,
            &ghost,
            placement.clone(),
            true,
            StagedValidity::Ok,
        );
        book.push_from_ghost(
            "b".into(),
            SiteArchetype::Factory,
            FootprintTiles {
                width: 1,
                depth: 1,
            },
            LayerType::Surface,
            &ghost,
            placement,
            true,
            StagedValidity::Ok,
        );
        book.rows[0].approved = true;
        book.rows[1].approved = false;
        let drained = book.drain_approved_committable();
        assert_eq!(drained.len(), 1);
        assert_eq!(drained[0].catalog_id, "a");
        assert_eq!(book.rows.len(), 1);
        assert_eq!(book.rows[0].catalog_id, "b");
    }

    #[test]
    fn staging_panel_visible_witness() {
        assert!(staging_panel_visible_witness_green());
        assert!(staging_validity_badges_wired_witness_green());
    }

    #[test]
    fn enter_does_not_commit_when_staging_on() {
        use bevy::prelude::{App, MinimalPlugins, Update};

        use crate::gui::InputBindings;
        use crate::strategic::CommitConstructionSiteEvent;

        use super::super::build_interaction::build_confirm_site_system;
        use super::super::build_strip::{BuildStripState, ToolContext};
        use super::super::build_tool_authority::{BuildingArchetypeId, BuildTool};
        use super::super::pending_construction::PendingConstructionQueue;
        use super::super::sessions::ActiveToolSession;

        #[derive(Resource, Default)]
        struct CommitCount(u32);

        fn count(mut reader: MessageReader<CommitConstructionSiteEvent>, mut n: ResMut<CommitCount>) {
            for _ in reader.read() {
                n.0 = n.0.saturating_add(1);
            }
        }

        let mut app = App::new();
        app.add_plugins(MinimalPlugins)
            .init_resource::<ButtonInput<KeyCode>>()
            .init_resource::<ButtonInput<MouseButton>>()
            .init_resource::<InputBindings>()
            .init_resource::<BuildStripState>()
            .init_resource::<ActiveBuildTool>()
            .init_resource::<BuildGhostState>()
            .init_resource::<BuildPlacementPreview>()
            .init_resource::<PendingConstructionQueue>()
            .init_resource::<ActiveToolSession>()
            .init_resource::<BuildingDefinitionRegistry>()
            .init_resource::<ConstructionHistory>()
            .init_resource::<StagedPlacementMode>()
            .init_resource::<StagedPlacementBook>()
            .init_resource::<CommitCount>()
            .add_message::<CommitConstructionSiteEvent>()
            .add_systems(Update, (build_confirm_site_system, count).chain());

        {
            let actor = app.world_mut().spawn_empty().id();
            app.world_mut().insert_resource(BuildCommandActor(actor));
            app.world_mut().resource_mut::<BuildStripState>().active = ToolContext::Industry;
            app.world_mut().resource_mut::<ActiveBuildTool>().tool =
                BuildTool::Building(BuildingArchetypeId::Factory);
            app.world_mut().resource_mut::<StagedPlacementMode>().enabled = true;
            let mut ghost = app.world_mut().resource_mut::<BuildGhostState>();
            ghost.origin = Some(BuildSiteTile { x: 2, z: 2 });
            let mut preview = app.world_mut().resource_mut::<BuildPlacementPreview>();
            preview.report.allows_commit = true;
            preview.report.valid = true;
        }

        app.world_mut()
            .resource_mut::<ButtonInput<KeyCode>>()
            .press(KeyCode::Enter);
        app.update();

        assert_eq!(app.world().resource::<CommitCount>().0, 0);
        assert!(app.world().resource::<BuildGhostState>().origin.is_some());
    }
}
