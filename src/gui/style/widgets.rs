//! Shared Bevy UI widget kit — panel, header, rows, buttons, toggle, stepper, scroll, tooltip.
//!
//! One look for every menu (world generator, build toolbox / picker / sheets). Callers spawn
//! once and mutate [`UiButton`] state in place; [`UiWidgetsPlugin`] paints idle / hover /
//! pressed / selected / disabled from [`UiPalette`] tokens. Action systems must ignore a
//! pressed button when [`UiButton::disabled`] is set (see [`ui_button_fired`]).

use bevy::ecs::hierarchy::ChildSpawnerCommands;
use bevy::input::mouse::{AccumulatedMouseScroll, MouseScrollUnit};
use bevy::prelude::*;
use bevy::ui::{FocusPolicy, RelativeCursorPosition};
use bevy::window::PrimaryWindow;

use super::UiPalette;

pub const FONT_TITLE: f32 = 14.0;
pub const FONT_BODY: f32 = 12.0;
pub const FONT_SMALL: f32 = 10.0;
pub const ROW_H: f32 = 26.0;
pub const PANEL_PAD: f32 = 8.0;
pub const GAP: f32 = 4.0;
pub const SECTION_GAP: f32 = 8.0;
pub const SCROLL_LINE_PX: f32 = 28.0;

/// Visual family for a [`UiButton`].
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum UiButtonVariant {
    #[default]
    Normal,
    /// Main call to action (generate, build).
    Primary,
    /// Destructive (discard, demolish).
    Danger,
    /// Category tab — selected shows a gold frame instead of an accent fill.
    Tab,
}

/// Mutable button state. Spawn once; flip `selected` / `disabled` in place.
#[derive(Component, Clone, Copy, Debug, Default)]
pub struct UiButton {
    pub variant: UiButtonVariant,
    pub selected: bool,
    pub disabled: bool,
}

impl UiButton {
    #[must_use]
    pub const fn new(variant: UiButtonVariant) -> Self {
        Self { variant, selected: false, disabled: false }
    }

    #[must_use]
    pub const fn selected(mut self, on: bool) -> Self {
        self.selected = on;
        self
    }

    #[must_use]
    pub const fn disabled(mut self, off: bool) -> Self {
        self.disabled = off;
        self
    }

    /// Set state in place; returns true when it changed (avoids needless change detection).
    pub fn set(&mut self, selected: bool, disabled: bool) -> bool {
        let changed = self.selected != selected || self.disabled != disabled;
        if changed {
            self.selected = selected;
            self.disabled = disabled;
        }
        changed
    }
}

/// True when `interaction` is a fresh press on an enabled button.
#[must_use]
pub fn ui_button_fired(interaction: &Interaction, button: &UiButton) -> bool {
    *interaction == Interaction::Pressed && !button.disabled
}

/// Text node inside a button — recoloured with the button state.
#[derive(Component)]
pub struct UiButtonLabel;

/// Check box square inside a toggle — filled while the toggle is on.
#[derive(Component)]
pub struct UiToggleBox;

/// Value bar fill inside a stepper; width follows [`UiSlider::fraction`].
#[derive(Component)]
pub struct UiMeterFill;

/// Click / drag track under a stepper. Callers write `fraction` (0..1) from their value and
/// read drags with [`ui_slider_drag`].
#[derive(Component, Clone, Copy, Debug, Default, PartialEq)]
pub struct UiSlider {
    pub fraction: f32,
}

/// Fraction (0..1) under the cursor while the slider track is held, else `None`.
#[must_use]
pub fn ui_slider_drag(interaction: &Interaction, cursor: &RelativeCursorPosition) -> Option<f32> {
    if *interaction != Interaction::Pressed {
        return None;
    }
    cursor.normalized.map(|n| (n.x + 0.5).clamp(0.0, 1.0))
}

