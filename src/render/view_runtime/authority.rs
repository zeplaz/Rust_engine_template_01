use std::collections::HashMap;

use bevy::prelude::*;

use crate::gui::ViewInstance;
use crate::gui::ViewCameraState;

use super::ids::{ViewIsolationGroup, ViewSurfaceId};
use super::layers::{InteractionViewportState, OverlayViewportPolicy, RenderViewportContract};
use super::surface::ViewSurface;

/// Who last committed a surface field (VM-A tracing).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum ViewAuthorityWriter {
    ViewportPipeline,
    MapCameraInput,
    MinimapFollow,
    MinimapShell,
    PreviewPanel,
    BridgeCompat,
    Unset,
}

/// Sole commit bus for per-surface pose + render contracts (VM-A scaffold).
#[derive(Resource, Default, Debug)]
pub struct ViewProjectionAuthority {
    pub surfaces: HashMap<ViewSurfaceId, ViewSurface>,
    pub last_commit_revision: u64,
    pub last_pose_writer: HashMap<ViewSurfaceId, ViewAuthorityWriter>,
    pub last_render_writer: HashMap<ViewSurfaceId, ViewAuthorityWriter>,
}

impl ViewProjectionAuthority {
    pub fn commit_pose(
        &mut self,
        id: ViewSurfaceId,
        camera: ViewCameraState,
        writer: ViewAuthorityWriter,
    ) {
        let prev_zoom = self.surfaces.get(&id).map(|s| s.camera.zoom);
        if let Some(surface) = self.surfaces.get_mut(&id) {
            surface.camera = camera;
        } else {
            self.surfaces.insert(
                id,
                ViewSurface {
                    id,
                    group: super::ids::default_isolation_group(id),
                    camera_entity: Entity::PLACEHOLDER,
                    semantic: None,
                    render: RenderViewportContract::default(),
                    interaction: InteractionViewportState::default(),
                    overlay: OverlayViewportPolicy::default(),
                    camera,
                    render_policy: crate::gui::ViewRenderPolicy::default(),
                },
            );
        }
        self.last_pose_writer.insert(id, writer);
        self.last_commit_revision = self.last_commit_revision.saturating_add(1);
        if id == ViewSurfaceId::WorldMain {
            crate::gui::on_world_main_pose_committed(writer, prev_zoom, camera.zoom);
        }
    }

    /// Update render contract + presentation policy — **never** overwrites gameplay pose.
    pub fn commit_bridge_render_policy(
        &mut self,
        id: ViewSurfaceId,
        group: ViewIsolationGroup,
        camera_entity: Entity,
        render: RenderViewportContract,
        render_policy: crate::gui::ViewRenderPolicy,
        inst: &ViewInstance,
    ) {
        let gameplay_pose = self
            .last_pose_writer
            .get(&id)
            .copied()
            .is_some_and(|w| w == ViewAuthorityWriter::MapCameraInput);

        if let Some(surface) = self.surfaces.get_mut(&id) {
            surface.render = render;
            surface.render_policy = render_policy;
            surface.camera_entity = camera_entity;
            surface.interaction.pan_delta = inst.interaction_state.pan_delta;
            surface.interaction.zoom_factor = inst.interaction_state.zoom_factor;
            if !gameplay_pose {
                surface.camera = inst.camera;
            }
            self.last_render_writer
                .insert(id, ViewAuthorityWriter::BridgeCompat);
            self.last_commit_revision = self.last_commit_revision.saturating_add(1);
            return;
        }

        let camera = self
            .surface(ViewSurfaceId::WorldMain)
            .map(|s| s.camera)
            .unwrap_or(inst.camera);
        let surface = ViewSurface {
            id,
            group,
            camera_entity,
            semantic: None,
            render,
            interaction: InteractionViewportState {
                captured: false,
                pan_delta: inst.interaction_state.pan_delta,
                zoom_factor: inst.interaction_state.zoom_factor,
            },
            overlay: OverlayViewportPolicy {
                allow_debug_outline: inst.render_policy.debug_flags.show_viewport_outline,
                allow_construction_ghost: false,
            },
            camera,
            render_policy,
        };
        self.surfaces.insert(id, surface);
        self.last_render_writer
            .insert(id, ViewAuthorityWriter::BridgeCompat);
        self.last_commit_revision = self.last_commit_revision.saturating_add(1);
    }

