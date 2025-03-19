use bevy::{
    prelude::*,
    render::{
        render_resource::{
            AddressMode, Extent3d, FilterMode, SamplerDescriptor, TextureDescriptor,
            TextureDimension, TextureFormat, TextureUsages,
        },
        view::RenderLayers,
        camera::RenderTarget,
    },
};
use bevy::image::ImageSampler;
use bevy::render::render_resource::Sampler;
use bevy::render::renderer::RenderDevice;
use crate::actors::player::Player;

// Plugin to set up the distortion post-processing effect
pub struct DistortionPostProcessPlugin;

impl Plugin for DistortionPostProcessPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, setup_distortion_post_process)
            .add_systems(PostUpdate, update_distortion_strength);
    }
}

// Components for the post-processing setup
#[derive(Component)]
struct DistortionCamera;

#[derive(Component)]
struct DistortionMaterial {
    handle: Handle<Image>,
    strength: f32,
}

// Set up the post-processing effect
fn setup_distortion_post_process(
    mut commands: Commands,
    mut images: ResMut<Assets<Image>>,
    window_query: Query<&Window>,
) {
    let window = window_query.single();

    // Create a render target texture
    let size = Extent3d {
        width: window.physical_width(),
        height: window.physical_height(),
        depth_or_array_layers: 1,
    };

    // Create the texture that will be rendered to
    let mut render_target = Image {
        texture_descriptor: TextureDescriptor {
            label: Some("distortion_render_target"),
            size,
            dimension: TextureDimension::D2,
            format: TextureFormat::bevy_default(),
            mip_level_count: 1,
            sample_count: 1,
            usage: TextureUsages::TEXTURE_BINDING | TextureUsages::RENDER_ATTACHMENT,
            view_formats: &[],
        },
        ..default()
    };

    // Allocate a new empty texture
    render_target.resize(size);

    // Add the texture to the asset system
    let render_target_handle = images.add(render_target);

    // Create a camera that renders to the texture for the particle simulation
    commands.spawn((
        Camera2dBundle {
            camera: Camera {
                order: -1, // Render before the main camera
                target: RenderTarget::Image(render_target_handle.clone()),
                ..default()
            },
            ..default()
        },
        DistortionCamera,
        RenderLayers::layer(1), // Assign to layer 1
    ));

    // Add the distortion material to the player
    commands.spawn((
        DistortionMaterial {
            handle: render_target_handle,
            strength: 1.0,
        },
    ));

    // Set up particle simulation entities to render to the first layer
    commands.insert_resource(ParticleRenderLayer(RenderLayers::layer(1)));
}

// Resource to track which layer particles should render to
#[derive(Resource)]
pub struct ParticleRenderLayer(pub RenderLayers);

// System to update the distortion effect based on player state
fn update_distortion_strength(
    mut distortion_query: Query<&mut DistortionMaterial>,
    player_query: Query<Entity, With<Player>>,
    game_state: Res<crate::state::MainGameState>,
) {
    if player_query.get_single().is_ok() {
        for mut distortion in distortion_query.iter_mut() {
            // Adjust distortion strength based on player shield/energy
            let shield_factor = (game_state.player_shield / 100.0).clamp(0.0, 1.0);
            let energy_factor = (game_state.player_energy / 100.0).clamp(0.0, 1.0);

            // Combine factors for overall distortion strength
            distortion.strength = 0.5 + (shield_factor * 0.3) + (energy_factor * 0.2);
        }
    }
}

// Function to add the distortion post-processing plugin
pub fn add_distortion_plugin(app: &mut App) {
    app.add_plugins(DistortionPostProcessPlugin);
}

// Function to be called when setting up particle systems
pub fn configure_particles_for_distortion(
    commands: &mut Commands,
    entity: Entity,
    render_layer: ParticleRenderLayer,
) {
    commands.entity(entity).insert(render_layer.0);
}