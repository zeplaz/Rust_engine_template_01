//! **Operational strategy** — continuous fields, sparse graphs, and derived blobs.
//!
//! ## Behavior scaffold (safe architecture)
//!
//! - **Layer 1** — [`behavior_entities`]: [`Agent`], [`Faction`], [`AgentFactionLink`].
//! - **Layer 2** — [`behavior_interface`]: [`BehaviorModel`] trait, [`BehaviorContext`], [`DecisionSet`], evaluation hook.
//! - **Authority / tooling** — [`behavior_pressure`] ([`PressureField`]) + [`behavior_script`] / [`behavior_mission`]:
//!   **pressure composition** (climate cards, mission packages), not quest scripts or cutscene triggers.
//! - **Layer 3** — control plane: scenarios / tools swap [`ActiveBehaviorModel`] and read [`DecisionPipelineSink`].
//! - **Pipeline** — [`behavior_pipeline`]: composed score (traits + emotion + faction + script + environment).
//! - **Fracture** — [`behavior_fracture`]: [`FractureEventBus`], meso drift, cohesion pressure, [`SubFactionStub`] hook.
//!
//! # Three spatial systems (do not collapse them)
//!
//! 1. **Static geographic skeleton** — Voronoi [`MacroRegion`](crate::terrain::generation::world_generator_enhanced::MacroRegion),
//!    [`MacroRegionRaster`](crate::terrain::generation::world_generator_enhanced::MacroRegionRaster), terrain semantics
//!    ([`MacroTerrainSemantics`](crate::terrain::generation::polygon_world_semantics::MacroTerrainSemantics)).
//!    Stable IDs, chunk grouping, ecology/climate, saves. Barely moves.
//!
//! 2. **Dynamic operational fields** (this module’s focus) — scalar/vector **heatmaps** per chunk cell: control,
//!    threat, recon certainty, logistics strength, fire/smoke, mobility cost, etc. Updated every tick (or sub-tick).
//!    **Not** polygon ownership. Fronts and contested belts **emerge** from gradients and thresholds on these fields.
//!
//! 3. **Strategic graphs** — roads, rails, pipelines, grids, supply routes, attack axes: sparse, weighted, degradable.
//!    Fields and graphs **couple** (e.g. logistics throughput on edges feeds cell `logistics_strength`).
//!
//! “Regions” in the sense of maneuver warfare are **derived**: flood-fill or level-set on field combinations
//! (`control > 0.65 && threat < τ`), not reassigned provinces.
//!
//! GPU note: layers are dense per chunk; diffusion and updates map cleanly to compute later.
//!
//! **Delivery phases:** `prompts/designer_questions/strategic_platforms/phased_engine_delivery_v1.md` (Phase 1a).
//!
//! **Runbook test rounds:** incremental stubs in [`runbook_rounds`](runbook_rounds.rs) mirror `prompts/guides/*_runbook_v1.md` execution tables.

mod agent_batch_scoring;
mod ai_explainability;
mod behavior_brain_plugin;
mod behavior_emergence_log;
mod behavior_entities;
mod behavior_fracture;
mod behavior_interface;
mod behavior_plugin;
mod behavior_pressure;
mod behavior_script;
mod behavior_mission;
mod behavior_pipeline;
mod build_order;
mod faction_plugin;
mod fracture_plugin;
mod comms_contract;
mod mission_kind;
mod strategic_command_queue;
mod stage7_behavioral;
mod construction_book;
mod frontline;
mod gpu_bridge_plugin;
mod hybrid_brain;
mod hybrid_fields;
mod infrastructure_graph;
mod logistics_net;
mod mission_plugin;
mod narrative_observation;
mod node_field_profile;
mod operational_dependency;
mod plugin;
mod program;
mod runbook_rounds;
mod schedule;
mod simulation_plugin;
mod sim;
mod site;
pub mod settlement;
mod spatial_network;
mod network_flow;
mod strategic_behavior_schedule;
mod transport_bridge;
mod world_field_layers;
mod world_read_snapshot;
mod zones;

