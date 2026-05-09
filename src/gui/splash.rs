// Canonical Bevy native UI: splash screen.
// Uses Node + required components pattern (Bevy 0.15+).
// No egui here — this is in-game rendering, not tooling.

use bevy::prelude::*;

use bevy::ui::widget::ImageNode;

// Tag component marking entities spawned for the splash screen.
#[derive(Component)]
pub struct OnSplashScreen;

#[derive(Resource, Deref, DerefMut)]
struct SplashTimer(Timer);

pub struct SplashPlugin;

impl Plugin for SplashPlugin {
    fn build(&self, app: &mut App) {
        app.init_state::<AppStartState>()
            .add_systems(OnEnter(AppStartState::Splash), splash_setup)
            .add_systems(Update, countdown.run_if(in_state(AppStartState::Splash)))
            .add_systems(OnExit(AppStartState::Splash), despawn_splash);
    }
}

/// Game-start state enum used only by the splash flow.
#[derive(Debug, Clone, Copy, Default, Eq, PartialEq, Hash, States)]
pub enum AppStartState {
    #[default]
    Splash,
    Menu,
}

fn splash_setup(mut commands: Commands, asset_server: Res<AssetServer>) {
    let icon: Handle<Image> = asset_server.load("splash/splash_01.png");

    // Full-bleed color so the splash is visible even before `splash_01.png` finishes loading (or if the file is absent).
    commands.spawn((
        Node {
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            align_items: AlignItems::Center,
            justify_content: JustifyContent::Center,
            ..default()
        },
        BackgroundColor(Color::srgb(0.07, 0.08, 0.11)),
        OnSplashScreen,
    ))
    .with_children(|parent| {
        // Logo image — UiImage + Node (required components; no legacy image bundle type).
        parent.spawn((
            Node {
                width: Val::Px(200.0),
                height: Val::Auto,
                ..default()
            },
            ImageNode::from(icon),
        ));
    });

    commands.insert_resource(SplashTimer(Timer::from_seconds(1.0, TimerMode::Once)));
}

fn countdown(
    launch: Option<Res<crate::engine::EngineLaunchArgs>>,
    mut next_state: ResMut<NextState<AppStartState>>,
    time: Res<Time>,
    mut timer: ResMut<SplashTimer>,
) {
    if launch.is_some_and(|l| l.test_mode()) {
        NextState::set_if_neq(&mut *next_state, AppStartState::Menu);
        return;
    }
    if timer.tick(time.delta()).is_finished() {
        NextState::set_if_neq(&mut *next_state, AppStartState::Menu);
    }
}

fn despawn_splash(
    mut commands: Commands,
    splash_query: Query<Entity, With<OnSplashScreen>>,
) {
    for entity in &splash_query {
        commands.entity(entity).despawn();
    }
}
