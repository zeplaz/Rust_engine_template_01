//! Viewport requests from UI; committed sizes live in [`crate::render::ResolvedViewports`].

use bevy::math::UVec2;
use bevy::prelude::*;
use bevy_egui::egui;

pub const VIEWPORT_PRIORITY_PREVIEW: u8 = 10;
pub const VIEWPORT_PRIORITY_MINIMAP: u8 = 5;
pub const VIEWPORT_PRIORITY_DEBUG: u8 = 1;

#[derive(Clone, Debug)]
pub struct ViewportRequest {
    pub logical_rect: egui::Rect,
    pub priority: u8,
    pub world_extent: UVec2,
}

#[derive(Clone, Debug)]
pub struct ResolvedViewport {
    pub logical: egui::Rect,
    pub physical: UVec2,
    pub world_extent: UVec2,
}

#[derive(Resource, Default, Clone)]
pub struct ViewportAuthority {
    pub pending: Vec<ViewportRequest>,
    pub requested: Option<ViewportRequest>,
    pub resolved: Option<ResolvedViewport>,
    pub revision: u64,
}

pub fn clear_viewport_requests(mut authority: ResMut<ViewportAuthority>) {
    authority.pending.clear();
    authority.requested = None;
}

pub fn submit_viewport_request(authority: &mut ViewportAuthority, request: ViewportRequest) {
    authority.pending.push(request);
}
