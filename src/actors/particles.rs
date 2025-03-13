// Optimized particles.rs with Structure of Arrays (SoA) implementation

use bevy::prelude::*;
use bevy::sprite::Anchor;
use rayon::prelude::*;
use rs_physics::particles::particle_interactions_barnes_hut_cosmological::{
    Particle, Quad as BHQuad, ParticleCollection,
    create_big_bang_particles_soa, modify_particle_masses_soa,
    simulate_step_soa
};
use rs_physics::models::Velocity2D;
use rs_physics::utils::fast_atan2;

#[derive(Resource)]
pub struct CosmologicalSimulation {
    particle_collection: ParticleCollection,
    bounds: BHQuad,
    time: f64,
    dt: f64,
    theta: f64,
    g: f64,
    initial_radius: f64,
}

impl CosmologicalSimulation {
    pub fn new(
        num_particles: usize,
        initial_radius: f64,
        dt: f64,
        theta: f64,
        g: f64
    ) -> Self {
        // Create bounding quad that encompasses the simulation area
        let bounds = BHQuad {
            cx: 0.0,
            cy: 0.0,
            half_size: initial_radius * 800.0  // Add some buffer around the simulation
        };

        // Create particles in a Big Bang configuration using SoA
        let particle_collection = create_big_bang_particles_soa(num_particles, initial_radius as f32);

        Self {
            particle_collection,
            bounds,
            time: 0.0,
            dt,
            theta,
            g,
            initial_radius,
        }
    }

    pub fn optimize_for_orbits(&mut self) {
        // Find massive bodies (those with mass > 1000.0)
        let mut massive_indices = Vec::new();
        for i in 0..self.particle_collection.count {
            if self.particle_collection.masses[i] > 1000.0 {
                massive_indices.push(i);
            }
        }

        if massive_indices.is_empty() {
            return; // No massive bodies to orbit around
        }

        // Pre-compute orbital zones for massive particles
        let orbital_zones: Vec<(f32, f32, f32)> = massive_indices.iter()
            .map(|&i| {
                // Return position and influence radius based on mass
                (
                    self.particle_collection.positions_x[i],
                    self.particle_collection.positions_y[i],
                    (self.particle_collection.masses[i].sqrt() * 0.2)
                )
            })
            .collect();

        // Tag particles that are in orbital zones by adjusting their density
        // (No parallelization here since it's just a one-time setup operation)
        for i in 0..self.particle_collection.count {
            // Skip massive bodies themselves
            if self.particle_collection.masses[i] >= 100.0 {
                continue;
            }

            // Check if this particle is in an orbital zone
            let px = self.particle_collection.positions_x[i];
            let py = self.particle_collection.positions_y[i];

            for &(center_x, center_y, radius) in &orbital_zones {
                let dx = px - center_x;
                let dy = py - center_y;
                let dist_sq = dx * dx + dy * dy;

                if dist_sq < radius * radius {
                    // In orbit - adjust density for visualization
                    self.particle_collection.densities[i] = self.particle_collection.densities[i].max(0.8);
                    break;
                }
            }
        }
    }

    pub fn step(&mut self) {
        // Execute the simulation step with all parameters
        simulate_step_soa(
            &mut self.particle_collection,
            self.bounds,
            self.theta as f32,
            self.g as f32,
            self.dt as f32,
            self.time as f32
        );

        // Apply orbital mechanics (handled by the modified simulate_step_soa)
        self.apply_orbital_mechanics();

        // Update simulation time
        self.time += self.dt;
    }