/// Mouse-wheel scrollable column (needs `overflow: scroll_y`).
#[derive(Component)]
pub struct UiScroll;

/// Hover text, shown near the cursor by [`UiWidgetsPlugin`].
#[derive(Component, Clone, Debug)]
pub struct UiTooltip(pub String);

#[derive(Component)]
struct UiTooltipRoot;

#[derive(Component)]
struct UiTooltipText;

/// Palette + font handle bundle passed to every spawn helper.
#[derive(Clone)]
pub struct UiKit<'a> {
    pub palette: &'a UiPalette,
    pub font: Handle<Font>,
}

impl<'a> UiKit<'a> {
    #[must_use]
    pub fn new(palette: &'a UiPalette, font: Handle<Font>) -> Self {
        Self { palette, font }
    }

    #[must_use]
    pub fn text_font(&self, size: f32) -> TextFont {
        TextFont::from_font_size(size).with_font(self.font.clone())
    }

    fn rgba(c: crate::gui::ui_types::UiRgba) -> Color {
        c.to_bevy_color()
    }
}

fn mix(a: Color, b: Color, t: f32) -> Color {
    let a = a.to_srgba();
    let b = b.to_srgba();
    let t = t.clamp(0.0, 1.0);
    Color::srgba(
        a.red + (b.red - a.red) * t,
        a.green + (b.green - a.green) * t,
        a.blue + (b.blue - a.blue) * t,
        a.alpha + (b.alpha - a.alpha) * t,
    )
}

/// Resolved (background, border, text) for a button state.
#[must_use]
pub fn button_colors(
    palette: &UiPalette,
    button: &UiButton,
    interaction: Interaction,
) -> (Color, Color, Color) {
    let base = UiKit::rgba(palette.bg_interactive);
    let fg = UiKit::rgba(palette.fg_primary);
    let muted = UiKit::rgba(palette.fg_muted);
    let border = palette.bevy_border_subtle();
    if button.disabled {
        return (
            UiKit::rgba(palette.bg_elevated),
            mix(border, Color::BLACK, 0.5),
            mix(muted, Color::BLACK, 0.35),
        );
    }
    let (accent, accent_text) = match button.variant {
        UiButtonVariant::Primary => (UiKit::rgba(palette.accent_terminal), UiKit::rgba(palette.accent_terminal)),
        UiButtonVariant::Danger => (UiKit::rgba(palette.danger), UiKit::rgba(palette.danger)),
        _ => (fg, fg),
    };
    let (mut bg, mut bd, mut tx) = (base, border, accent_text);
    if button.variant == UiButtonVariant::Normal || button.variant == UiButtonVariant::Tab {
        tx = if button.variant == UiButtonVariant::Tab { muted } else { fg };
    }
    if button.variant == UiButtonVariant::Primary || button.variant == UiButtonVariant::Danger {
        bd = mix(border, accent, 0.7);
    }
    if button.selected {
        if button.variant == UiButtonVariant::Tab {
            bg = UiKit::rgba(palette.selection_bg);
            bg = mix(base, bg, 0.9);
            bd = palette.bevy_accent_gold();
            tx = fg;
        } else {
            bg = palette.bevy_accent_action();
            bd = palette.bevy_accent_gold();
            tx = UiKit::rgba(palette.fg_on_accent);
        }
    }
    match interaction {
        Interaction::Hovered => {
            bg = mix(bg, fg, 0.14);
            bd = if button.selected { bd } else { fg };
        }
        Interaction::Pressed => {
            bg = mix(bg, UiKit::rgba(palette.accent_terminal), 0.35);
            bd = palette.bevy_accent_gold();
        }
        Interaction::None => {}
    }
    (bg, bd, tx)
}