pub use behavior_emergence_log::{
    strategic_emergence_log_hybrid_resolution_system, StrategicEmergenceLog,
};
pub use behavior_entities::{
    behavior_sync_entity_ids_system, Agent, AgentFactionLink, Faction,
};
pub use behavior_fracture::{
    faction_cohesion_pressure_system, faction_internal_stage_system,
    faction_meso_internal_tick_system, fracture_event_emit_system, fracture_probability_overlay_system,
    sub_faction_stub_hook_system, FractureDriver, FractureEvent, FractureEventBus, FractureOverlaySettings,
    FractureProbabilityOverlay, FractureSignal, FractureSignalBus, FractureSignalScratch, FractureStageScratch,
    FractureType, SubFactionStub,
};
pub use agent_batch_scoring::{
    agent_batch_cpu_score_system, resolve_agent_action, score_agent_cpu, AgentBatchScoringPlugin,
    AgentCpuBatchScoring, AgentScoreInput, AgentScoreOutput, AgentScoreResult, BatchTacticalChoice,
    GpuAgentScoringPipeline, WorldPressureSample,
};
pub use ai_explainability::{
    decision_explainability_capture_system, format_hybrid_telemetry_explain,
    format_pipeline_contributors, DecisionExplainabilitySnapshot,
};
pub use behavior_mission::{
    active_missions_advance_elapsed_system, active_missions_expire_system,
    mission_success_readout_note, narrative_mission_influence_apply_system,
    pressure_field_from_active_missions_system, ActiveMissions, Mission, MissionId, MissionPressure,
    Objective,
};
pub use build_order::{
    process_build_order_queue_system, ApprovedBuildOrders, BuildOrder, BuildOrderQueue, BuildReason,
    BuildSiteTile, StructureType,
};
pub use comms_contract::{
    BeliefRecord, BeliefSnapshotDto, CommunicationPlane, DispatchEnvelope, DispatchMessage,
    IntelConfidence, MissionIntent, OverlayChannelDescriptor, PlaneAuthority, StrategicOverlayType,
    UtilityChannel,
};
pub use mission_kind::{mission_kinds_supported, MissionKind};
pub use strategic_command_queue::{
    dispatch_delay_ticks, enqueue_strategic_command, tick_strategic_command_queue,
    StrategicCommandQueue, DISPATCH_DELAY_TICKS,
};
pub use stage7_behavioral::{
    ensure_stage7_behavioral_m3_witness_fields, ensure_stage7_m4_play_witness_fields,
    publish_stage7_behavioral_overlay_samples, resolve_stage7_m3_overlay_sample_counts, seed_stage7_behavioral_overlay_resources,
    seed_stage7_behavioral_m2_lib_proof, seed_stage7_behavioral_witness_for_lib_proof,
    seed_stage7_m4_playtest_enqueue, stage7_overlay_reader_sample_counts,
    sync_stage7_overlay_witness_from_reader_samples, tick_strategic_command_queue_system,
    Stage7BehavioralHud,
    Stage7BehavioralPlugin, Stage7BehavioralWitnessState, Stage7BeliefState,
};
pub use behavior_pipeline::{
    compose_decision_score, decision_pipeline_composition_system, sample_decision_components,
    DecisionScoreComponents,
};
pub use behavior_pressure::{PressureField, PressureProfile};
pub use behavior_script::{Condition, IntentChannel, ScriptedIntentWeight, ScriptInfluence};
pub use behavior_interface::{
    behavior_model_evaluation_hook_system, ActiveBehaviorModel, BehaviorContext, BehaviorModel,
    DecisionPipelineSink, DecisionSet, NoopBehaviorModel,
};
#[allow(deprecated)]
pub use behavior_plugin::BehaviorScaffoldPlugin;
pub use behavior_brain_plugin::BehaviorPlugin;
pub use faction_plugin::FactionPlugin;
pub use fracture_plugin::FracturePlugin;
pub use frontline::{derive_frontline_from_control_system, FrontlineState};
pub use gpu_bridge_plugin::{
    AgentGpuPacket, AgentGpuResult, GpuBridgePlugin, GpuBridgeState, GpuSimLane,
};
pub use network_flow::{
    effective_visibility_sample, network_digest_marks_flow_dirty_system,
    network_flow_chunk_local_solver_system, network_insulation_visibility_post_system,
    sample_network_flow_at_world_tile, NetworkDirtyMask, NetworkFlowFieldSample,
    NetworkFlowPrevSignatures, NETWORK_DIRTY_CONNECTIVITY, NETWORK_DIRTY_FLOW,
};
pub use mission_plugin::MissionPlugin;
pub use narrative_observation::{
    narrative_observations_from_runtime_system, NarrativeCategory, NarrativeObservation,
    NarrativeObservationBus,
};
pub use simulation_plugin::SimulationPlugin;
pub use spatial_network::{
    rebuild_chunk_network_digest_system, ChunkNetworkDigest, ChunkNetworkSummary, LayerType,
    NetworkAttachmentMask, NetworkFlowRules, NetworkInsulatedNode, NetworkMembership,
    NetworkPrimitiveKind, NetworkType, SpatialNetworkEdge, SpatialNetworkGraph, SpatialNetworkPlugin,
    SpatialNetworkPosition, SpatialNode,
};
pub use strategic_behavior_schedule::{StrategicBehaviorSchedule, StrategicBehaviorSchedulePlugin};
pub use construction_book::{
    advance_corridor_construction_book_on_sim_tick, advance_corridor_construction_row,
    align_corridor_book_with_transport_directory, apply_corridor_book_from_transport_snapshot,
    corridor_phase_from_wire, corridor_phase_to_wire, corridor_r8_roundtrip_witness_green,
    corridor_sim_tick_writer_witness_green, transport_construction_records_from_book,
    transport_directory_edge_signature, ConstructionPhase, CorridorConstructionBook,
    CorridorConstructionPhase, CorridorConstructionRow, CorridorConstructionStatus,
    CorridorConstructionTickConfig, CorridorEdgeId,
};
pub use site::{
    apply_site_zone_emitters_to_overlays_system,
    commit_construction_site_system,
    evaluate_site_placement_at_world_tile,
    evaluate_site_placement_stubs,
    footprint_affected_chunk_coords,
    site_advance_planned_to_under_construction_system,
    site_construction_progression_system,
    site_phase_from_corridor_coarse,
    site_provisioning_system,
    sync_zone_emitter_from_archetype_system,
    validate_committed_site_terrain_system,
    validate_network_access_for_site,
    validate_site_placement_stubs,
    validate_terrain_for_site,
    zone_emitter_for_archetype,
    BuildingScaleParams,
    CommitConstructionSiteEvent,
    CommittedPlacementSnapshot,
    commit_carries_scale_and_weights_witness_green,
    overlap_blocks_commit_witness_green,
    ConstructionSite,
    FootprintTiles,
    ProceduralBuildingSpec,
    SiteWeightedFootprint,
    TileOccupationBook,
    NetworkMask,
    PlannedSite,
    SiteArchetype,
    SiteConstructionBook,
    SiteConstructionPhase,
    SiteConstructionRate,
    SiteConstructionStatus,
    SiteFootprint,
    SiteId,
    SiteIdIssuer,
    SiteNetworkAttachment,
    SiteOperationalStats,
    SitePlacementValidation,
    SiteResourceManifest,
    SiteTerrainValidation,
    ZoneEmitter,
};
pub use settlement::{
    construction_organic_growth_001_witness_green, set_p5_002_block_assignment_witness_green,
    assign_block_for_tile, register_site_on_commit, AutoBuildPolicy, AutoBuildPolicyBook,
    BlockBook, BlockId, BlockRecord, BuildingUsage, DevelopmentPressure, DevelopmentPressureBook,
    DistrictBook, DistrictId, DistrictMetrics, DistrictMetricsBook, DistrictRecord,
    GrowthActorLayer, GrowthProposal, GrowthProposalGhostState, GrowthProposalQueue,
    GrowthReasonCode, MarketSaturation, MarketSaturationBook, SettlementPlugin, TownBook,
    TownId, TownRecord, ZoningClass,
};
pub use hybrid_brain::{
    apply_agent_intent, agent_decision_score, agent_decision_score_with_world, control_variance,
    fracture_pressure_exceeds, hybrid_action_base_value, hybrid_agent_intent_contribution_system,
    hybrid_emotion_drift_system, hybrid_intent_reset_system, hybrid_phase_clock_tick_system,
    hybrid_resolve_and_feedback_system, perceive_world, perceive_world_biased, resolution_masses,
    resolve_world_state_from_masses, ActionWeights, HybridActionKind, HybridAgentEmotions,
    HybridAgentTraits, HybridBeliefBias, HybridBrainSample, HybridResolutionMasses,
    HybridResolutionTelemetry, HybridSimLastResolved, HybridSimPhaseClock, HybridSimScratch,
    Perception, resolve_world_state, StateControlModel, WorldEvent, WorldIntentField,
};
pub use hybrid_fields::{
    region_stats_spatial_smoothing_system, regional_target_from_world, smooth, RegionStats,
    RegionalStatsOverlay, WorldFields,
};
pub use infrastructure_graph::{
    InfrastructureEdge, InfrastructureGraph, InfrastructureGraphBridgePlugin, InfrastructureNetworkType,
    InfrastructureNode,
};
pub use node_field_profile::{FieldContribution, FieldEmitterParent, NodeRole};
pub use operational_dependency::{
    composite_operational_stress_note, startup_spawn_operational_causality_anchors,
    sync_site_operational_dependency_links_apply_system, trace_operational_cause_chain,
    OperationalCausalityAnchors, OperationalDependencyKind, OperationalDependencyLink,
    OperationalSupplyAnchor,
};
pub use program::StrategicFieldsAndAiPlugin;
pub use schedule::{StrategicOverlayCouplingScratch, StrategicOverlayDisplayPolicy};
pub use sim::{
    CityPlanningHints, HybridSimPipeline, InfrastructureCorridor, LogisticsAiRuntime,
    OperationalTheaterSummary, SettlementSite, SimDebugView, StrategicSimulationPlugin,
    StrategicTransportCorridor,
};

