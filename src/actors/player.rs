use bevy::log::tracing_subscriber::fmt::time;
use bevy::math::VectorSpace;
use bevy::prelude::*;
use bevy::render::view::prepare_windows;
use bevy::sprite::Anchor;
use bevy::tasks::futures_lite::StreamExt;
use bevy::window::WindowRef;
use rs_physics::forces::Force;
use rs_physics::interactions::elastic_collision_2d;
use rs_physics::models::{ObjectIn2D, Velocity2D};
use rs_physics::utils::{DEFAULT_PHYSICS_CONSTANTS, PhysicsConstants};
use crate::hud::{EnergyBar, HpBar, ShieldBar};
use crate::state::MainGameState;

pub(crate) const GROUND_LEVEL: f64 = -300.0;

const PHYSICS_CONSTANTS: PhysicsConstants = PhysicsConstants {
    gravity: DEFAULT_PHYSICS_CONSTANTS.gravity / 2.0,
    ground_level: GROUND_LEVEL,
    ..DEFAULT_PHYSICS_CONSTANTS
};

#[derive(Component)]
pub struct PhysicsSystem2D(rs_physics::forces::PhysicsSystem2D);

impl PhysicsSystem2D {
    fn new(constants: PhysicsConstants, player_object: ObjectIn2D) -> Self {
        let mut physics_system = rs_physics::forces::PhysicsSystem2D::new(constants);
        physics_system.add_object(player_object);
        physics_system.apply_gravity();
        physics_system.apply_drag(0.47, 0.5);
        Self (physics_system)
    }
}

#[derive(Component)]
pub struct Player;

pub fn setup_camera(
    mut commands: Commands,
) {
    commands.spawn(
        Camera2d {
            ..default()
        }
    );
}

pub fn setup_player(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    // Player - updated to use the new ObjectIn2D::new with velocity components
    let player_object = ObjectIn2D::new(65.0, 0.0, 0.0, (-200.0, GROUND_LEVEL + 100.0));
    let player_color = Color::srgb(0.5, 1.0, 0.5);

    commands
        .spawn((
            Player,
            Mesh2d(
                meshes.add(Circle::new(30.0))
            ),
            MeshMaterial2d(materials.add(player_color)),
            Transform {
                translation: Vec3::new(-200.0, GROUND_LEVEL as f32 + 60.0, -1.0),
                ..Default::default()
            },
            Anchor::BottomCenter,
            PhysicsSystem2D::new(PHYSICS_CONSTANTS, player_object),
        ));

    // Ground
    commands.spawn((
        Mesh2d(meshes.add(Rectangle::new(800.0, 10.0))),
        MeshMaterial2d(materials.add(Color::srgb(0.5, 0.5, 0.5))),
        Transform {
            translation: Vec3::new(0.0, GROUND_LEVEL as f32, 0.0),
            ..Default::default()
        },
    ));

    // Ground
    commands.spawn((
        Mesh2d(meshes.add(Rectangle::new(800.0, 10.0))),
        MeshMaterial2d(materials.add(Color::srgb(0.5, 0.5, 0.5))),
        Transform {
            translation: Vec3::new(768.0, GROUND_LEVEL as f32 + 152.5, 0.0),
            rotation: Quat::from_rotation_z(std::f32::consts::FRAC_PI_8),
            ..Default::default()
        },
    ));
}

fn calculate_ground_level(x_pos: f64) -> f64 {
    let join_x = 396.54;
    let base = GROUND_LEVEL + 30.0;
    if x_pos > join_x {
        let offset = x_pos - join_x;
        let slope = std::f32::consts::FRAC_PI_8.tan() as f64; // ≈0.4142
        base + offset * slope
    } else {
        base
    }
}

