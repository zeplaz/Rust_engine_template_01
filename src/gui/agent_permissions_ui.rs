use bevy::prelude::*;
use bevy_egui::{egui, EguiContexts, EguiPrimaryContextPass};

use crate::idgen::EntityId;
use crate::systems::agents::permissions::{
    Agent, AgentPermissions, PermissionDomain, AccessLevel, PermissionGrant, 
    PermissionGrantEvent, PermissionRevokeEvent
};
use crate::systems::agents::agent_manager::AgentManager;
use crate::events::ownership_events::FactionColors;

/// UI state for the permissions window
#[derive(Resource)]
pub struct PermissionsUiState {
    pub visible: bool,
    pub selected_agent: Option<EntityId>,
    pub selected_target: Option<EntityId>,
    pub filter_domain: Option<PermissionDomain>,
    pub domain_scroll_pos: f32,
    pub temp_grant_expiration: Option<f64>,
}

impl Default for PermissionsUiState {
    fn default() -> Self {
        Self {
            visible: false,
            selected_agent: None,
            selected_target: None,
            filter_domain: None,
            domain_scroll_pos: 0.0,
            temp_grant_expiration: None,
        }
    }
}

/// Event to toggle the permissions UI
#[derive(Message)]
pub struct TogglePermissionsUiEvent;

/// System to handle UI visibility toggle
pub fn toggle_permissions_ui(
    mut events: MessageReader<TogglePermissionsUiEvent>,
    mut ui_state: ResMut<PermissionsUiState>,
) {
    for _ in events.read() {
        ui_state.visible = !ui_state.visible;
    }
}