pub use runbook_rounds::city_planning::{
    site_score, utility_redundancy_weight, SettlementArchetype,
};
pub use runbook_rounds::corridor::{
    corridor_capacity_weight, corridor_total_cost, pick_cheaper_corridor_index, CorridorCost,
    CorridorType,
};
pub use runbook_rounds::logistics_ai_policy::{
    demand_forecast, effective_priority_weight, reroute_recommended, LogisticsPriority,
};
pub use runbook_rounds::operational_warfare::{
    doctrine_strike_weight, offensive_commit_score, DroneDoctrine,
    INFRASTRUCTURE_COUPLED_STRIKES_DEFAULT,
};
pub use runbook_rounds::settlement::{
    ecology_hazard_pressure, migration_pull, tier_from_population, SettlementTier,
};

pub use logistics_net::{edge_flow_for_overlay, logistics_net_inject_into_overlays};
pub use plugin::{InfrastructureSiteSet, StrategicFieldPipeline, StrategicFieldsPlugin};
pub use transport_bridge::{rebuild_logistics_graph_from_transport, StrategicRasterConfig};
pub use world_field_layers::{
    ChunkFieldCell, WorldFieldLayerConfig, WorldFieldLayerEpoch,
};
pub use world_read_snapshot::{world_read_snapshot_refresh_system, WorldReadSnapshot};
pub use zones::{apply_zones_to_strategic_overlays_system, Zone, ZoneKind};