pub fn camera_movement(
    query: Query<(&Player, &Transform)>,
    mut shield_bar_query: Query<&mut Transform, (With<ShieldBar>, Without<Camera2d>, Without<Player>, Without<HpBar>, Without<EnergyBar>)>,
    mut hp_bar_query: Query<&mut Transform, (With<HpBar>, Without<Camera2d>, Without<Player>, Without<EnergyBar>, Without<ShieldBar>)>,
    mut energy_bar_query: Query<&mut Transform, (With<EnergyBar>, Without<Camera2d>, Without<Player>, Without<HpBar>, Without<ShieldBar>)>,
    mut camera_query: Query<&mut Transform, (With<Camera2d>, Without<Player>, Without<HpBar>, Without<EnergyBar>, Without<ShieldBar>)>,
) {
    let (_, player_transform) = query.iter()
        .next()
        .expect("There should only be one player entity");

    let mut camera_transform = camera_query.iter_mut()
        .next()
        .expect("There should only be one camera entity");

    let mut shield_bar_transform = shield_bar_query.iter_mut()
        .next()
        .expect("There should only be one shield bar entity");

    let mut hp_bar_transform = hp_bar_query.iter_mut()
        .next()
        .expect("There should only be one hp bar entity");

    let mut energy_bar_transform = energy_bar_query.iter_mut()
        .next()
        .expect("There should only be one energy bar entity");

    camera_transform.translation.x = player_transform.translation.x * 0.5;
    camera_transform.translation.y = player_transform.translation.y * 0.25;

    shield_bar_transform.translation.x = camera_transform.translation.x - 400.0;
    shield_bar_transform.translation.y = camera_transform.translation.y + 320.0;

    hp_bar_transform.translation.x = camera_transform.translation.x - 400.0;
    hp_bar_transform.translation.y = camera_transform.translation.y + 300.0;

    energy_bar_transform.translation.x = camera_transform.translation.x - 400.0;
    energy_bar_transform.translation.y = camera_transform.translation.y + 280.0;
}

pub fn player_movement_physics (
    mut player_transform_query: Query<&mut Transform, With<Player>>,
    mut player_query: Query<&mut PhysicsSystem2D>,
    time: Res<Time<Fixed>>,
) {
    let mut player_transform = player_transform_query.get_single_mut()
        .expect("There should only be one player entity");
    player_query
        .par_iter_mut()
        .for_each(|mut physics_system| {
            physics_system.0.update(0.62);

            let ground_level = calculate_ground_level(player_transform.translation.x as f64);
            physics_system.0.update_ground_level(ground_level);

            let player_obj = physics_system.0.get_object_mut(0).unwrap();

            // Apply velocity damping - updated to work with velocity components
            player_obj.velocity.x *= 0.98;
            player_obj.velocity.y *= 0.98;

            // Check if velocity is very small and zero it out if so
            if player_obj.speed() < 1.0 {
                player_obj.velocity.x = 0.0;
                player_obj.velocity.y = 0.0;
            }

            // Enforce position bounds
            if player_obj.position.x >= 1100.0 || player_obj.position.x <= -400.0 {
                player_obj.position.x = player_obj.position.x.clamp(-400.0, 1100.0);
                player_obj.velocity.x = 0.0;
            }

            // Ground collision detection
            if player_obj.position.y <= ground_level + 10.0 && player_obj.velocity.y < 0.0 {
                // Create a ground object with very high mass (effectively immovable)
                let mut ground_object = ObjectIn2D::new(1e11, 0.0, 0.0, (player_obj.position.x, ground_level));

                // Determine collision angle based on terrain
                let collision_angle = if player_transform.translation.x > 396.54 {
                    std::f64::consts::FRAC_PI_2 + std::f64::consts::FRAC_PI_8
                } else {
                    std::f64::consts::FRAC_PI_2
                };

                // Simulate elastic collision with the ground
                elastic_collision_2d(
                    &PHYSICS_CONSTANTS,
                    player_obj,
                    &mut ground_object,
                    collision_angle,
                    time.delta_secs_f64(),
                    0.45,
                    0.15
                ).expect("Elastic collision failed");
            }
        });
}

