use std::cmp::max;
use bevy::{
    prelude::*,
    render::{
        render_resource::{AsBindGroup, ShaderType},
    },
    sprite::{Material2d, Material2dPlugin},
};
use rs_physics::forces::Force;
use rs_physics::utils::fast_atan2;
use crate::actors::player::{PhysicsSystem2D, Player};
use crate::props::wall_base::{Wall, WallShape};
use crate::state::MainGameState;

// Define the space-time rip shader material
#[derive(Asset, AsBindGroup, Debug, Clone, TypePath)]
pub struct SpaceTimeRipMaterial {
    #[uniform(0)]
    pub properties: SpaceTimeRipProperties,
}

#[derive(Clone, Debug, ShaderType)]
pub struct SpaceTimeRipProperties {
    pub start_point: Vec2,
    pub end_point: Vec2,
    pub width: f32,
    pub glow_intensity: f32,
    pub distortion_strength: f32,
    pub time: f32,
    pub glow_color: Vec4,
    pub animation_speed: f32,
}

// Implement Material2d for the shader
impl Material2d for SpaceTimeRipMaterial {
    fn fragment_shader() -> bevy::render::render_resource::ShaderRef {
        "shaders/space_time_rip.wgsl".into()
    }

    fn alpha_mode(&self) -> bevy::sprite::AlphaMode2d {
        bevy::sprite::AlphaMode2d::Blend
    }
}

// Component to link the rip effect to walls
#[derive(Component)]
pub struct SpaceTimeRipEffect {
    pub material_handle: Handle<SpaceTimeRipMaterial>,
    pub collision_width: f32, // Width of collision area
    pub pull_strength: f32,   // Strength of gravitational pull
    pub energy_drain: f32,    // Energy drain per second
    pub shield_damage: f32,   // Shield damage on direct contact
}

// Plugin for the space-time rip effect
pub struct SpaceTimeRipPlugin;

impl Plugin for SpaceTimeRipPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(Material2dPlugin::<SpaceTimeRipMaterial>::default())
            .add_systems(Startup, setup_space_time_rips)
            .add_systems(PostUpdate, (update_space_time_rip_material, detect_rip_collisions));
    }
}

fn setup_space_time_rips(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<SpaceTimeRipMaterial>>,
    wall_query: Query<(Entity, &Wall, &Transform)>,
    window_query: Query<&Window>,
) {
    println!("Setting up space-time rips...");

    let window = window_query.get_single().unwrap_or_else(|_| panic!("No window found"));
    let window_width = window.width();
    let window_height = window.height();

    for (wall_entity, wall, wall_transform) in wall_query.iter() {
        // Only create rips for walls with the SpaceTimeRip shape
        if matches!(wall.wall_shape, WallShape::SpaceTimeRip) {
            // Get the corners of the wall
            let corners = wall.get_corners();

            let wall_center = (corners[0] + corners[1] + corners[2] + corners[3]) / 4.0;

            // Calculate wall vectors accurately
            let top_edge_start = corners[0]; // top_left corner
            let top_edge_end = corners[1];   // top_right corner

            // Calculate wall center and direction accurately
            let wall_direction = (top_edge_end - top_edge_start).normalize();

            // Calculate precise angle
            let angle = wall_direction.y.atan2(wall_direction.x);

            // Calculate wall length
            let wall_length = (top_edge_end - top_edge_start).length();

            // Create a material specifically tailored for this wall's orientation
            let material_handle = materials.add(SpaceTimeRipMaterial {
                properties: SpaceTimeRipProperties {
                    // Map to centered UV coordinates for consistent tearing effect
                    start_point: Vec2::new(0.0, 0.5),
                    end_point: Vec2::new(1.0, 0.5),
                    width: 8.0,
                    glow_intensity: 0.8,
                    distortion_strength: 1.5,
                    time: 0.0,
                    glow_color: Vec4::new(0.6, 0.0, 1.0, 0.8),
                    animation_speed: 0.7,
                },
            });

            // Add the effect component to the wall entity
            commands.entity(wall_entity).insert(SpaceTimeRipEffect {
                material_handle: material_handle.clone(),
                collision_width: wall_length,
                pull_strength: 100.0,
                energy_drain: 5.0,
                shield_damage: 2.0
            });

            // Calculate mesh dimensions - narrower height with precise length
            let mesh_width = wall_length;
            let mesh_height = wall.width * 0.4; // Narrow enough to not be too rectangular

            // Z position to prevent Z-fighting with wall
            let z_position = rand::random_range(-2.0..-1.0);

            // Spawn the effect with precise positioning and rotation
            commands.spawn((
                Mesh2d(meshes.add(Rectangle::new(mesh_width, mesh_height))),
                MeshMaterial2d(material_handle),
                Transform {
                    // Position exactly at wall center
                    translation: Vec3::new(wall_center.x, wall_center.y, z_position),
                    // Apply precise rotation
                    rotation: Quat::from_rotation_z(angle),
                    scale: Vec3::ONE,
                },
                Visibility::default(),
                InheritedVisibility::default(),
                ViewVisibility::default(),
            ));

            println!("Space-time rip added to wall at ({}, {}) with angle {}",
                     wall.center_x, wall.center_y, angle);
        }
    }
}