use bevy::prelude::{Component, IVec2, Resource, UVec2};

use crate::terrain::ChunkCellKey;

/// Packed faction scalars per cell index (`faction_slot` → value). Runtime maps logical faction id → slot.
pub const MAX_STRATEGIC_FACTION_SLOTS: usize = 16;

/// One simulation cell: **operational** state (many quantities can coexist on the same tile).
///
/// For dense chunk storage prefer SOA via [`ChunkStrategicOverlay`]; this struct documents the logical bundle.
#[derive(Clone, Copy, Debug, Default)]
pub struct StrategicFieldCell {
    pub faction_control: [f32; MAX_STRATEGIC_FACTION_SLOTS],
    pub threat: [f32; MAX_STRATEGIC_FACTION_SLOTS],
    pub recon_confidence: [f32; MAX_STRATEGIC_FACTION_SLOTS],
    pub artillery_danger: [f32; MAX_STRATEGIC_FACTION_SLOTS],
    pub logistics_strength: [f32; MAX_STRATEGIC_FACTION_SLOTS],
    pub fire_risk: f32,
    pub smoke_density: f32,
    pub mobility_cost: f32,
    pub attrition_rate: f32,
    pub civilian_stability: f32,
    /// Local routing congestion 0..1 (transport + field coupling).
    pub routing_congestion: f32,
    /// EW / comms denial proxy 0..1.
    pub ew_denial: f32,
}