/// Frame bundle for a floating panel. Caller adds `position_type`, offsets, `ZIndex`.
#[must_use]
pub fn panel_bundle(palette: &UiPalette, mut node: Node) -> impl Bundle {
    node.padding = UiRect::all(Val::Px(PANEL_PAD));
    node.border = UiRect::all(Val::Px(1.0));
    node.flex_direction = FlexDirection::Column;
    node.row_gap = Val::Px(GAP);
    (
        node,
        BackgroundColor(palette.bevy_hud_panel_fill()),
        BorderColor::all(palette.bevy_border_subtle()),
        FocusPolicy::Block,
    )
}

/// Title row: title text (with caller marker) and an optional close button.
pub fn spawn_header(
    parent: &mut ChildSpawnerCommands<'_>,
    kit: &UiKit,
    title: &str,
    title_marker: impl Bundle,
    close_marker: Option<impl Bundle>,
) {
    parent
        .spawn(Node {
            width: Val::Percent(100.0),
            flex_direction: FlexDirection::Row,
            justify_content: JustifyContent::SpaceBetween,
            align_items: AlignItems::Center,
            column_gap: Val::Px(GAP),
            padding: UiRect::bottom(Val::Px(GAP)),
            border: UiRect::bottom(Val::Px(1.0)),
            ..default()
        })
        .insert(BorderColor::all(kit.palette.bevy_border_subtle()))
        .with_children(|row| {
            row.spawn((
                Text::new(title),
                kit.text_font(FONT_TITLE),
                TextColor(kit.palette.bevy_primary_text()),
                title_marker,
            ));
            if let Some(close) = close_marker {
                spawn_button(row, kit, "X", UiButton::new(UiButtonVariant::Normal), close)
                    .insert((
                        Node {
                            width: Val::Px(ROW_H),
                            height: Val::Px(ROW_H),
                            justify_content: JustifyContent::Center,
                            align_items: AlignItems::Center,
                            border: UiRect::all(Val::Px(1.0)),
                            ..default()
                        },
                        UiTooltip("Close (Esc)".to_string()),
                    ));
            }
        });
}

/// Small-caps section divider.
pub fn spawn_section_header(parent: &mut ChildSpawnerCommands<'_>, kit: &UiKit, label: &str) {
    parent.spawn((
        Text::new(label.to_uppercase()),
        kit.text_font(FONT_SMALL),
        TextColor(kit.palette.bevy_accent_terminal()),
        Node {
            width: Val::Percent(100.0),
            margin: UiRect::top(Val::Px(SECTION_GAP)),
            padding: UiRect::bottom(Val::Px(2.0)),
            border: UiRect::bottom(Val::Px(1.0)),
            ..default()
        },
        BorderColor::all(kit.palette.bevy_border_subtle()),
    ));
}

/// Plain wrapped text line (muted when `muted`).
pub fn spawn_note(
    parent: &mut ChildSpawnerCommands<'_>,
    kit: &UiKit,
    text: &str,
    muted: bool,
    marker: impl Bundle,
) {
    parent.spawn((
        Text::new(text),
        kit.text_font(FONT_SMALL),
        TextColor(if muted {
            kit.palette.bevy_text_muted()
        } else {
            kit.palette.bevy_primary_text()
        }),
        Node { width: Val::Percent(100.0), ..default() },
        marker,
    ));
}

/// `label ........ value` row; `value_marker` lets callers update the right-hand text.
pub fn spawn_label_row(
    parent: &mut ChildSpawnerCommands<'_>,
    kit: &UiKit,
    label: &str,
    value: &str,
    value_marker: impl Bundle,
) {
    parent
        .spawn(Node {
            width: Val::Percent(100.0),
            min_height: Val::Px(ROW_H - 4.0),
            flex_direction: FlexDirection::Row,
            justify_content: JustifyContent::SpaceBetween,
            align_items: AlignItems::Center,
            column_gap: Val::Px(SECTION_GAP),
            ..default()
        })
        .with_children(|row| {
            row.spawn((
                Text::new(label),
                kit.text_font(FONT_BODY),
                TextColor(kit.palette.bevy_text_muted()),
            ));
            row.spawn((
                Text::new(value),
                kit.text_font(FONT_BODY),
                TextColor(kit.palette.bevy_primary_text()),
                value_marker,
            ));
        });
}

