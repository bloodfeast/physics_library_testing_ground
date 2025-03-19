use bevy::{
    prelude::*,
    render::{
        render_resource::{AsBindGroup, BufferInitDescriptor, BufferUsages, ShaderType, Buffer},
        extract_resource::{ExtractResource, ExtractResourcePlugin},
        renderer::RenderDevice,
    },
    sprite::{Material2d, Material2dPlugin},
};
use bevy::render::RenderApp;
use bevy::render::storage::ShaderStorageBuffer;
use bevy::sprite::AlphaMode2d;
use crate::actors::player::{PhysicsSystem2D, Player};
use crate::state::MainGameState;
use crate::actors::particles::CosmologicalSimulation;


// Define the black hole shader material
#[derive(Asset, AsBindGroup, Debug, Clone, TypePath)]
pub struct BlackHoleMaterial {
    #[uniform(0)]
    pub properties: BlackHoleProperties,
}

#[derive(Clone, Debug, ShaderType)]
pub struct BlackHoleProperties {
    pub center: Vec2,
    pub radius: f32,
    pub accretion_radius: f32,
    pub distortion_strength: f32,
    pub rotation_speed: f32,
    pub time: f32,
    pub glow_color: Vec4,
}

// Implement Material2d for the shader
impl Material2d for BlackHoleMaterial {
    fn fragment_shader() -> bevy::render::render_resource::ShaderRef {
        "shaders/black_hole.wgsl".into()
    }

    fn alpha_mode(&self) -> AlphaMode2d {
        AlphaMode2d::Blend
    }
}

// Component to link the black hole effect to the player
#[derive(Component)]
pub struct BlackHoleEffect {
    pub material_handle: Handle<BlackHoleMaterial>,
}

// Marker component for entities with black hole material
#[derive(Component)]
pub struct BlackHoleMaterialMarker;

// Plugin for the black hole effect
pub struct BlackHolePlugin;

impl Plugin for BlackHolePlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(Material2dPlugin::<BlackHoleMaterial>::default())
            .add_systems(PostStartup, setup_black_hole)
            .add_systems(PostUpdate, (
                update_black_hole_material,
                update_black_hole_position
            ));

    }
}

// Setup the black hole effect
fn setup_black_hole(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<BlackHoleMaterial>>,
    player_query: Query<Entity, With<Player>>,
) {
    println!("Setting up black hole...");

    let player_result = player_query.get_single();
    println!("Player entity found: {}", player_result.is_ok());

    if let Ok(player_entity) = player_result {
        // Create a mesh for the black hole effect
        let size = 150.0; // Larger size for better lensing effect
        let mesh = Mesh2d(meshes.add(Rectangle::new(size, size)));


        // Create the black hole material with default settings
        let material_handle = materials.add(BlackHoleMaterial {
            properties: BlackHoleProperties {
                center: Vec2::new(0.5, 0.5),                // Center in UV space
                radius: 0.1,                                // Core black hole radius
                accretion_radius: 0.2,                      // Outer accretion disk radius
                distortion_strength: 5.0,                   // Gravitational distortion strength
                rotation_speed: 0.5,                        // Speed of rotation
                time: 0.0,                                  // Initial time
                glow_color: Vec4::new(0.2, 0.7, 1.0, 1.0),
            },
        });

        // Add the black hole effect component to the player
        commands.entity(player_entity).insert(BlackHoleEffect {
            material_handle: material_handle.clone(),
        });

        // Spawn the black hole effect entity
        commands.spawn((
            mesh,
            MeshMaterial2d(material_handle.clone()),
            Transform::from_translation(Vec3::new(0.0, 0.0, 2.0)),
            BlackHoleMaterialMarker,
            BlackHoleEffect {
                material_handle,
            },
            Visibility::default(),
            InheritedVisibility::default(),
            ViewVisibility::default(),
        ));

        println!("Black hole spawned!");
    } else {
        println!("Player entity not found!");
    }
}

// Update the black hole material
fn update_black_hole_material(
    mut materials: ResMut<Assets<BlackHoleMaterial>>,
    time: Res<Time>,
    player_query_bh: Query<&BlackHoleEffect>,
    player_query: Query<&PhysicsSystem2D>,
    game_state: Res<MainGameState>,
) {
    let player = player_query
        .get_single()
        .expect("Player not found");

    let player_phys = &player.0
        .get_object(0)
        .expect("Player physics not found");

    let player_speed = player_phys.speed() as f32;

    for effect in player_query_bh.iter() {
        if let Some(material) = materials.get_mut(&effect.material_handle) {
            // Update time for animation
            material.properties.time = time.elapsed_secs();

            // Adjust black hole properties based on player state
            let shield_factor = (game_state.player_shield / 100.0).clamp(0.1, 1.0);

            // Adjust radius based on shield - smaller radius (more black) with higher shield
            material.properties.radius = 0.15 - (shield_factor * 0.06);

            // Stronger distortion with higher shield
            material.properties.distortion_strength = 3.0 + (shield_factor * 5.0);

            // Change color based on shield/health
            if game_state.player_shield > 50.0 {
                // Blue-ish for high shield
                material.properties.glow_color = Vec4::new(0.1, 0.33, 1.0, 1.0);
            } else if game_state.player_shield > 0.0 {
                // Purple-ish for medium shield
                material.properties.glow_color = Vec4::new(0.8, 0.0, 1.0, 1.0);
            } else if game_state.player_hp > 50.0 {
                // Yellow-ish for good health but no shield
                material.properties.glow_color = Vec4::new(1.0, 0.8, 0.2, 1.0);
            } else {
                // Red-ish for low health
                material.properties.glow_color = Vec4::new(1.0, 0.3, 0.2, 1.0);
            }

            // Adjust rotation speed based on player energy
            material.properties.rotation_speed = (player_speed * 0.2) + std::f32::consts::PI;
        }
    }
}

// Update the position of the black hole effect to follow the player
fn update_black_hole_position(
    player_query: Query<&Transform, With<Player>>,
    mut black_hole_query: Query<&mut Transform, (Without<Player>, With<BlackHoleMaterialMarker>)>,
) {
    if let Ok(player_transform) = player_query.get_single() {
        for mut transform in black_hole_query.iter_mut() {
            // Position the black hole effect at the player's position
            transform.translation.x = player_transform.translation.x;
            transform.translation.y = player_transform.translation.y;

            // Keep the z-coordinate slightly above other elements
            transform.translation.z = 0.5;
        }
    }
}