    fn apply_orbital_mechanics(&mut self) {
        // Pre-calculate orbital factors
        let particle_count = self.particle_collection.count;
        let orbital_factors: Vec<f32> = (0..particle_count)
            .map(|i| (self.particle_collection.masses[i] / 1000.0).min(10.0).max(0.1))
            .collect();

        // Process in smaller batches for better cache locality
        let chunk_size = 4096.min(particle_count);

        for chunk_start in (0..particle_count).step_by(chunk_size) {
            let chunk_end = (chunk_start + chunk_size).min(particle_count);

            for i in chunk_start..chunk_end {
                // Get particle data
                let vx = self.particle_collection.velocities_x[i];
                let vy = self.particle_collection.velocities_y[i];
                let mass = self.particle_collection.masses[i];

                // Skip massive bodies and nearly stationary particles
                if mass >= 1000.0 || (vx * vx + vy * vy) < 1e-6 {
                    continue;
                }

                // Calculate velocity magnitude
                let vel_magnitude = (vx * vx + vy * vy).sqrt();

                // Calculate orbital adjustment
                let orbital_strength = 0.75 * orbital_factors[i] * 0.01;
                let perpendicular_vx = -vy * orbital_strength;
                let perpendicular_vy = vx * orbital_strength;

                // Calculate optimal orbit speed and speed difference
                let optimal_orbit_speed = (mass / 1000.0).sqrt() * 0.2;
                let speed_diff = (vel_magnitude - optimal_orbit_speed).abs();

                // Determine drag strength based on current speed
                let drag_strength = if vel_magnitude > optimal_orbit_speed {
                    speed_diff * 0.015  // Slow down faster particles
                } else if vel_magnitude < optimal_orbit_speed * 0.314 {
                    -speed_diff * 0.05  // Speed up very slow particles
                } else {
                    speed_diff * 0.001  // Minimal drag in the "orbital zone"
                };

                // Calculate drag components
                let speed_reciprocal = if vel_magnitude > 0.0001 { 1.0 / vel_magnitude } else { 0.0 };
                let drag_vx = -vx * speed_reciprocal * drag_strength;
                let drag_vy = -vy * speed_reciprocal * drag_strength;

                // Apply velocity adjustments
                self.particle_collection.velocities_x[i] += perpendicular_vx + drag_vx;
                self.particle_collection.velocities_y[i] += perpendicular_vy + drag_vy;
            }
        }
    }

    pub fn modify_particle_masses(&mut self) {
        // Use the SoA implementation to modify masses
        modify_particle_masses_soa(&mut self.particle_collection);
    }

    pub fn get_particle_count(&self) -> usize {
        self.particle_collection.count
    }

    // Efficient particle accessor that avoids unnecessary conversions
    pub fn get_particle(&self, index: usize) -> Particle {
        Particle {
            position: (
                self.particle_collection.positions_x[index] as f64,
                self.particle_collection.positions_y[index] as f64
            ),
            velocity: Velocity2D {
                x: self.particle_collection.velocities_x[index] as f64,
                y: self.particle_collection.velocities_y[index] as f64
            },
            mass: self.particle_collection.masses[index] as f64,
            spin: self.particle_collection.spins[index] as f64,
            age: self.particle_collection.ages[index] as f64,
            density: self.particle_collection.densities[index] as f64
        }
    }
}

// Particle identifier component
#[derive(Component)]
pub struct ParticleId(usize);

// Bevy systems

// Setup system - initialize the simulation
pub fn setup(
    mut commands: Commands,
    time: Res<Time<Fixed>>,
) {

    // Initialize simulation with parameters tuned for performance
    let num_particles = 24_000;  // Adjust based on your performance requirements
    let initial_radius = 8.0 * std::f64::consts::PI.ln_1p();
    let dt = time.timestep().as_secs_f64();
    let theta = 0.85;  // Barnes-Hut approximation parameter
    let g = 1.0 / std::f64::consts::PI;  // Gravitational constant

    info!("Creating simulation with {} particles", num_particles);
    let start_time = std::time::Instant::now();

    let mut simulation = CosmologicalSimulation::new(
        num_particles,
        initial_radius,
        dt,
        theta,
        g
    );

    // Set up particle masses
    simulation.modify_particle_masses();

    // Set up orbital dynamics
    simulation.optimize_for_orbits();

    info!("Simulation created in {:.2?}", start_time.elapsed());

    // Add the simulation as a resource
    commands.insert_resource(simulation);
}