// Collision detection system for space-time rips
fn detect_rip_collisions(
    mut player_query: Query<(&Transform, &mut PhysicsSystem2D), With<Player>>,
    rip_query: Query<(&Transform, &SpaceTimeRipEffect)>,
    mut game_state: ResMut<MainGameState>,
    time: Res<Time>,
) {
    // Only process if we have a player
    if let Ok((player_transform, mut player_physics)) = player_query.get_single_mut() {
        let player_pos = Vec2::new(player_transform.translation.x, player_transform.translation.y);
        let dt = time.delta_secs();

        // Check each space-time rip for collision
        for (rip_transform, rip_effect) in rip_query.iter() {
            let rip_pos = Vec2::new(rip_transform.translation.x, rip_transform.translation.y);
            let rip_rotation = rip_transform.rotation;

            // Get the distance from player to rip center
            let distance = player_pos.distance(rip_pos);

            // Define collision distances
            let close_distance = rip_effect.collision_width; // Outer influence zone
            let direct_contact = rip_effect.collision_width * 0.4; // Inner damage zone

            // Apply effect if player is within influence range
            if distance < close_distance {
                // Calculate influence factor (stronger closer to center)
                let influence = 1.0 - (distance / close_distance).clamp(0.0, 1.0);

                // 1. Apply gravitational pull toward the rip center
                let pull_direction = (rip_pos - player_pos).normalize();
                let pull_force = rip_effect.pull_strength * influence;

                // Get physics object and apply force
                let physics_obj = player_physics.0.get_object_mut(0).unwrap();
                physics_obj.add_force(Force::Thrust {
                    magnitude: pull_force as f64,
                    angle: fast_atan2(pull_direction.y, pull_direction.x) as f64,
                });

                // 2. Drain energy proportional to proximity and time
                let energy_drain = rip_effect.energy_drain * influence * dt;
                game_state.player_energy = (game_state.player_energy - energy_drain).max(0.0);

                // 3. Apply shield damage for direct contact with center
                if distance < direct_contact {
                    // Only damage shield if player has shield
                    if game_state.player_shield > 0.0 {
                        let shield_damage = rip_effect.shield_damage;
                        game_state.player_shield -= shield_damage;
                    } else {
                        // If no shield, damage HP directly (at reduced rate)
                        let hp_damage = rip_effect.shield_damage;
                        game_state.player_hp -= hp_damage;
                    }

                    // 4. Apply velocity distortion effect (randomize direction slightly)
                    if physics_obj.speed() > 5.0 {
                        // Get current velocity angle
                        let vel_angle = fast_atan2(
                            physics_obj.velocity.y as f32,
                            physics_obj.velocity.x as f32
                        );

                        // Add small random perturbation to angle
                        let perturbation = (time.elapsed_secs() * 10.0).sin() * 0.2;
                        let new_angle = vel_angle + perturbation;

                        // Get current speed but keep it constant
                        let speed = physics_obj.speed();

                        // Apply modified velocity
                        physics_obj.velocity.x = (new_angle.cos() as f64) * speed;
                        physics_obj.velocity.y = (new_angle.sin() as f64) * speed;
                    }
                }

            }
        }
    }
}

// Update the space-time rip material
fn update_space_time_rip_material(
    mut materials: ResMut<Assets<SpaceTimeRipMaterial>>,
    time: Res<Time>,
    query: Query<&SpaceTimeRipEffect>,
    game_state: Res<crate::state::MainGameState>,
    player_query: Query<&Transform, With<crate::actors::player::Player>>,
) {
    // Get player position for dynamic effects
    let player_transform = player_query.get_single().ok();

    for effect in query.iter() {
        if let Some(material) = materials.get_mut(&effect.material_handle) {
            // Update time for animation
            material.properties.time = time.elapsed_secs();


            // Make the rip width pulse slightly
            let pulse = (time.elapsed_secs().sin() * 0.2 + 1.0);
            material.properties.width = 6.0 * pulse;

            // Adjust the rip effect intensity based on player proximity
            if let Some(player_pos) = player_transform {
                // Get player position
                let player_pos_2d = Vec2::new(player_pos.translation.x, player_pos.translation.y);

                // Create a normalized intensity factor based on proximity
                // This would need to use the actual rip position, but we'll approximate
                let proximity_multiplier = 1.0 ; // Default value - modify if needed

                // Apply the proximity effect to intensity and distortion
                material.properties.glow_intensity *= proximity_multiplier;
                material.properties.distortion_strength *= proximity_multiplier;
            }
        }
    }
}