/// Button with hover / pressed / selected / disabled painting. Returns the button entity.
pub fn spawn_button<'a>(
    parent: &'a mut ChildSpawnerCommands<'_>,
    kit: &UiKit,
    label: &str,
    state: UiButton,
    marker: impl Bundle,
) -> EntityCommands<'a> {
    spawn_button_with(parent, kit, label, state, marker, |_| {})
}

/// [`spawn_button`] with extra children (icon, glyph) placed before the label.
pub fn spawn_button_with<'a>(
    parent: &'a mut ChildSpawnerCommands<'_>,
    kit: &UiKit,
    label: &str,
    state: UiButton,
    marker: impl Bundle,
    leading: impl FnOnce(&mut ChildSpawnerCommands<'_>),
) -> EntityCommands<'a> {
    let (bg, bd, tx) = button_colors(kit.palette, &state, Interaction::None);
    let mut ec = parent.spawn((
        Button,
        state,
        marker,
        Node {
            min_height: Val::Px(ROW_H),
            padding: UiRect::axes(Val::Px(SECTION_GAP), Val::Px(GAP)),
            border: UiRect::all(Val::Px(1.0)),
            column_gap: Val::Px(GAP),
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            ..default()
        },
        BackgroundColor(bg),
        BorderColor::all(bd),
        FocusPolicy::Block,
    ));
    ec.with_children(|b| {
        leading(b);
        b.spawn((
            UiButtonLabel,
            Text::new(label),
            kit.text_font(FONT_BODY),
            TextColor(tx),
        ));
    });
    ec
}

/// Full-width, left-aligned list row button (catalog pick, tool row).
pub fn spawn_list_button<'a>(
    parent: &'a mut ChildSpawnerCommands<'_>,
    kit: &UiKit,
    label: &str,
    state: UiButton,
    marker: impl Bundle,
) -> EntityCommands<'a> {
    let mut ec = spawn_button(parent, kit, label, state, marker);
    ec.insert(Node {
        width: Val::Percent(100.0),
        min_height: Val::Px(ROW_H + 6.0),
        padding: UiRect::axes(Val::Px(SECTION_GAP), Val::Px(GAP)),
        border: UiRect::all(Val::Px(1.0)),
        justify_content: JustifyContent::FlexStart,
        align_items: AlignItems::Center,
        ..default()
    });
    ec
}

/// Check-box toggle: `[x] label`. Selected state is [`UiButton::selected`].
pub fn spawn_toggle<'a>(
    parent: &'a mut ChildSpawnerCommands<'_>,
    kit: &UiKit,
    label: &str,
    on: bool,
    marker: impl Bundle,
) -> EntityCommands<'a> {
    let state = UiButton::new(UiButtonVariant::Normal);
    let mut ec = parent.spawn((
        Button,
        state,
        ToggleState(on),
        marker,
        Node {
            width: Val::Percent(100.0),
            min_height: Val::Px(ROW_H),
            padding: UiRect::axes(Val::Px(SECTION_GAP), Val::Px(GAP)),
            border: UiRect::all(Val::Px(1.0)),
            column_gap: Val::Px(SECTION_GAP),
            align_items: AlignItems::Center,
            ..default()
        },
        BackgroundColor(kit.palette.bevy_button_idle()),
        BorderColor::all(kit.palette.bevy_border_subtle()),
        FocusPolicy::Block,
    ));
    ec.with_children(|b| {
        b.spawn((
            UiToggleBox,
            Node {
                width: Val::Px(12.0),
                height: Val::Px(12.0),
                border: UiRect::all(Val::Px(1.0)),
                ..default()
            },
            BackgroundColor(Color::NONE),
            BorderColor::all(kit.palette.bevy_primary_text()),
        ));
        b.spawn((
            UiButtonLabel,
            Text::new(label),
            kit.text_font(FONT_BODY),
            TextColor(kit.palette.bevy_primary_text()),
        ));
    });
    ec
}