    pub fn commit_render_contract(
        &mut self,
        id: ViewSurfaceId,
        render: RenderViewportContract,
        writer: ViewAuthorityWriter,
    ) {
        if let Some(surface) = self.surfaces.get_mut(&id) {
            surface.render = render;
        } else {
            let camera = self
                .surfaces
                .get(&ViewSurfaceId::WorldMain)
                .map(|s| s.camera)
                .unwrap_or_default();
            self.surfaces.insert(
                id,
                ViewSurface {
                    id,
                    group: super::ids::default_isolation_group(id),
                    camera_entity: Entity::PLACEHOLDER,
                    semantic: None,
                    render,
                    interaction: InteractionViewportState::default(),
                    overlay: OverlayViewportPolicy::default(),
                    camera,
                    render_policy: crate::gui::ViewRenderPolicy::default(),
                },
            );
        }
        self.last_render_writer.insert(id, writer);
        self.last_commit_revision = self.last_commit_revision.saturating_add(1);
    }

    #[must_use]
    pub fn surface(&self, id: ViewSurfaceId) -> Option<&ViewSurface> {
        self.surfaces.get(&id)
    }

    pub fn upsert_from_view_instance(
        &mut self,
        id: ViewSurfaceId,
        group: ViewIsolationGroup,
        inst: &ViewInstance,
        render: RenderViewportContract,
        writer: ViewAuthorityWriter,
    ) {
        let prior_pose_writer = self.last_pose_writer.get(&id).copied();
        let prior_camera = self.surfaces.get(&id).map(|s| s.camera);
        // BridgeCompat must never overwrite pose committed by gameplay / viewport writers.
        let camera = match prior_pose_writer {
            Some(w) if w != ViewAuthorityWriter::BridgeCompat && w != ViewAuthorityWriter::Unset => {
                prior_camera.unwrap_or(inst.camera)
            }
            _ => inst.camera,
        };
        let pose_writer = match prior_pose_writer {
            Some(w) if w != ViewAuthorityWriter::BridgeCompat && w != ViewAuthorityWriter::Unset => w,
            _ => writer,
        };
        let surface = ViewSurface {
            id,
            group,
            camera_entity: inst.camera_entity,
            semantic: None,
            render,
            interaction: InteractionViewportState {
                captured: false,
                pan_delta: inst.interaction_state.pan_delta,
                zoom_factor: inst.interaction_state.zoom_factor,
            },
            overlay: OverlayViewportPolicy {
                allow_debug_outline: inst.render_policy.debug_flags.show_viewport_outline,
                allow_construction_ghost: false,
            },
            camera,
            render_policy: inst.render_policy.clone(),
        };
        self.surfaces.insert(id, surface);
        self.last_pose_writer.insert(id, pose_writer);
        self.last_render_writer.insert(id, ViewAuthorityWriter::BridgeCompat);
        self.last_commit_revision = self.last_commit_revision.saturating_add(1);
    }

    pub fn commit_pose_traced(
        &mut self,
        id: ViewSurfaceId,
        camera: ViewCameraState,
        writer: ViewAuthorityWriter,
        trace: Option<&mut super::trace::ViewRuntimeTrace>,
    ) {
        if let Some(t) = trace {
            if let Some(prev) = self.last_pose_writer.get(&id) {
                if *prev != writer && *prev != ViewAuthorityWriter::Unset {
                    t.push_violation(super::trace::ViewViolationKind::DualWriterPose);
                }
            }
            t.record(id, writer, "commit_pose");
        }
        self.commit_pose(id, camera, writer);
    }
}