// Update simulation system - advances physics and updates entities
pub fn update_simulation(
    mut sim_res: ResMut<CosmologicalSimulation>,
    mut query: Query<(&mut Transform, &mut Visibility, &ParticleId)>,
) {
    // Advance the simulation
    let sim_start = std::time::Instant::now();
    sim_res.step();
    let sim_duration = sim_start.elapsed();

    // Skip rendering update if simulation took too long (slow frames)
    if sim_duration > std::time::Duration::from_millis(64) {
        warn!("Simulation step too slow: {:.2?}", sim_duration);
        return;
    }

    // Rendering constants
    let visible_radius = 4092.0_f64;

    query.iter_mut().for_each(|(mut transform, mut visibility, particle_id)| {
        // Skip if ID is out of bounds
        if particle_id.0 >= sim_res.get_particle_count() {
            return;
        }

        // Skip already hidden particles
        if *visibility == Visibility::Hidden {
            return;
        }

        // Get particle data
        let particle = sim_res.get_particle(particle_id.0);

        // Check if particle is worth rendering
        let dist_squared = particle.position.0.powi(2) + particle.position.1.powi(2);
        if dist_squared > visible_radius.powi(2) {
            // Make invisible to skip rendering
            *visibility = Visibility::Hidden;
            return;
        }

        // Scale based on mass
        let scale_factor = (particle.mass.log10() * 0.75).max(1.0).min(10.0) as f32;

        // Update transform
        transform.translation.x = particle.position.0 as f32;
        transform.translation.y = particle.position.1 as f32;

        // Update rotation
        let direction = particle.velocity.direction();
        let rotation_angle = fast_atan2(direction.y as f32, direction.x as f32);
        let spin_factor = (particle.spin as f32 * 0.75).min(std::f32::consts::PI * 2.0);
        transform.rotation = Quat::from_rotation_z(rotation_angle + spin_factor);

        // Update scale
        transform.scale = Vec3::splat(scale_factor);
    });
}

// Spawn particles system
pub fn spawn_particles(
    mut commands: Commands,
    sim_res: Res<CosmologicalSimulation>,
) {
    let start_time = std::time::Instant::now();
    let particle_count = sim_res.get_particle_count();

    info!("Spawning {} particles", particle_count);

    // Use batch spawning to reduce memory pressure
    let batch_size = 2_048; // Smaller batch size to maintain responsiveness

    for batch_start in (0..particle_count).step_by(batch_size) {
        let batch_end = (batch_start + batch_size).min(particle_count);

        // Prepare batch of commands
        let mut batch_commands = Vec::with_capacity(batch_end - batch_start);

        for i in batch_start..batch_end {
            let particle = sim_res.get_particle(i);

            // Calculate color based on particle properties
            let hue = 360.0 * (i as f32 / particle_count as f32);
            let saturation = (particle.density as f32 * 0.5).clamp(0.35, 0.65);
            let lightness = ((particle.age as f32 * 0.01) + 0.5).clamp(0.65, 1.0);
            let color = Color::hsl(hue, saturation, lightness);

            // Create the entity
            batch_commands.push((
                Sprite {
                    color,
                    custom_size: Some(Vec2::new(1.0, 1.0)),
                    anchor: Anchor::Center,
                    ..Default::default()
                },
                Transform {
                    translation: Vec3::new(
                        particle.position.0 as f32,
                        particle.position.1 as f32,
                        -2.0
                    ),
                    scale: Vec3::splat((particle.mass.log10() * 0.85).max(1.0).min(10.0) as f32),
                    ..Default::default()
                },
                ParticleId(i),
            ));
        }

        // Spawn all entities in this batch
        commands.spawn_batch(batch_commands);
    }

    info!("Particles spawned in {:.2?}", start_time.elapsed());
}