use std::ops::DerefMut;
use bevy::asset::AssetContainer;
use bevy::prelude::*;
use rs_physics::particles::Simulation;
use rs_physics::utils::{DEFAULT_PHYSICS_CONSTANTS, PhysicsConstants};

#[derive(Resource)]
pub struct PhysicsSim(Simulation);

pub fn setup(mut commands: Commands) {
    // Initialize physics constants and simulation parameters
    let constants = PhysicsConstants {
        gravity: -DEFAULT_PHYSICS_CONSTANTS.gravity,
        ..DEFAULT_PHYSICS_CONSTANTS
    };

    let mut sim = Simulation::new(
        1000,               // number of particles
        (0.0, -400.0),         // initial position
        100.0,               // initial speed
        (0.0, 1.0),         // initial direction
        0.25,                // mass
        constants,
        0.016,              // time step (16ms per frame)
    )
        .expect("Failed to create simulation");

    sim.speeds.iter_mut().for_each(|s| *s = rand::random_range(50.0..200.0));
    sim.directions_x.iter_mut().for_each(|x| *x = rand::random_range(-1.0..1.0));
    sim.directions_y.iter_mut().for_each(|y| *y = rand::random_range(0.25..1.0));

    let physics_sim = PhysicsSim(sim);
    commands.insert_resource(physics_sim);

    // Optionally, spawn Bevy entities for visualization
}

#[derive(Component)]
pub struct ParticleId(usize);

pub fn update_simulation(
    mut sim_res: ResMut<PhysicsSim>,
    mut query: Query<(&mut Transform, &ParticleId)>,
) {
    // Advance the simulation one time step
    sim_res.0.step().expect("Simulation step failed");

    // Update each entity’s transform using the simulation’s positions.
    for (mut transform, particle_id) in query.iter_mut() {
        let x = sim_res.0.positions_x[particle_id.0];
        let y = sim_res.0.positions_y[particle_id.0];
        transform.translation = Vec3::new(x as f32, y as f32, 1.0);
    }
}

pub fn spawn_particles(
    mut commands: Commands,
    sim_res: Res<PhysicsSim>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    mut meshes: ResMut<Assets<Mesh>>,
) {
    for i in 0..sim_res.0.positions_x.len() {
        commands.spawn((
            Mesh2d(meshes.add(Circle::new(1.0))),
            MeshMaterial2d(materials.add(Color::srgb(1.0, 0.25, 0.15))),
            Transform {
                translation: Vec3::new(sim_res.0.positions_x[i] as f32, sim_res.0.positions_y[i] as f32, 1.0),
                ..Default::default()
            },
        )).insert(ParticleId(i));
    }
}