/// System to render the permissions UI — EguiPrimaryContextPass, returns Result.
#[allow(clippy::too_many_arguments)]
pub fn permissions_ui_system(
    mut contexts: EguiContexts,
    mut ui_state: ResMut<PermissionsUiState>,
    agent_manager: Res<AgentManager>,
    agent_query: Query<(&Agent, &AgentPermissions)>,
    _faction_colors: Res<FactionColors>,
    local_player_agent: Option<Res<LocalPlayerAgent>>, // Would need to be set up elsewhere
    game_time: Res<crate::systems::agents::permissions::GameTime>,
    mut grant_events: MessageWriter<PermissionGrantEvent>,
    mut revoke_events: MessageWriter<PermissionRevokeEvent>,
) -> Result {
    if !ui_state.visible {
        return Ok(());
    }
    
    // Default to local player's agent if none selected
    if ui_state.selected_agent.is_none() && local_player_agent.is_some() {
        ui_state.selected_agent = local_player_agent.as_ref().map(|lpa| lpa.agent_id);
    }
    
    egui::Window::new("Agent Permissions")
        .resizable(true)
        .min_width(600.0)
        .show(contexts.ctx_mut()?, |ui| {
            ui.horizontal(|ui| {
                ui.label("Viewing permissions for:");
                
                egui::ComboBox::from_id_salt("agent_selector")
                    .selected_text(
                        ui_state.selected_agent
                            .and_then(|id| agent_manager.get_agent_entity(id))
                            .and_then(|entity| agent_query.get(entity).ok())
                            .map(|(agent, _)| agent.name.clone())
                            .unwrap_or_else(|| "Select Agent".to_string())
                    )
                    .show_ui(ui, |ui| {
                        // List all agents
                        for agent_id in &agent_manager.human_players {
                            if let Some(entity) = agent_manager.get_agent_entity(*agent_id) {
                                if let Ok((agent, _)) = agent_query.get(entity) {
                                    if ui.selectable_label(
                                        ui_state.selected_agent == Some(*agent_id),
                                        &agent.name
                                    ).clicked() {
                                        ui_state.selected_agent = Some(*agent_id);
                                    }
                                }
                            }
                        }
                        
                        ui.separator();
                        
                        for agent_id in &agent_manager.ai_players {
                            if let Some(entity) = agent_manager.get_agent_entity(*agent_id) {
                                if let Ok((agent, _)) = agent_query.get(entity) {
                                    if ui.selectable_label(
                                        ui_state.selected_agent == Some(*agent_id),
                                        &agent.name
                                    ).clicked() {
                                        ui_state.selected_agent = Some(*agent_id);
                                    }
                                }
                            }
                        }
                    });
            });
            
            ui.separator();
            
            if let Some(agent_id) = ui_state.selected_agent {
                if let Some(entity) = agent_manager.get_agent_entity(agent_id) {
                    if let Ok((agent, permissions)) = agent_query.get(entity) {
                        // Agent info section
                        ui.horizontal(|ui| {
                            let color = agent.color;
                            let color_rect = egui::Rect::from_min_size(
                                ui.cursor().min,
                                egui::vec2(20.0, 20.0)
                            );
                            let srgba = color.to_srgba();
                            ui.painter().rect_filled(
                                color_rect,
                                0.0,
                                egui::Color32::from_rgba_unmultiplied(
                                    (srgba.red * 255.0) as u8,
                                    (srgba.green * 255.0) as u8,
                                    (srgba.blue * 255.0) as u8,
                                    (srgba.alpha * 255.0) as u8,
                                )
                            );
                            ui.add_space(25.0);
                            
                            ui.vertical(|ui| {
                                ui.label(format!("Name: {}", agent.name));
                                ui.label(format!("Type: {:?}", agent.agent_type));
                                ui.label(format!("Faction: {}", 
                                    agent_manager.get_agent_entity(agent.faction_id)
                                        .and_then(|e| agent_query.get(e).ok())
                                        .map(|(a, _)| a.name.clone())
                                        .unwrap_or_else(|| "Unknown".to_string())
                                ));
                            });
                        });
                        
                        ui.separator();
                        
                        // Tab view for different permission views
                        egui::TopBottomPanel::top("tabs").show_inside(ui, |ui| {
                            ui.horizontal(|ui| {
                                ui.selectable_value(&mut ui_state.filter_domain, None, "All Permissions");
                                
                                let _domains = [
                                    PermissionDomain::EconomicPolicy,
                                    PermissionDomain::TradePolicy,
                                    PermissionDomain::ResourceManagement,
                                    PermissionDomain::Production,
                                    PermissionDomain::RoadConstruction,
                                    PermissionDomain::RailConstruction,
                                    PermissionDomain::PowerInfrastructure,
                                    PermissionDomain::WaterInfrastructure,
                                    PermissionDomain::MilitaryCommand,
                                    PermissionDomain::DefenseInfrastructure,
                                    PermissionDomain::IntelligenceOps,
                                    PermissionDomain::MilitaryProduction,
                                    PermissionDomain::DiplomaticRelations,
                                    PermissionDomain::FactionPolicy,
                                    PermissionDomain::Research,
                                    PermissionDomain::PopulationManagement,
                                    PermissionDomain::Admin,
                                    PermissionDomain::Observer,
                                ];
                                
                                // Group domains into categories
                                let categories = [
                                    ("Economic", [
                                        PermissionDomain::EconomicPolicy,
                                        PermissionDomain::TradePolicy,
                                        PermissionDomain::ResourceManagement,
                                        PermissionDomain::Production,
                                    ].as_slice()),
                                    ("Infrastructure", [
                                        PermissionDomain::RoadConstruction,
                                        PermissionDomain::RailConstruction,
                                        PermissionDomain::PowerInfrastructure,
                                        PermissionDomain::WaterInfrastructure,
                                    ].as_slice()),
                                    ("Military", [
                                        PermissionDomain::MilitaryCommand,
                                        PermissionDomain::DefenseInfrastructure,
                                        PermissionDomain::IntelligenceOps,
                                        PermissionDomain::MilitaryProduction,
                                    ].as_slice()),
                                    ("Governance", [
                                        PermissionDomain::DiplomaticRelations,
                                        PermissionDomain::FactionPolicy,
                                        PermissionDomain::Research,
                                        PermissionDomain::PopulationManagement,
                                    ].as_slice()),
                                    ("Special", [
                                        PermissionDomain::Admin,
                                        PermissionDomain::Observer,
                                    ].as_slice()),
                                ];
                                
                                for (category_name, category_domains) in categories {
                                    ui.menu_button(category_name, |ui| {
                                        for &domain in category_domains {
                                            if ui.selectable_label(
                                                ui_state.filter_domain == Some(domain),
                                                format!("{:?}", domain)
                                            ).clicked() {
                                                ui_state.filter_domain = Some(domain);
                                                ui.close();
                                            }
                                        }
                                    });
                                }
                            });
                        });
                        
                        // Permissions table
                        egui::ScrollArea::vertical().show(ui, |ui| {
                            egui::Grid::new("permissions_grid")
                                .striped(true)
                                .spacing([5.0, 5.0])
                                .min_col_width(100.0)
                                .show(ui, |ui| {
                                    // Header
                                    ui.label("Domain");
                                    ui.label("Access Level");
                                    ui.label("Granted By");
                                    ui.label("Expiration");
                                    ui.label("Limitations");
                                    ui.label("Actions");
                                    ui.end_row();
                                    
                                    // List permissions
                                    for (&domain, grant) in &permissions.permissions {
                                        // Skip if filtered
                                        if let Some(filter_domain) = ui_state.filter_domain {
                                            if domain != filter_domain {
                                                continue;
                                            }
                                        }
                                        
                                        // Domain
                                        ui.label(format!("{:?}", domain));
                                        
                                        // Access Level
                                        ui.label(format!("{:?}", grant.access_level));
                                        
                                        // Granted By
                                        ui.label(
                                            agent_manager.get_agent_entity(grant.grantor)
                                                .and_then(|e| agent_query.get(e).ok())
                                                .map(|(a, _)| a.name.clone())
                                                .unwrap_or_else(|| "System".to_string())
                                        );
                                        
                                        // Expiration
                                        if let Some(exp) = grant.expiration {
                                            let time_left = exp - game_time.current_time;
                                            if time_left > 0.0 {
                                                ui.label(format!("{:.1} hours left", time_left));
                                            } else {
                                                ui.label("Expired");
                                            }
                                        } else {
                                            ui.label("Never");
                                        }
                                        
                                        // Limitations
                                        let mut limitations = Vec::new();
                                        if let Some(regions) = &grant.region_limited {
                                            limitations.push(format!("{} regions", regions.len()));
                                        }
                                        if let Some(entities) = &grant.entity_limited {
                                            limitations.push(format!("{} entities", entities.len()));
                                        }
                                        if let Some(condition) = &grant.condition {
                                            limitations.push(format!("Conditional: {:?}", condition));
                                        }
                                        
                                        if limitations.is_empty() {
                                            ui.label("None");
                                        } else {
                                            ui.label(limitations.join(", "));
                                        }
                                        
                                        // Actions
                                        ui.horizontal(|ui| {
                                            if ui.button("Revoke").clicked() {
                                                // Check if local player has authority to revoke
                                                if let Some(local_id) = local_player_agent.as_ref().map(|lpa| lpa.agent_id) {
                                                    revoke_events.write(PermissionRevokeEvent {
                                                        from_agent_id: agent_id,
                                                        revoker_id: local_id,
                                                        domain,
                                                    });
                                                }
                                            }
                                            
                                            if ui.button("Delegate").clicked() {
                                                // Open delegation dialog
                                                ui_state.selected_target = None;
                                            }
                                        });
                                        
                                        ui.end_row();
                                    }
                                });
                                
                            // Add new permission section
                            if local_player_agent.is_some() {
                                ui.separator();
                                ui.heading("Grant New Permission");
                                
                                egui::Grid::new("grant_permission_grid")
                                    .spacing([5.0, 5.0])
                                    .min_col_width(100.0)
                                    .show(ui, |ui| {
                                        // Domain selection
                                        ui.label("Domain:");
                                        let mut selected_domain = ui_state.filter_domain.unwrap_or(PermissionDomain::Observer);
                                        egui::ComboBox::from_id_salt("domain_selector")
                                            .selected_text(format!("{:?}", selected_domain))
                                            .show_ui(ui, |ui| {
                                                for &domain in &[
                                                    PermissionDomain::EconomicPolicy,
                                                    PermissionDomain::TradePolicy,
                                                    PermissionDomain::ResourceManagement,
                                                    PermissionDomain::Production,
                                                    PermissionDomain::RoadConstruction,
                                                    PermissionDomain::RailConstruction,
                                                    PermissionDomain::PowerInfrastructure,
                                                    PermissionDomain::WaterInfrastructure,
                                                    PermissionDomain::MilitaryCommand,
                                                    PermissionDomain::DefenseInfrastructure,
                                                    PermissionDomain::IntelligenceOps,
                                                    PermissionDomain::MilitaryProduction,
                                                    PermissionDomain::DiplomaticRelations,
                                                    PermissionDomain::FactionPolicy,
                                                    PermissionDomain::Research,
                                                    PermissionDomain::PopulationManagement,
                                                    PermissionDomain::Admin,
                                                    PermissionDomain::Observer,
                                                ] {
                                                    if ui.selectable_label(
                                                        selected_domain == domain,
                                                        format!("{:?}", domain)
                                                    ).clicked() {
                                                        selected_domain = domain;
                                                        ui_state.filter_domain = Some(domain);
                                                    }
                                                }
                                            });
                                        ui.end_row();
                                        
                                        // Access level selection
                                        ui.label("Access Level:");
                                        let mut selected_level = AccessLevel::ReadOnly;
                                        egui::ComboBox::from_id_salt("access_level_selector")
                                            .selected_text(format!("{:?}", selected_level))
                                            .show_ui(ui, |ui| {
                                                for &level in &[
                                                    AccessLevel::ReadOnly,
                                                    AccessLevel::Limited,
                                                    AccessLevel::Full,
                                                    AccessLevel::Owner,
                                                ] {
                                                    if ui.selectable_label(
                                                        selected_level == level,
                                                        format!("{:?}", level)
                                                    ).clicked() {
                                                        selected_level = level;
                                                    }
                                                }
                                            });
                                        ui.end_row();
                                        
                                        // Expiration
                                        ui.label("Expiration:");
                                        let mut temp_grant = ui_state.temp_grant_expiration.is_some();
                                        if ui.checkbox(&mut temp_grant, "Temporary").changed() {
                                            if temp_grant {
                                                ui_state.temp_grant_expiration = Some(game_time.current_time + 24.0); // 24 hours
                                            } else {
                                                ui_state.temp_grant_expiration = None;
                                            }
                                        }
                                        ui.end_row();
                                        
                                        if temp_grant {
                                            ui.label("");
                                            let mut hours = ui_state.temp_grant_expiration
                                                .map(|t| t - game_time.current_time)
                                                .unwrap_or(24.0);
                                            
                                            if ui.add(egui::Slider::new(&mut hours, 1.0..=72.0)
                                                .text("Hours")
                                                .logarithmic(true)
                                            ).changed() {
                                                ui_state.temp_grant_expiration = Some(game_time.current_time + hours);
                                            }
                                            ui.end_row();
                                        }
                                        
                                        // Grant button
                                        ui.label("");
                                        if ui.button("Grant Permission").clicked() {
                                            if let Some(local_id) = local_player_agent.as_ref().map(|lpa| lpa.agent_id) {
                                                grant_events.write(PermissionGrantEvent {
                                                    to_agent_id: agent_id,
                                                    from_agent_id: local_id,
                                                    grant: PermissionGrant {
                                                        domain: selected_domain,
                                                        access_level: selected_level,
                                                        grantor: local_id,
                                                        grant_time: game_time.current_time,
                                                        expiration: ui_state.temp_grant_expiration,
                                                        region_limited: None, // Would be set in a real UI
                                                        entity_limited: None, // Would be set in a real UI
                                                        condition: None,      // Would be set in a real UI
                                                    },
                                                });
                                            }
                                        }
                                        ui.end_row();
                                    });
                            }
                        });
                        
                        // Delegated permissions section
                        if !permissions.delegated_to.is_empty() {
                            ui.separator();
                            ui.heading("Delegated Permissions");
                            
                            egui::Grid::new("delegated_permissions_grid")
                                .striped(true)
                                .spacing([5.0, 5.0])
                                .min_col_width(100.0)
                                .show(ui, |ui| {
                                    // Header
                                    ui.label("Delegated To");
                                    ui.label("Domain");
                                    ui.label("Access Level");
                                    ui.label("Expiration");
                                    ui.label("Actions");
                                    ui.end_row();
                                    
                                    // List delegated permissions
                                    for (delegate_id, grants) in &permissions.delegated_to {
                                        for grant in grants {
                                            // Skip if filtered
                                            if let Some(filter_domain) = ui_state.filter_domain {
                                                if grant.domain != filter_domain {
                                                    continue;
                                                }
                                            }
                                            
                                            // Delegated To
                                            ui.label(
                                                agent_manager.get_agent_entity(*delegate_id)
                                                    .and_then(|e| agent_query.get(e).ok())
                                                    .map(|(a, _)| a.name.clone())
                                                    .unwrap_or_else(|| "Unknown".to_string())
                                            );
                                            
                                            // Domain
                                            ui.label(format!("{:?}", grant.domain));
                                            
                                            // Access Level
                                            ui.label(format!("{:?}", grant.access_level));
                                            
                                            // Expiration
                                            if let Some(exp) = grant.expiration {
                                                let time_left = exp - game_time.current_time;
                                                if time_left > 0.0 {
                                                    ui.label(format!("{:.1} hours left", time_left));
                                                } else {
                                                    ui.label("Expired");
                                                }
                                            } else {
                                                ui.label("Never");
                                            }
                                            
                                            // Actions
                                            ui.horizontal(|ui| {
                                                if ui.button("Revoke").clicked() {
                                                    // Check if local player has authority to revoke
                                                    if let Some(local_id) = local_player_agent.as_ref().map(|lpa| lpa.agent_id) {
                                                        revoke_events.write(PermissionRevokeEvent {
                                                            from_agent_id: *delegate_id,
                                                            revoker_id: local_id,
                                                            domain: grant.domain,
                                                        });
                                                    }
                                                }
                                            });
                                            
                                            ui.end_row();
                                        }
                                    }
                                });
                        }
                    }
                }
            } else {
                ui.heading("Select an agent to view permissions");
            }
        });
    Ok(())
}

/// Resource to track the local player's agent
#[derive(Resource)]
pub struct LocalPlayerAgent {
    pub agent_id: EntityId,
}

/// Plugin for agent permissions UI
pub struct AgentPermissionsUiPlugin;

impl Plugin for AgentPermissionsUiPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<PermissionsUiState>()
           .add_message::<TogglePermissionsUiEvent>()
           .add_systems(Update, toggle_permissions_ui)
           .add_systems(EguiPrimaryContextPass, permissions_ui_system);
    }
}