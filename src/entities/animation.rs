// animation states:

#[derive(Debug, Clone, Copy, Default, Eq, PartialEq, Hash, States)]
pub enum LightingState {
    Night,
    #[default]
    Midday,
}

#[derive(Debug, Deserialize)]
pub struct TextureInfo {
    pub path: String,
    pub tiles: u32,
    pub emission: f32,
}

#[derive(Component)]
pub struct Textures {
    pub textures: HashMap<String, HashMap<String, HashMap<String, TextureInfo>>>,
}
