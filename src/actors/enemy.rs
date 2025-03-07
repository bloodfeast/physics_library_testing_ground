use bevy::asset::Assets;
use bevy::color::Color;
use bevy::math::Vec3;
use bevy::prelude::*;
use bevy::utils::tracing::Id;
use rand::RngCore;
use rs_physics::forces::Force;
use rs_physics::models::{Direction2D, FromCoordinates, ObjectIn2D};
use rs_physics::utils::{DEFAULT_PHYSICS_CONSTANTS, PhysicsConstants};
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
        31..=40 => 0.1,
        41..=50 => 0.125,
        50..=100 => 0.175,
        _ => 0.35,
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
    let spawn_x_position = rand::random_range((player_transform.translation.x - 800.0).max(-800.0)..=(player_transform.translation.x + 800.0).min(1200.0));
    let spawn_y_position = rand::random_range(player_transform.translation.y + 600.0..=player_transform.translation.y + 800.0);
    let initial_velocity = rand::random_range(200.0..=400.0);

    // Enemy - Updated to use the new directional velocities API
    // Since the enemy moves straight down, we set vx=0 and vy=-initial_velocity
    let enemy_object = ObjectIn2D::new(1.0, 0.0, -initial_velocity, (spawn_x_position as f64, spawn_y_position as f64));
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
    query.iter_mut()
        .for_each(|(entity, mut transform, mut enemy)| {
            let mut enemy: &mut Enemy = &mut enemy;
            let player_transform = player_query.iter().next().expect("There should only be one player entity");
            let player_x = player_transform.translation.x;
            let player_y = player_transform.translation.y;

            enemy.0.update(time.delta_secs_f64());

            let mut enemy_object = enemy.0.get_object_mut(0).unwrap();

            if enemy_object.position.y as f32 <= player_y + 30.0
                && enemy_object.position.y as f32 >= player_y - 30.0
                && enemy_object.position.x as f32 >= player_x - 30.0
                && enemy_object.position.x as f32 <= player_x + 30.0 {

                if game_state.player_shield > 0.0 {
                    game_state.player_shield -= 25.0;
                } else {
                    game_state.player_hp -= 10.0;
                }
                commands.entity(entity).despawn();
            }
            if enemy_object.position.y as f32 <= -800.0 {
                commands.entity(entity).despawn();
            }

            transform.translation = Vec3::new(enemy_object.position.x as f32, enemy_object.position.y as f32, -1.0);
        });
}