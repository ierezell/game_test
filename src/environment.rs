use bevy::prelude::*;

pub struct EnvironmentPlugin;

impl Plugin for EnvironmentPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, (setup_atmospheric_elements, create_atmosphere))
            .add_systems(Update, (update_atmosphere, flickering_lights));
    }
}

#[derive(Component)]
pub struct AtmosphericLight {
    pub base_intensity: f32,
    pub flicker_speed: f32,
    pub flicker_intensity: f32,
    pub color_shift: f32,
}

#[derive(Component)]
pub struct EnvironmentObject {
    pub object_type: EnvironmentType,
    pub is_interactive: bool,
}

#[derive(Clone, Copy, PartialEq)]
pub enum EnvironmentType {
    Wall,
    Door,
    Terminal,
    Crate,
    Pipe,
    VentilationShaft,
    PowerBox,
    SecurityCamera,
}

#[derive(Resource)]
pub struct AtmosphereSettings {
    pub fog_density: f32,
    pub ambient_intensity: f32,
    pub shadow_intensity: f32,
    pub emergency_lighting: bool,
}

impl Default for AtmosphereSettings {
    fn default() -> Self {
        Self {
            fog_density: 0.1,
            ambient_intensity: 0.05,
            shadow_intensity: 0.8,
            emergency_lighting: false,
        }
    }
}

fn setup_atmospheric_elements(
    _commands: Commands,
    _meshes: ResMut<Assets<Mesh>>,
    _materials: ResMut<Assets<StandardMaterial>>,
) {
    // Do nothing - let static level handle everything
}

fn create_atmosphere(mut commands: Commands) {
    commands.insert_resource(AtmosphereSettings::default());

    // Minimal atmosphere - just the resource
    info!("Atmospheric environment created");
}

fn update_atmosphere(atmosphere: Res<AtmosphereSettings>, mut ambient_light: ResMut<AmbientLight>) {
    if atmosphere.is_changed() {
        ambient_light.brightness = atmosphere.ambient_intensity;
    }
}

fn flickering_lights(
    time: Res<Time>,
    mut light_query: Query<(&mut PointLight, &AtmosphericLight)>,
) {
    for (mut light, atmospheric) in light_query.iter_mut() {
        let time_factor = time.elapsed_secs() * atmospheric.flicker_speed;
        let flicker = (time_factor.sin() * 0.5 + 0.5) * atmospheric.flicker_intensity;

        light.intensity = atmospheric.base_intensity * (1.0 - flicker);

        // Slight color variation
        let color_variation = (time_factor * 2.0).sin() * atmospheric.color_shift;
        light.color = Color::srgb(
            1.0,
            0.9 + color_variation * 0.1,
            0.7 + color_variation * 0.2,
        );
    }
}
