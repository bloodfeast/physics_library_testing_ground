use bevy::asset::Assets;
use bevy::color::Color;
use bevy::math::Vec3;
use bevy::prelude::*;
use bevy::utils::tracing::Id;
use rand::RngCore;
use rs_physics::forces::Force;
use rs_physics::interactions::gravitational_force;
use rs_physics::models::{Direction2D, FromCoordinates, ObjectIn2D};
use rs_physics::utils::{DEFAULT_PHYSICS_CONSTANTS, fast_atan2, fast_sqrt_f64, PhysicsConstants};
use crate::actors::player::Player;
use crate::state::MainGameState;

#[derive(Component)]
pub struct Enemy(rs_physics::forces::PhysicsSystem2D);

pub fn spawn_enemy(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    mut game_state: ResMut<MainGameState>,
    time: Res<Time>,
    query: Query<&Transform, (With<Player>, Without<Enemy>)>,
) {
    if time.elapsed_secs_f64() < 5.0 {
        return;
    }
    let spawn_rate = match game_state.enemies.len() {
        0..=10 => 0.01,
        11..=20 => 0.03,
        21..=30 => 0.05,
        _ => 0.075,
    };
    if !rand::random_bool(spawn_rate) {
        return;
    }
    let physics_constants = PhysicsConstants {
        gravity: 0.0,
        ground_level: -1600.0,
        air_density: 0.0,
        ..DEFAULT_PHYSICS_CONSTANTS
    };
    let mut enemy_physics = rs_physics::forces::PhysicsSystem2D::new(physics_constants);

    let player_transform = query.iter()
        .next()
        .expect("There should only be one player entity");
    let spawn_x_position = rand::random_range((player_transform.translation.x - 2000.0).min(-1000.0)..=(player_transform.translation.x + 2000.0).max(1200.0));
    let spawn_y_position = rand::random_range(player_transform.translation.y + 1000.0..=player_transform.translation.y + 1400.0);
    let initial_velocity = rand::random_range(100.0..=200.0);

    //calculate the angle between the player and the enemy
    let angle = fast_atan2(player_transform.translation.y - spawn_y_position as f32, player_transform.translation.x - spawn_x_position as f32);
    // calculate the x and y components of the velocity
    let x_velocity = initial_velocity * angle.cos();
    let y_velocity = initial_velocity * angle.sin();

    let enemy_object = ObjectIn2D::new(1.0, x_velocity as f64, y_velocity as f64, (spawn_x_position as f64, spawn_y_position as f64));
    let enemy_color = Color::srgb(1.0, 0.25, 0.25);

    let enemy_mesh = Circle::new(3.14);
    enemy_physics.add_object(enemy_object);

    let enemy_entity = commands.spawn_empty().id();
    game_state.enemies.push(enemy_entity);
    commands.entity(enemy_entity)
        .insert(Enemy(enemy_physics))
        .insert(Mesh2d(
            meshes.add(enemy_mesh)
        ))
        .insert(MeshMaterial2d(materials.add(enemy_color)))
        .insert(Transform {
            translation: Vec3::new(spawn_x_position, spawn_y_position as f32, -1.0),
            ..Default::default()
        });
}


