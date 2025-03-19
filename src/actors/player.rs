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
use rs_physics::utils::{DEFAULT_PHYSICS_CONSTANTS, fast_atan2, PhysicsConstants};
use crate::hud::{EnergyBar, HpBar, ScoreCounter, ShieldBar};
use crate::state::MainGameState;

pub(crate) const GROUND_LEVEL: f64 = -860.0;

const PHYSICS_CONSTANTS: PhysicsConstants = PhysicsConstants {
    gravity: 0.0,
    ground_level: GROUND_LEVEL,
    ..DEFAULT_PHYSICS_CONSTANTS
};

#[derive(Component)]
pub struct PhysicsSystem2D(pub rs_physics::forces::PhysicsSystem2D);

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
    let player_object = ObjectIn2D::new(65.0, 0.0, 0.0, (-400.0, -300.0));
    let player_color = Color::srgb(0.1, 0.1, 0.1);

    commands
        .spawn((
            Player,
            Mesh2d(
                meshes.add(Circle::new(30.0))
            ),
            MeshMaterial2d(materials.add(player_color)),
            Transform {
                translation: Vec3::new(-400.0, -300.0, 1.0),
                ..Default::default()
            },
            PhysicsSystem2D::new(PHYSICS_CONSTANTS, player_object),
        ));

}

pub fn camera_movement(
    query: Query<(&Player, &Transform)>,
    mut shield_bar_query: Query<&mut Transform, (With<ShieldBar>, Without<Camera2d>, Without<Player>, Without<HpBar>, Without<EnergyBar>, Without<ScoreCounter>)>,
    mut hp_bar_query: Query<&mut Transform, (With<HpBar>, Without<Camera2d>, Without<Player>, Without<EnergyBar>, Without<ShieldBar>, Without<ScoreCounter>)>,
    mut energy_bar_query: Query<&mut Transform, (With<EnergyBar>, Without<Camera2d>, Without<Player>, Without<HpBar>, Without<ShieldBar>, Without<ScoreCounter>)>,
    mut score_counter_query: Query<&mut Transform, (With<ScoreCounter>, Without<Camera2d>, Without<Player>, Without<HpBar>, Without<ShieldBar>, Without<EnergyBar>)>,
    mut camera_query: Query<&mut Transform, (With<Camera2d>, Without<Player>, Without<HpBar>, Without<EnergyBar>, Without<ShieldBar>, Without<ScoreCounter>)>,
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

    let mut score_counter_transform = score_counter_query.iter_mut()
        .next()
        .expect("There should only be one score counter");

    camera_transform.translation.x = player_transform.translation.x * 0.5;
    camera_transform.translation.y = player_transform.translation.y * 0.75;

    shield_bar_transform.translation.x = camera_transform.translation.x - 850.0;
    shield_bar_transform.translation.y = camera_transform.translation.y + 520.0;

    hp_bar_transform.translation.x = camera_transform.translation.x - 850.0;
    hp_bar_transform.translation.y = camera_transform.translation.y + 500.0;

    energy_bar_transform.translation.x = camera_transform.translation.x - 850.0;
    energy_bar_transform.translation.y = camera_transform.translation.y + 480.0;

    score_counter_transform.translation.x = camera_transform.translation.x + 850.0;
    score_counter_transform.translation.y = camera_transform.translation.y + 500.0;
}

pub fn player_movement_physics (
    mut player_query: Query<&mut PhysicsSystem2D>,
    time: Res<Time<Fixed>>,
) {
    player_query
        .par_iter_mut()
        .for_each(|mut physics_system| {
            physics_system.0.update(1.-time.timestep().as_secs_f64());


            let player_obj = physics_system.0.get_object_mut(0).unwrap();

            // Apply velocity damping - updated to work with velocity components
            player_obj.velocity.x *= 0.98;
            player_obj.velocity.y *= 0.98;

            // Check if velocity is very small and zero it out if so
            if player_obj.speed() < 1.0 {
                player_obj.velocity.x = 0.0;
                player_obj.velocity.y = 0.0;
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
    let base_magnitude = 50.0;

    if keyboard_input.just_pressed(KeyCode::Space) && game_state.player_energy >= 20.0 {
        let player_phys_obj = physics_system.0.get_object_mut(0).unwrap();

        let angle = fast_atan2(player_phys_obj.velocity.y as f32, player_phys_obj.velocity.x as f32);

        let magnitude = base_magnitude * 20.0;

        // Apply thrust along this angle
        player_phys_obj.add_force(Force::Thrust { magnitude, angle: angle as f64 });

        game_state.player_energy -= 20.0;
    }

    if keyboard_input.pressed(KeyCode::KeyW) {
        let player_phys_obj = physics_system.0.get_object_mut(0).unwrap();

        let angle = fast_atan2(base_magnitude as f32, player_phys_obj.velocity.x as f32);

        let magnitude = if player_phys_obj.velocity.x == 0.0 {
            base_magnitude * 2.0
        } else {
            base_magnitude
        };

        // Apply thrust along this angle
        player_phys_obj.add_force(Force::Thrust { magnitude, angle: angle as f64 });
    }

    if keyboard_input.pressed(KeyCode::KeyA) {
        let player_phys_obj = physics_system.0.get_object_mut(0).unwrap();

        let angle = fast_atan2(player_phys_obj.velocity.y as f32, -base_magnitude as f32);

        let magnitude = if player_phys_obj.velocity.y == 0.0 {
            base_magnitude * 2.0
        } else {
            base_magnitude
        };

        // Apply thrust along this angle
        player_phys_obj.add_force(Force::Thrust { magnitude, angle: angle as f64 });
    }

    if keyboard_input.pressed(KeyCode::KeyD) {
        let player_phys_obj = physics_system.0.get_object_mut(0).unwrap();

        let angle = fast_atan2(player_phys_obj.velocity.y as f32, base_magnitude as f32);

        let magnitude = if player_phys_obj.velocity.y == 0.0 {
            base_magnitude * 2.0
        } else {
            base_magnitude
        };

        player_phys_obj.add_force(Force::Thrust { magnitude, angle: angle as f64 });
    }

    if keyboard_input.pressed(KeyCode::KeyS) {
        let player_phys_obj = physics_system.0.get_object_mut(0).unwrap();

        // Compute the angle from the tangent vector
        let angle = fast_atan2(-base_magnitude as f32, player_phys_obj.velocity.x as f32);

        let magnitude = if player_phys_obj.velocity.x == 0.0 {
            base_magnitude * 2.0
        } else {
            base_magnitude
        };


        player_phys_obj.add_force(Force::Thrust { magnitude, angle: angle as f64 });
    }
}