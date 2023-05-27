use bevy::prelude::*;

const MAX_LIGHTS: usize = 16;
pub struct LightAffected;

#[derive(Debug)]
pub struct LightData {
    pub position: Vec3,
    pub color: Color,
    pub intensity: f32,
    pub range: f32,
}

#[derive(Debug, Default)]
pub struct ActiveLights {
    pub lights: Vec<LightData>,
}

pub struct LocalLightPlugin;

impl Plugin for LocalLightPlugin {
    fn build(&self, app: &mut AppBuilder) {
          app.init_resource::<ActiveLights>()
            .add_system(light_management_system.system())
            .add_system(update_light_uniforms.system());
    }
}

fn light_management_system(
    query: Query<(&Light, &GlobalTransform)>,
    mut active_lights: ResMut<ActiveLights>,
) {
    active_lights.lights.clear();

    for (light, transform) in query.iter() {
        let position = transform.translation;
        active_lights.lights.push(Light {
            position,
            color: light.color,
            intensity: light.intensity,
            range: light.range,
        });
    }
}

fn update_light_uniforms(
    mut query: Query<(&mut Handle<ColorMaterial>, &GlobalTransform, &Children)>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    active_lights: Res<ActiveLights>,
) {
    for (mut material_handle, transform, children) in query.iter_mut() {
        if let Some(material) = materials.get_mut(&mut *material_handle) {
            material.set_uniform("light_count", active_lights.lights.len() as f32);
            for (i, light) in active_lights.lights.iter().enumerate() {
                material.set_uniform(
                    format!("lights[{}].position", i),
                    light.position.extend(0.0),
                );
                material.set_uniform(format!("lights[{}].color", i), light.color);
                material.set_uniform(format!("lights[{}].radius", i), light.range);
            }
        }
    }
}