/// Authoritative on/off for a toggle; the plugin mirrors it into the check box.
#[derive(Component, Clone, Copy, Debug)]
pub struct ToggleState(pub bool);

/// `label  [-] value [+]` with a click/drag slider underneath. Markers are caller-owned;
/// `slider` lands on the track entity (which carries [`UiSlider`]).
pub fn spawn_stepper(
    parent: &mut ChildSpawnerCommands<'_>,
    kit: &UiKit,
    label: &str,
    minus: impl Bundle,
    plus: impl Bundle,
    readout: impl Bundle,
    initial: &str,
    slider: impl Bundle,
    fraction: f32,
) {
    parent
        .spawn(Node {
            width: Val::Percent(100.0),
            flex_direction: FlexDirection::Column,
            row_gap: Val::Px(2.0),
            ..default()
        })
        .with_children(|col| {
            col.spawn(Node {
                width: Val::Percent(100.0),
                flex_direction: FlexDirection::Row,
                align_items: AlignItems::Center,
                column_gap: Val::Px(GAP),
                ..default()
            })
            .with_children(|row| {
                row.spawn((
                    Text::new(label),
                    kit.text_font(FONT_BODY),
                    TextColor(kit.palette.bevy_text_muted()),
                    Node { flex_grow: 1.0, flex_shrink: 1.0, ..default() },
                ));
                spawn_button(row, kit, "-", UiButton::new(UiButtonVariant::Normal), minus)
                    .insert(Node {
                        width: Val::Px(ROW_H - 4.0),
                        height: Val::Px(ROW_H - 4.0),
                        padding: UiRect::ZERO,
                        border: UiRect::all(Val::Px(1.0)),
                        justify_content: JustifyContent::Center,
                        align_items: AlignItems::Center,
                        ..default()
                    });
                row.spawn((
                    Text::new(initial),
                    kit.text_font(FONT_BODY),
                    TextColor(kit.palette.bevy_primary_text()),
                    Node { width: Val::Px(64.0), justify_content: JustifyContent::Center, ..default() },
                    TextLayout::justify(Justify::Center),
                    readout,
                ));
                spawn_button(row, kit, "+", UiButton::new(UiButtonVariant::Normal), plus)
                    .insert(Node {
                        width: Val::Px(ROW_H - 4.0),
                        height: Val::Px(ROW_H - 4.0),
                        padding: UiRect::ZERO,
                        border: UiRect::all(Val::Px(1.0)),
                        justify_content: JustifyContent::Center,
                        align_items: AlignItems::Center,
                        ..default()
                    });
            });
            col.spawn((
                UiSlider { fraction: fraction.clamp(0.0, 1.0) },
                slider,
                Interaction::default(),
                RelativeCursorPosition::default(),
                FocusPolicy::Block,
                Node {
                    width: Val::Percent(100.0),
                    height: Val::Px(8.0),
                    border: UiRect::all(Val::Px(1.0)),
                    ..default()
                },
                BackgroundColor(mix(kit.palette.bevy_button_idle(), Color::BLACK, 0.3)),
                BorderColor::all(kit.palette.bevy_border_subtle()),
            ))
            .with_children(|track| {
                track.spawn((
                    UiMeterFill,
                    Node {
                        width: Val::Percent(fraction.clamp(0.0, 1.0) * 100.0),
                        height: Val::Percent(100.0),
                        ..default()
                    },
                    BackgroundColor(kit.palette.bevy_accent_terminal()),
                ));
            });
        });
}

