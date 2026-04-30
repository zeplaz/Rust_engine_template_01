use bevy::prelude::*;

struct Images {
    splash: Handle<Image>,
    background: Handle<Image>,
}

impl FromWorld for Images {
    fn from_world(world: &mut World) -> Self {
        let asset_server = world.get_resource_mut::<AssetServer>().unwrap();
        Self {
            splash: asset_server.load("splash/splash_01.png"),
            background: asset_server.load("icon_inverted.png"),
        }
    }
}

/*
.insert_resource(ClearColor(Color::srgb(0.0, 0.0, 0.0)))
        .insert_resource(Msaa::Sample4)
        .init_resource::<UiState>()
        */
