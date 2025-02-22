use std::cmp::{max, min};
use std::thread::spawn;
use bevy::input::InputSystem;
use bevy::input::keyboard::KeyboardInput;
use bevy::prelude::*;
use bevy::sprite::Anchor;
use bevy::text::cosmic_text::rustybuzz::Direction;
use rs_physics::forces::Force;
use rs_physics::interactions::apply_force;
use rs_physics::models::{Direction2D, FromCoordinates, ObjectIn2D, ToCoordinates};
use rs_physics::physics::calculate_terminal_velocity;
use rs_physics::utils::{DEFAULT_PHYSICS_CONSTANTS, PhysicsConstants};

const GROUND_LEVEL: f64 = -100.0;

#[derive(Component)]
pub struct PhysicsSystem2D(rs_physics::forces::PhysicsSystem2D);

impl PhysicsSystem2D {
    fn new(constants: rs_physics::utils::PhysicsConstants, player_object: ObjectIn2D) -> Self {
        // Invert gravity value here to make it act downwards
        // (the DEFAULT_PHYSICS_CONSTANTS.gravity is a positive f64)
        let constants = PhysicsConstants {
            gravity: -constants.gravity,
            ground_level: GROUND_LEVEL,
            ..DEFAULT_PHYSICS_CONSTANTS
        };
        let mut physics_system = rs_physics::forces::PhysicsSystem2D::new(constants);
        physics_system.add_object(player_object);
        physics_system.apply_gravity();
        physics_system.apply_drag(0.45, 0.5);
        Self (physics_system)
    }

}

#[derive(Component)]
pub struct Player;

pub fn setup_player(mut commands: Commands) {
    commands.spawn(Camera2d::default());

    // Player
    let player_object = ObjectIn2D::new(65.0, 0.0, (0.0, 0.0), (-200.0, GROUND_LEVEL));
    commands
        .spawn((
            Player,
            Sprite {
                color: Color::srgb(0.5, 1.0, 0.5),
                custom_size: Some(Vec2::new(30.0, 50.0)),
                anchor: Anchor::BottomCenter,
                ..default()
            },
            Transform::from_xyz(-300.00, GROUND_LEVEL as f32, 0.0),
            PhysicsSystem2D::new(DEFAULT_PHYSICS_CONSTANTS, player_object),
        ));

    // Ground
    commands.spawn((
        Sprite {
            color: Color::srgb(0.5, 0.5, 0.5),
            custom_size: Some(Vec2::new(800.0, 10.0)),
            anchor: Anchor::TopLeft,
            ..default()
        },
        Transform::from_xyz(-400.0, GROUND_LEVEL as f32, 0.0)
    ));
}

pub fn player_movement(
    mut player_query: Query<(&mut Transform, &mut PhysicsSystem2D), With<Player>>,
    time: Res<Time>,
) {
    if let Ok((mut transform, mut physics_system)) =
        player_query.get_single_mut() {

        physics_system.0.update(time.delta_secs_f64() * 60.0);

        let mut player_obj = physics_system.0.get_object_mut(0).unwrap();

        player_obj.velocity = player_obj.velocity * 0.99;
        if player_obj.velocity.abs() < 0.5 {
            player_obj.velocity = 0.0;
            player_obj.direction.x = 0.0;
        }

        if player_obj.position.x >= 400.0 || player_obj.position.x <= -400.0 {
            player_obj.position.x = player_obj.position.x.clamp(-400.0, 400.0);
            player_obj.direction.x = 0.0;
        }

        transform.translation.x = player_obj.position.x as f32;
        transform.translation.y = player_obj.position.y as f32;

    }
}

pub fn player_input(
    mut events: EventReader<KeyboardInput>,
    mut player_query: Query<(&mut PhysicsSystem2D), With<Player>>,
) {

    for e in events.read() {
        if let Ok(mut physics_system) =
            player_query.get_single_mut() {

            if e.state.is_pressed() && e.key_code == KeyCode::Space {
                let mut player_phys_obj = physics_system.0.get_object_mut(0).unwrap();
                if player_phys_obj.position.y == GROUND_LEVEL {

                    // this is a bit hacky, but it makes the movement feel better.
                    if player_phys_obj.velocity.abs() <= 2.5 {
                        player_phys_obj.velocity = 0.0;
                        player_phys_obj.direction.x = 0.0;
                    }

                    let max_offset = std::f64::consts::FRAC_PI_6; // ~30 degrees
                    // Offset angle based on x direction (between -1.0 and 1.0 inclusive)
                    let angle_offset = player_phys_obj.direction.x * max_offset;
                    // Subtract offset from π/2 so that if x > 0, jump tilts right (angle becomes < π/2)
                    // and if x < 0, jump tilts left (angle becomes > π/2).
                    let thrust_angle = std::f64::consts::FRAC_PI_2 - angle_offset;
                    let base_magnitude = 9600.0;
                    // the only issue with this is that it takes away from the vertical thrust.
                    // this can be fixed by increasing the magnitude of the thrust.
                    // which is why the magnitude is doubled when x != 0.
                    // (this is less realistic, but it makes the movement feel better)
                    let magnitude = if player_phys_obj.direction.x.abs() != 0.0 {
                        base_magnitude * 2.0
                    } else {
                        base_magnitude
                    };

                    player_phys_obj.add_force(Force::Thrust {
                        magnitude,
                        angle: thrust_angle,
                    });
                }
            }
            if e.state.is_pressed() && e.key_code == KeyCode::KeyA {
                let mut player_phys_obj = physics_system.0.get_object_mut(0).unwrap();
                // Apply leftward thrust.
                if e.repeat {
                    player_phys_obj.add_force(Force::Thrust { magnitude: 200.0, angle: std::f64::consts::PI });
                } else {
                    player_phys_obj.add_force(Force::Thrust { magnitude: 600.0, angle: std::f64::consts::PI });
                }
            }
            if e.state.is_pressed() && e.key_code == KeyCode::KeyD {
                let mut player_phys_obj = physics_system.0.get_object_mut(0).unwrap();
                // Apply rightward thrust.
                if e.repeat {
                    player_phys_obj.add_force(Force::Thrust { magnitude: 200.0, angle: 0.0 });
                } else {
                    player_phys_obj.add_force(Force::Thrust { magnitude: 600.0, angle: 0.0 });
                }
            }

        }
    }
}