/// Scrollable column: wheel scrolls it while the cursor is over it.
#[must_use]
pub fn scroll_column_bundle(mut node: Node) -> impl Bundle {
    node.flex_direction = FlexDirection::Column;
    node.overflow = Overflow::scroll_y();
    (
        UiScroll,
        node,
        ScrollPosition::default(),
        RelativeCursorPosition::default(),
    )
}

/// Right-edge hint for any control (e.g. valid range).
pub fn spawn_hint_row(parent: &mut ChildSpawnerCommands<'_>, kit: &UiKit, text: &str) {
    parent.spawn((
        Text::new(text),
        kit.text_font(FONT_SMALL),
        TextColor(kit.palette.bevy_text_muted()),
    ));
}

fn sync_button_visuals(
    palette: Res<UiPalette>,
    mut buttons: Query<
        (&Interaction, &UiButton, &mut BackgroundColor, &mut BorderColor, Option<&Children>),
        Without<UiButtonLabel>,
    >,
    mut labels: Query<&mut TextColor, With<UiButtonLabel>>,
) {
    for (interaction, button, mut bg, mut border, children) in &mut buttons {
        let (b, bd, tx) = button_colors(&palette, button, *interaction);
        if bg.0 != b {
            bg.0 = b;
        }
        let next = BorderColor::all(bd);
        if *border != next {
            *border = next;
        }
        if let Some(children) = children {
            for child in children.iter() {
                if let Ok(mut c) = labels.get_mut(child) {
                    if c.0 != tx {
                        c.0 = tx;
                    }
                }
            }
        }
    }
}

fn sync_toggle_visuals(
    palette: Res<UiPalette>,
    mut boxes: Query<(&ChildOf, &mut BackgroundColor), With<UiToggleBox>>,
    states: Query<&ToggleState>,
) {
    let on_color = palette.bevy_accent_terminal();
    for (parent, mut bg) in &mut boxes {
        let on = states.get(parent.parent()).map(|s| s.0).unwrap_or(false);
        let next = if on { on_color } else { Color::NONE };
        if bg.0 != next {
            bg.0 = next;
        }
    }
}

fn sync_slider_fill(
    sliders: Query<(&UiSlider, &Children), Changed<UiSlider>>,
    mut fills: Query<&mut Node, With<UiMeterFill>>,
) {
    for (slider, children) in &sliders {
        for child in children.iter() {
            if let Ok(mut node) = fills.get_mut(child) {
                let w = Val::Percent(slider.fraction.clamp(0.0, 1.0) * 100.0);
                if node.width != w {
                    node.width = w;
                }
            }
        }
    }
}

fn scroll_wheel(
    scroll: Option<Res<AccumulatedMouseScroll>>,
    mut roots: Query<(&RelativeCursorPosition, &mut ScrollPosition), With<UiScroll>>,
) {
    let Some(scroll) = scroll else { return };
    let dy = match scroll.unit {
        MouseScrollUnit::Line => scroll.delta.y * SCROLL_LINE_PX,
        MouseScrollUnit::Pixel => scroll.delta.y,
    };
    if dy == 0.0 {
        return;
    }
    for (cursor, mut pos) in &mut roots {
        if cursor.cursor_over() {
            pos.0.y = (pos.0.y - dy).max(0.0);
        }
    }
}

fn spawn_tooltip_root(mut commands: Commands, palette: Res<UiPalette>, assets: Res<AssetServer>) {
    let font = assets.load::<Font>("fonts/FiraMono-Medium.ttf");
    commands
        .spawn((
            UiTooltipRoot,
            Node {
                position_type: PositionType::Absolute,
                left: Val::Px(0.0),
                top: Val::Px(0.0),
                max_width: Val::Px(260.0),
                padding: UiRect::axes(Val::Px(GAP + 2.0), Val::Px(GAP)),
                border: UiRect::all(Val::Px(1.0)),
                ..default()
            },
            BackgroundColor(mix(palette.bevy_hud_panel_fill(), Color::BLACK, 0.4)),
            BorderColor::all(palette.bevy_accent_gold()),
            FocusPolicy::Pass,
            Visibility::Hidden,
            GlobalZIndex(2000),
            Name::new("ui_tooltip"),
        ))
        .with_children(|root| {
            root.spawn((
                UiTooltipText,
                Text::new(""),
                TextFont::from_font_size(FONT_SMALL).with_font(font),
                TextColor(palette.bevy_primary_text()),
            ));
        });
}