/// Chunk-aligned **SOA** operational overlay (same cell ordering as [`crate::terrain::generation::ChunkCellMatrix`] when sizes match).
#[derive(Component, Clone, Debug)]
pub struct ChunkStrategicOverlay {
    pub chunk_coord: IVec2,
    pub size: UVec2,
    pub faction_control: Vec<[f32; MAX_STRATEGIC_FACTION_SLOTS]>,
    pub threat: Vec<[f32; MAX_STRATEGIC_FACTION_SLOTS]>,
    pub recon_confidence: Vec<[f32; MAX_STRATEGIC_FACTION_SLOTS]>,
    pub artillery_danger: Vec<[f32; MAX_STRATEGIC_FACTION_SLOTS]>,
    pub logistics_strength: Vec<[f32; MAX_STRATEGIC_FACTION_SLOTS]>,
    pub logistics_throughput: Vec<f32>,
    /// Network-flow solver: power grid / energy availability (0..1).
    pub power_flow: Vec<f32>,
    /// Network-flow solver: roads, pipes-as-pressure, logistics lanes (0..1).
    pub logistics_flow: Vec<f32>,
    /// Blended control from faction slots + logistics flow (0..1).
    pub control_pressure: Vec<f32>,
    /// Sensor / data-network derived visibility channel (0..1); modulated by EW + insulation pass.
    pub visibility: Vec<f32>,
    pub mobility_cost: Vec<f32>,
    pub attrition_rate: Vec<f32>,
    pub fire_risk: Vec<f32>,
    pub smoke_density: Vec<f32>,
    pub civilian_stability: Vec<f32>,
    /// Routing stress from transport endpoints (R5+ channel).
    pub routing_congestion: Vec<f32>,
    /// EW / GNSS denial proxy field.
    pub ew_denial: Vec<f32>,
}

impl ChunkStrategicOverlay {
    pub fn new(chunk_coord: IVec2, size: UVec2) -> Self {
        let n = (size.x as usize).saturating_mul(size.y as usize);
        let z_pack = || vec![[0.0; MAX_STRATEGIC_FACTION_SLOTS]; n];
        let z_scalar = || vec![0.0; n];
        Self {
            chunk_coord,
            size,
            faction_control: z_pack(),
            threat: z_pack(),
            recon_confidence: z_pack(),
            artillery_danger: z_pack(),
            logistics_strength: z_pack(),
            logistics_throughput: z_scalar(),
            power_flow: z_scalar(),
            logistics_flow: z_scalar(),
            control_pressure: z_scalar(),
            visibility: z_scalar(),
            mobility_cost: z_scalar(),
            attrition_rate: z_scalar(),
            fire_risk: z_scalar(),
            smoke_density: z_scalar(),
            civilian_stability: z_scalar(),
            routing_congestion: z_scalar(),
            ew_denial: z_scalar(),
        }
    }

    #[inline]
    pub fn len_cells(&self) -> usize {
        self.faction_control.len()
    }

    /// Write **per-faction** threat for one cell (`strategic_overlay` runbook — faction slots / field writers).
    pub fn set_faction_threat(
        &mut self,
        cell: usize,
        faction_slot: usize,
        value: f32,
    ) -> Result<(), ()> {
        if cell >= self.threat.len() || faction_slot >= MAX_STRATEGIC_FACTION_SLOTS {
            return Err(());
        }
        self.threat[cell][faction_slot] = value;
        Ok(())
    }

    /// Recon certainty for a faction slot (overlay writer stub).
    pub fn set_recon_confidence(
        &mut self,
        cell: usize,
        faction_slot: usize,
        value: f32,
    ) -> Result<(), ()> {
        if cell >= self.recon_confidence.len() || faction_slot >= MAX_STRATEGIC_FACTION_SLOTS {
            return Err(());
        }
        self.recon_confidence[cell][faction_slot] = value;
        Ok(())
    }

    /// Artillery danger heat per faction slot.
    pub fn set_artillery_danger(
        &mut self,
        cell: usize,
        faction_slot: usize,
        value: f32,
    ) -> Result<(), ()> {
        if cell >= self.artillery_danger.len() || faction_slot >= MAX_STRATEGIC_FACTION_SLOTS {
            return Err(());
        }
        self.artillery_danger[cell][faction_slot] = value;
        Ok(())
    }
}

#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq)]
pub struct LogisticsNodeId(pub u32);

#[derive(Clone, Debug)]
pub struct LogisticsNode {
    pub id: LogisticsNodeId,
    pub throughput: f32,
    pub stockpile: f32,
    /// Chunk cell for this junction / depot; edges inject flow at anchored endpoints.
    pub anchor: Option<ChunkCellKey>,
}

#[derive(Clone, Debug)]
pub struct LogisticsEdge {
    pub from: LogisticsNodeId,
    pub to: LogisticsNodeId,
    /// Authoritative transport edge (LOG-A-03); `None` only for portal stub edges.
    pub transport_edge: Option<crate::systems::transport::TransportEdgeId>,
    pub capacity: f32,
    pub disruption: f32,
    pub traversal_cost: f32,
}

/// **Derived cache only** — rebuilt at `GraphSync`; never mutated during freight solve.
#[derive(Resource, Clone, Debug, Default)]
pub struct LogisticsGraph {
    pub revision: u64,
    pub nodes: Vec<LogisticsNode>,
    pub edges: Vec<LogisticsEdge>,
}