pub fn update_enemy(
    mut commands: Commands,
    mut query: Query<(Entity, &mut Transform, &mut Enemy)>,
    player_query: Query<&Transform, (With<Player>, Without<Enemy>)>,
    mut game_state: ResMut<MainGameState>,
    time: Res<Time>,
) {
    // Get player position for gravitational calculations
    let player_transform = player_query
        .iter()
        .next()
        .expect("There should only be one player entity");

    let player_x = player_transform.translation.x as f64;
    let player_y = player_transform.translation.y as f64;
    let player_mass = 1000000.0 * (game_state.score as f64 * 0.5).max(1.0); // Adjust this to control gravitational strength

    query.iter_mut()
        .for_each(|(entity, mut transform, mut enemy)| {
            let mut enemy: &mut Enemy = &mut enemy;

            // Apply gravitational force toward player
            let enemy_object = enemy.0
                .get_object_mut(0)
                .expect("Failed to get enemy object");

            let dx = player_x - enemy_object.position.x;
            let dy = player_y - enemy_object.position.y;
            let distance_squared = dx * dx + dy * dy;

            // Only apply gravity if enemy is within a certain range
            if distance_squared < 50000.0 { // ~707 units radius
                // Calculate distance (with minimal value to prevent extreme forces)
                let distance = fast_sqrt_f64(distance_squared).max(200.0);

                let dx = dx / distance;
                let dy = dy / distance;

                // Calculate the maximum magnitude of the force
                let max_force_magnitude = 200.0; // Adjust this to control strength

                // Calculate gravitational strength (inverse square law)
                let gravitational_constant = distance * (1./std::f64::consts::PI); // Adjust this to control strength
                let force_magnitude = (gravitational_constant * player_mass * enemy_object.mass / distance_squared).min(max_force_magnitude);

                // Calculate angle of force for the gravitational pull
                let radial_angle = fast_atan2(dy as f32, dx as f32);

                // NEW: Add velocity dampening to help capture objects
                // Get current velocity components
                let vel_x = enemy_object.velocity.x;
                let vel_y = enemy_object.velocity.y;

                // Calculate velocity magnitude
                let velocity_squared = vel_x * vel_x + vel_y * vel_y;
                let velocity_magnitude = fast_sqrt_f64(velocity_squared);

                // Apply dampening based on distance - stronger near ideal orbit
                let ideal_orbit_distance = 400.0; // The distance where orbital force is strongest
                let orbit_width = 200.0_f32; // How wide the "sweet spot" for orbiting is

                // Calculate distance factor that peaks at ideal distance
                let distance_factor = (-(distance as f32 - ideal_orbit_distance).powi(2) /
                    (2.0 * orbit_width.powi(2))).exp();

                // Dampening factor - adjust as needed
                let dampening = 0.02 * distance_factor as f64;

                // Calculate dampening force opposing current velocity
                let dampening_magnitude = velocity_magnitude * dampening;

                // Only apply dampening if the object has significant velocity
                if velocity_magnitude > 10.0 {
                    let dampening_angle = fast_atan2(vel_y as f32, vel_x as f32) + std::f32::consts::PI; // Opposite to velocity

                    let dampening_force = Force::Thrust {
                        magnitude: dampening_magnitude,
                        angle: dampening_angle as f64,
                    };

                    enemy_object.add_force(dampening_force);
                }

                // For clockwise orbit, subtract FRAC_PI_2 (90 degrees)
                let orbital_angle = radial_angle - std::f32::consts::FRAC_PI_2;

                // Calculate orbital coefficient - stronger at ideal orbit distance
                let orbit_coefficient = (-(distance as f32 - ideal_orbit_distance).powi(2) /
                    (2.0 * orbit_width.powi(2))).exp();

                // Adjust orbital strength based on approach angle
                // Calculate current direction of movement relative to radial direction
                let movement_angle = if velocity_magnitude > 0.1 {
                    fast_atan2(vel_y as f32, vel_x as f32)
                } else {
                    0.0
                };

                // Calculate the angle between movement and radial direction
                let angle_diff = ((movement_angle - radial_angle + std::f32::consts::PI) %
                    (2.0 * std::f32::consts::PI)) - std::f32::consts::PI;

                // Calculate an approach factor (1.0 when perpendicular, lower when head-on or away)
                let approach_factor = angle_diff.abs() / (std::f32::consts::FRAC_PI_2);

                // Lower orbital force for direct approaches to prevent flinging
                let orbital_strength_factor = 0.8 * approach_factor as f64;

                // Calculate orbital force magnitude
                let orbital_force_magnitude = force_magnitude * orbital_strength_factor * orbit_coefficient as f64;

                // Create gravitational force (inward pull)
                let gravitational_force = Force::Thrust {
                    magnitude: force_magnitude,
                    angle: radial_angle as f64,
                };

                // Create orbital force (perpendicular to gravitational pull)
                let orbital_force = Force::Thrust {
                    magnitude: orbital_force_magnitude,
                    angle: orbital_angle as f64,
                };

                // Apply gravitational and orbital forces
                enemy_object.add_force(gravitational_force);
                enemy_object.add_force(orbital_force);
            }

            // Update physics
            enemy.0.update(time.delta_secs_f64());

            let enemy_object = enemy.0.get_object(0).unwrap();

            // Check collision with player
            if (enemy_object.position.y - player_y).abs() < 30.0
                && (enemy_object.position.x - player_x).abs() < 30.0 {

                if game_state.player_shield > 0.0 {
                    game_state.player_shield -= 25.0;
                } else {
                    game_state.player_hp -= 10.0;
                }
                game_state.score += 1;

                // Remove the enemy upon collision
                game_state.enemies.retain(|&id| id != entity);
                commands.entity(entity).despawn();
                return;
            }

            // Remove enemies that fall too low
            if enemy_object.position.y as f32 <= -1000.0 {
                game_state.enemies.retain(|&id| id != entity);
                commands.entity(entity).despawn();
                return;
            }

            // Update transform position
            transform.translation = Vec3::new(
                enemy_object.position.x as f32,
                enemy_object.position.y as f32,
                -1.0
            );
        });
}