fn sync_tooltip(
    windows: Query<&Window, With<PrimaryWindow>>,
    hovered: Query<(&Interaction, &UiTooltip)>,
    mut root: Query<(&mut Node, &mut Visibility), With<UiTooltipRoot>>,
    mut text: Query<&mut Text, With<UiTooltipText>>,
) {
    let Ok((mut node, mut vis)) = root.single_mut() else { return };
    let tip = hovered
        .iter()
        .find(|(i, _)| **i == Interaction::Hovered)
        .map(|(_, t)| t.0.as_str());
    let cursor = windows.single().ok().and_then(Window::cursor_position);
    match (tip, cursor) {
        (Some(tip), Some(cursor)) => {
            if let Ok(mut t) = text.single_mut() {
                if t.as_str() != tip {
                    *t = Text::new(tip);
                }
            }
            node.left = Val::Px(cursor.x + 14.0);
            node.top = Val::Px(cursor.y + 18.0);
            if *vis != Visibility::Visible {
                *vis = Visibility::Visible;
            }
        }
        _ => {
            if *vis != Visibility::Hidden {
                *vis = Visibility::Hidden;
            }
        }
    }
}

/// Paints [`UiButton`] / toggles, wheel-scrolls [`UiScroll`], shows [`UiTooltip`].
pub struct UiWidgetsPlugin;

impl Plugin for UiWidgetsPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, spawn_tooltip_root)
            .add_systems(
                Update,
                (
                    sync_button_visuals,
                    sync_toggle_visuals,
                    sync_slider_fill,
                    scroll_wheel,
                    sync_tooltip,
                ),
            );
    }
}

/// Test helper: query-access conflicts (B0001) panic at system init — catch them in tests.
#[cfg(test)]
pub(crate) mod test_support {
    use bevy::prelude::*;

    pub(crate) fn init_ok<M>(f: impl IntoSystem<(), (), M>) {
        use bevy::ecs::system::System;
        let mut world = World::new();
        let mut sys = IntoSystem::into_system(f);
        sys.initialize(&mut world);
    }
}

#[cfg(test)]
mod tests {
    use super::test_support::init_ok;
    use super::*;

    #[test]
    fn widget_systems_have_disjoint_queries() {
        init_ok(sync_button_visuals);
        init_ok(sync_toggle_visuals);
        init_ok(sync_slider_fill);
        init_ok(scroll_wheel);
        init_ok(sync_tooltip);
    }

    #[test]
    fn disabled_button_never_fires() {
        let b = UiButton::new(UiButtonVariant::Primary).disabled(true);
        assert!(!ui_button_fired(&Interaction::Pressed, &b));
        assert!(ui_button_fired(&Interaction::Pressed, &UiButton::default()));
    }

    #[test]
    fn button_states_are_distinct() {
        let pal = UiPalette::default();
        let b = UiButton::default();
        let idle = button_colors(&pal, &b, Interaction::None);
        let hover = button_colors(&pal, &b, Interaction::Hovered);
        let press = button_colors(&pal, &b, Interaction::Pressed);
        assert_ne!(idle.0, hover.0);
        assert_ne!(hover.0, press.0);
        let sel = button_colors(&pal, &b.selected(true), Interaction::None);
        assert_ne!(idle.0, sel.0);
        let dis = button_colors(&pal, &b.disabled(true), Interaction::None);
        assert_ne!(idle.2, dis.2);
    }
}
