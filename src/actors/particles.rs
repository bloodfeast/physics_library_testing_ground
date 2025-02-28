use std::ops::DerefMut;
use std::sync::atomic::AtomicPtr;
use std::sync::Mutex;
use bevy::asset::AssetContainer;
use bevy::prelude::*;
use bevy::sprite::Anchor;
use rs_physics::particles::{build_tree, compute_net_force, ParticleData, Quad, Simulation};
use rs_physics::utils::{DEFAULT_PHYSICS_CONSTANTS, PhysicsConstants};
use rayon::prelude::*;

#[derive(Resource)]
pub struct PhysicsSim(Simulation);

pub fn setup(
    mut commands: Commands,
) {
    // Initialize physics constants and simulation parameters
    let constants = PhysicsConstants {
        gravity: 0.0,
        air_density: 0.0,
        ..DEFAULT_PHYSICS_CONSTANTS
    };

    let mut sim = Simulation::new(
        42_000,               // number of particles
        (0.0, 0.0),         // initial position
        200.0 / std::f64::consts::PI,               // initial speed
        (0.0, 0.25),         // initial direction
        0.15,                // mass
        constants,
        0.0016,              // time step (0.0016 is essentially slowing down the simulation, ~0.16 looks more natural imo) nut this creates that cool 'big bang' effect
    )
        .expect("Failed to create simulation");

    sim.directions_x.par_iter_mut().for_each(|y| *y = std::f64::consts::PI * rand::random_range(-0.25..0.25));
    sim.directions_y.par_iter_mut().for_each(|y| *y = std::f64::consts::PI * rand::random_range(-0.25..0.25));

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
    query.par_iter_mut()
        .for_each(|(mut transform, particle_id)| {
            let x = sim_res.0.positions_x[particle_id.0];
            let y = sim_res.0.positions_y[particle_id.0];
            transform.translation = Vec3::new(x as f32, y as f32, -2.0);
        });
}

pub fn update_forces(
    mut sim_res: ResMut<PhysicsSim>,
) {
    // 1. Convert simulation arrays into a Vec<ParticleData>
    let particle_count = sim_res.0.positions_x.len();
    let particles: Vec<ParticleData> = (0..particle_count)
        .into_par_iter()
        .map(|i| ParticleData {
            x: sim_res.0.positions_x[i],
            y: sim_res.0.positions_y[i],
            mass: sim_res.0.masses[i],
        })
        .collect();

    // 2. Define a quad that bounds the simulation (adjust as needed)
    let bounding_quad = Quad { cx: 0.0, cy: 0.0, half_size: 1.0 / std::f64::consts::PI };

    // 3. Build the Barnes–Hut tree
    let tree = build_tree(&particles, bounding_quad);

    // Barnes–Hut parameters
    let theta = std::f64::consts::FRAC_2_SQRT_PI; // controls approximation accuracy
    let g = 0.314;

    let mut mutex_sim_res = AtomicPtr::new(sim_res.deref_mut());

    // 4. For each particle, compute net force and update velocity/position.
    (0..particle_count)
        .into_par_iter()
        .for_each(|i| {
            let sim_res = unsafe { &mut *mutex_sim_res.load(std::sync::atomic::Ordering::Relaxed) };
            let p = ParticleData {
                x: sim_res.0.positions_x[i],
                y: sim_res.0.positions_y[i],
                mass: sim_res.0.masses[i],
            };

            // Compute the net force from the Barnes–Hut tree.
            let (force_x, force_y) = compute_net_force(&tree, p, theta, g);

            // Compute acceleration: a = F / m.
            let ax = force_x / p.mass;
            let ay = force_y / p.mass;

            // Recover the current velocity components.
            let vx = sim_res.0.speeds[i] * sim_res.0.directions_x[i];
            let vy = sim_res.0.speeds[i] * sim_res.0.directions_y[i];

            // Update velocity with acceleration.
            let new_vx = vx + ax * sim_res.0.dt;
            let new_vy = vy + ay * sim_res.0.dt;

            // Recompute speed and normalize direction.
            let new_speed = (new_vx * new_vx + new_vy * new_vy).sqrt().log(g);
            sim_res.0.speeds[i] = new_speed;
            if new_speed != 0.0 {
                sim_res.0.directions_x[i] = new_vx / new_speed;
                sim_res.0.directions_y[i] = new_vy / new_speed;
            }

            // Update position based on the new velocity.
            sim_res.0.positions_x[i] += new_vx * sim_res.0.dt;
            sim_res.0.positions_y[i] += new_vy * sim_res.0.dt;
        });

}

pub fn spawn_particles(
    mut commands: Commands,
    sim_res: Res<PhysicsSim>,
) {
    let particle_count = sim_res.0.positions_x.len();
    for i in 0..particle_count {
        let color = Color::hsl(360. * i as f32 / particle_count as f32, rand::random_range(0.45..1.0), rand::random_range(0.5..0.95));
        let (x, y) = (
            1.0 / i as f32 * 0.314 / rand::random_range(-100.0..100.0) * std::f32::consts::PI,
            1.0 / i as f32 * 0.314 / rand::random_range(-100.0..100.0) * std::f32::consts::PI
        );
        commands.spawn((
            Sprite {
                color,
                custom_size: Some(Vec2::new(1.0, 1.0)),
                anchor: Anchor::Center,
                ..Default::default()
            },
            Transform {
                translation: Vec3::new(x, y, -2.0),
                ..Default::default()
            },
        )).insert(ParticleId(i));
    }
}