pub fn update_player_movement(
    mut player_transform_query: Query<&mut Transform, With<Player>>,
    mut player_query: Query<&mut PhysicsSystem2D>,
) {
    let mut player_transform = player_transform_query.get_single_mut()
        .expect("There should only be one player entity");
    if let Ok(mut physics_system) = player_query.get_single_mut() {
        let player_obj = physics_system.0.get_object_mut(0).unwrap();

        player_transform.translation.x = player_obj.position.x as f32;
        player_transform.translation.y = player_obj.position.y as f32;
    }
}

fn ground_tangent(x_pos: f32) -> (f32, f32) {
    // Assume that for x > 398, the ground is sloped with an angle of FRAC_PI_8.
    if x_pos > 396.0 {
        let theta = std::f32::consts::FRAC_PI_8; // slope angle in radians.
        // For a slope inclined upward to the right, the ground normal might be:
        // N = (-sin(theta), cos(theta)) and then the tangent is:
        (theta.cos(), theta.sin())
    } else {
        // Flat ground
        (1.0, 0.0)
    }
}

pub fn player_input(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut player_query: Query<&mut PhysicsSystem2D, With<Player>>,
    mut game_state: ResMut<MainGameState>,
    time: Res<Time>,
) {
    let mut physics_system = player_query.iter_mut()
        .next()
        .expect("There should only be one player entity");

    if keyboard_input.just_pressed(KeyCode::Space) {
        if game_state.player_energy <= 0.0 {
            return;
        }

        let player_phys_obj = physics_system.0.get_object_mut(0).unwrap();
        if player_phys_obj.position.y <= calculate_ground_level(player_phys_obj.position.x) + 10.0 {
            // Get the direction of movement from velocity components
            let direction = player_phys_obj.direction();

            let max_offset = std::f64::consts::FRAC_PI_6; // ~30 degrees
            // Offset angle based on x direction
            let angle_offset = direction.x * max_offset;
            // Subtract offset from π/2 so that if x > 0, jump tilts right (angle becomes < π/2)
            // and if x < 0, jump tilts left (angle becomes > π/2).
            let thrust_angle = std::f64::consts::FRAC_PI_2 - angle_offset;
            let base_magnitude = 9800.0;

            // Adjust magnitude based on horizontal movement
            let magnitude = if direction.x.abs() != 0.0 {
                base_magnitude * 0.75
            } else {
                base_magnitude
            };

            player_phys_obj.add_force(Force::Thrust {
                magnitude,
                angle: thrust_angle,
            });
            game_state.player_energy = (game_state.player_energy - 25.0).max(0.0);
        }
    }

    if keyboard_input.pressed(KeyCode::KeyA) {
        let player_phys_obj = physics_system.0.get_object_mut(0).unwrap();
        if player_phys_obj.position.y > calculate_ground_level(player_phys_obj.position.x) + 5.0 {
            return;
        }

        // Calculate ground tangent for appropriate movement angle
        let tangent = ground_tangent(player_phys_obj.position.x as f32);
        // Compute the angle from the left tangent vector
        let angle = VectorSpace::lerp(tangent.1, tangent.0, time.delta_secs()).to_radians();

        // Adjust force magnitude based on position
        let magnitude = if player_phys_obj.position.y >= calculate_ground_level(player_phys_obj.position.x) + 5.0 {
            -100.0
        } else {
            -280.0
        };

        // Apply thrust along this angle
        player_phys_obj.add_force(Force::Thrust { magnitude, angle: angle as f64 });
    }

    if keyboard_input.pressed(KeyCode::KeyD) {
        let player_phys_obj = physics_system.0.get_object_mut(0).unwrap();
        if player_phys_obj.position.y > calculate_ground_level(player_phys_obj.position.x) + 5.0 {
            return;
        }

        let tangent = ground_tangent(player_phys_obj.position.x as f32);
        // Compute the angle from the tangent vector
        let angle = VectorSpace::lerp(tangent.1, tangent.0, time.delta_secs()).to_radians();

        let magnitude = if player_phys_obj.position.y >= calculate_ground_level(player_phys_obj.position.x) + 5.0 {
            100.0
        } else {
            280.0
        };

        player_phys_obj.add_force(Force::Thrust { magnitude, angle: angle as f64 });
    }
}