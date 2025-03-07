// Add this to particles.rs to immediately improve performance

use std::ops::DerefMut;
use std::sync::atomic::AtomicPtr;
use std::time::Duration;
use bevy::log::tracing_subscriber::fmt::time;
use bevy::prelude::*;
use bevy::sprite::Anchor;
use bevy::utils::hashbrown::Equivalent;
use bevy::utils::HashMap;
use rand::random as rand_random;
use rayon::prelude::*;
use rs_physics::particles::{
    ParticleData, Quad, build_tree, compute_net_force
};
use rs_physics::particles::particle_interactions_barnes_hut_cosmological;
use rs_physics::utils::{DEFAULT_PHYSICS_CONSTANTS, PhysicsConstants};

const TIME_STEP_MODIFIERS: [f64; 4] = [0.16, 0.1, 0.064, 0.016];

// Convert from cosmological Particle to standard ParticleData
fn convert_to_particle_data(p: &particle_interactions_barnes_hut_cosmological::Particle) -> ParticleData {
    ParticleData {
        x: p.position.0,
        y: p.position.1,
        mass: p.mass
    }
}

// Convert from standard Quad to cosmological Quad
fn convert_quad(q: &Quad) -> particle_interactions_barnes_hut_cosmological::Quad {
    particle_interactions_barnes_hut_cosmological::Quad {
        cx: q.cx,
        cy: q.cy,
        half_size: q.half_size
    }
}

#[derive(Resource)]
pub struct CosmologicalSimulation {
    particles: Vec<particle_interactions_barnes_hut_cosmological::Particle>,
    standard_particles: Vec<ParticleData>, // Add this for efficient computation
    bounds: particle_interactions_barnes_hut_cosmological::Quad,
    standard_bounds: Quad, // Add this for efficient computation 
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
        let bounds = particle_interactions_barnes_hut_cosmological::Quad {
            cx: 0.0,
            cy: 0.0,
            half_size: initial_radius * 100.0  // Add some buffer around the simulation
        };

        let standard_bounds = Quad {
            cx: 0.0,
            cy: 0.0,
            half_size: initial_radius * 80.0
        };

        // Create particles in a Big Bang configuration
        let particles = particle_interactions_barnes_hut_cosmological::create_big_bang_particles(num_particles, initial_radius);

        // Create standard particles as well
        let standard_particles: Vec<ParticleData> = particles.iter()
            .map(|p| convert_to_particle_data(p))
            .collect();

        Self {
            particles,
            standard_particles,
            bounds,
            standard_bounds,
            time: 0.0,
            dt,
            theta,
            g,
            initial_radius,
        }
    }

    pub fn optimize_for_orbits(&mut self) {
        // Group particles by proximity to massive bodies
        let massive_indices: Vec<usize> = self.particles.iter()
            .enumerate()
            .filter(|(_, p)| p.mass > 1000.0)
            .map(|(i, _)| i)
            .collect();

        // Pre-compute orbital zones for massive particles
        let orbital_zones: Vec<(f64, f64, f64)> = massive_indices.iter()
            .map(|&i| {
                let p = &self.particles[i];
                // Return position and influence radius based on mass
                (p.position.0, p.position.1, p.mass.sqrt() * 0.2)
            })
            .collect();

        // Tag particles that are in orbital zones
        // This could be used to optimize force calculations
        for (i, p) in self.particles.iter_mut().enumerate() {
            let mut in_orbit = false;

            for &(center_x, center_y, radius) in &orbital_zones {
                let dx = p.position.0 - center_x;
                let dy = p.position.1 - center_y;
                let dist = (dx * dx + dy * dy).sqrt();

                if dist < radius && p.mass < 100.0 {
                    // This particle is in an orbital zone
                    in_orbit = true;
                    break;
                }
            }

            // You could use this flag to apply special physics/rendering
            // to particles in orbital zones
            if in_orbit {
                // Example: Adjust particle color to indicate it's in orbit
                p.density = p.density.max(0.8);  // Increase density parameter used for coloring
            }
        }
    }

    pub fn step(&mut self) {
        // Use the standard (more performant) Barnes-Hut implementation
        // First, build the Barnes-Hut tree with standard particles
        let tree = build_tree(&self.standard_particles, self.standard_bounds);

        let orbital_factors: Vec<f64> = self.standard_particles.iter()
            .map(|p| (p.mass / 1000.0).min(10.0).max(0.1))
            .collect();


        let a_ptr_mut = AtomicPtr::new(self);
        let a_ptr = unsafe{ &mut *a_ptr_mut.load(std::sync::atomic::Ordering::Relaxed) };

        let forces: Vec<(f64, f64)> = self.standard_particles.par_iter()
            .enumerate()
            .map(|(i, p)| {
                let (fx, fy) = compute_net_force(&tree, *p, self.theta, self.g);

                // Calculate perpendicular force component for orbital motion
                // This creates a small tangential force to encourage stable orbits
                let dist_sq = p.x * p.x + p.y * p.y;
                let dist = dist_sq.sqrt();

                // Only apply orbital adjustment to smaller particles
                if p.mass < 1000.0 && dist > 0.1 {
                    // Create perpendicular force vector (rotate 90 degrees)
                    let orbital_strength = 0.75 * orbital_factors[i];  // Adjust this multiplier
                    let perpendicular_fx = -fy * orbital_strength;
                    let perpendicular_fy = fx * orbital_strength;

                    (fx + perpendicular_fx, fy + perpendicular_fy)
                } else {
                    (fx, fy)
                }
            })
            .collect();

        // Update our cosmological particles with the computed forces
        a_ptr.particles
            .par_iter_mut()
            .enumerate()
            .for_each(|(i, p)| {

                let (fx, fy) = forces[i];



                // Apply force to update velocity
                p.apply_force(fx, fy, self.dt);

                let vel_magnitude = p.velocity.magnitude();
                let optimal_orbit_speed = (p.mass / 1000.0).sqrt() * 0.2; // Scale with particle mass
                let speed_diff = (vel_magnitude - optimal_orbit_speed).abs();

                // Apply drag that pushes velocity toward optimal orbital speed
                let drag_strength = if vel_magnitude > optimal_orbit_speed {
                    // Slow down faster particles
                    speed_diff * 0.015
                } else if vel_magnitude < optimal_orbit_speed * 0.314 {
                    // Speed up very slow particles
                    -speed_diff * 0.05
                } else {
                    // Minimal drag in the "orbital zone"
                    speed_diff * 0.001
                };

                let drag_angle = p.velocity.direction();
                let drag_fx = -drag_strength * drag_angle.x;
                let drag_fy = -drag_strength * drag_angle.y;

                // Apply drag force
                p.apply_force(drag_fx, drag_fy, self.dt);

                // Update position
                p.update_position(self.dt);

                // commented out for performance and cause it looks cooler
                //self.apply_boundary_conditions(p);

                let a_ptr = unsafe{ &mut *a_ptr_mut.load(std::sync::atomic::Ordering::Relaxed) };

                // Update the standard particle to stay in sync
                a_ptr.standard_particles[i].x = p.position.0;
                a_ptr.standard_particles[i].y = p.position.1;
        });

        self.time += self.dt;
    }

    pub fn apply_boundary_conditions(&self, p: &mut particle_interactions_barnes_hut_cosmological::Particle) {
        let bound_size = self.bounds.half_size * 4.0;

        // Periodic boundary conditions (wraparound)
        if p.position.0 < self.bounds.cx - self.bounds.half_size {
            p.position.0 += bound_size;
        } else if p.position.0 >= self.bounds.cx + self.bounds.half_size {
            p.position.0 -= bound_size;
        }

        if p.position.1 < self.bounds.cy - self.bounds.half_size {
            p.position.1 += bound_size;
        } else if p.position.1 >= self.bounds.cy + self.bounds.half_size {
            p.position.1 -= bound_size;
        }
    }

    pub fn modify_particle_masses(&mut self) {
        // Process in chunks to avoid excessive memory usage
        let chunk_size = 8_192;
        let a_ptr_mut = AtomicPtr::new(self);

        for chunk_start in (0..self.particles.len()).step_by(chunk_size) {
            let chunk_end = (chunk_start + chunk_size).min(self.particles.len());

            (chunk_start..chunk_end).into_par_iter().for_each(|i| {
                let a_ptr = unsafe { &mut *a_ptr_mut.load(std::sync::atomic::Ordering::Relaxed) };

                // Create a more diverse mass distribution
                let mass_type = rand_random::<f64>();

                if mass_type < 0.001 {
                    // Super massive "suns" (0.1% of particles)
                    a_ptr.particles[i].mass = std::f64::consts::PI * rand_random::<f64>().mul_add(5000.0, 2000.0);
                    a_ptr.particles[i].spin *= 20.0;
                } else if mass_type < 0.01 {
                    // Medium "planets" (0.9% of particles)
                    a_ptr.particles[i].mass = std::f64::consts::PI * rand_random::<f64>().mul_add(500.0, 100.0);
                    a_ptr.particles[i].spin *= 10.0;
                } else if mass_type < 0.1 {
                    // Small "asteroids" (9% of particles)
                    a_ptr.particles[i].mass = std::f64::consts::PI * rand_random::<f64>().mul_add(50.0, 20.0);
                    a_ptr.particles[i].spin *= 5.0;
                } else {
                    // Tiny "dust" (90% of particles)
                    a_ptr.particles[i].mass = std::f64::consts::PI * rand_random::<f64>().mul_add(10.0, 1.0);
                }

                // Update standard particle mass to match
                a_ptr.standard_particles[i].mass = a_ptr.particles[i].mass;
            });
        }
    }

    pub fn randomize_particle_directions(&mut self) {
        // Process in chunks to avoid excessive memory usage
        let chunk_size = 4_096;

        let a_ptr_mut = AtomicPtr::new(self);
        for chunk_start in (0..self.particles.len()).step_by(chunk_size) {
            let chunk_end = (chunk_start + chunk_size).min(self.particles.len());

            (chunk_start..chunk_end).into_par_iter().for_each(|i| {
                let a_ptr = unsafe{ &mut *a_ptr_mut.load(std::sync::atomic::Ordering::Relaxed) };
                let p = &mut a_ptr.particles[i];
                let angle_variation = rand_random::<f64>().mul_add(0.5, -0.5) * std::f64::consts::PI;
                let current_speed = p.velocity.magnitude();

                let current_dir = if current_speed > 0.0 {
                    (p.velocity.x / current_speed, p.velocity.y / current_speed)
                } else {
                    (0.0, 0.0)
                };

                // Rotate the direction by a random angle
                let cos_angle = angle_variation.cos();
                let sin_angle = angle_variation.sin();
                let new_x = current_dir.0 * cos_angle - current_dir.1 * sin_angle;
                let new_y = current_dir.0 * sin_angle + current_dir.1 * cos_angle;

                // Update velocity with new direction but maintain speed
                p.velocity.x = new_x * current_speed;
                p.velocity.y = new_y * current_speed;
            });
        };
    }

    pub fn get_particles(&self) -> &[particle_interactions_barnes_hut_cosmological::Particle] {
        &self.particles
    }

    pub fn get_particle_count(&self) -> usize {
        self.particles.len()
    }
}

// Reduced particle count for setup phase
pub fn setup(
    mut commands: Commands,
    time: Res<Time<Fixed>>,
) {
    // Initialize simulation parameters with reduced particle count
    let num_particles = 46_000;  // Reduced from 128,000 to improve initial loading
    let initial_radius = 25.0 * std::f64::consts::PI.ln_1p();
    let dt = time.timestep().as_secs_f64() * TIME_STEP_MODIFIERS[0];
    let theta = 0.85;  // Increased from 0.5 for better approximation/performance balance
    let g = 1.0 / std::f64::consts::PI;  // Gravitational constant

    // Create the simulation
    let mut simulation = CosmologicalSimulation::new(
        num_particles,
        initial_radius,
        dt,
        theta,
        g
    );

    // Modify some particles to have more mass
    simulation.modify_particle_masses();

    // Add some randomness to initial directions
    simulation.randomize_particle_directions();

    simulation.optimize_for_orbits();

    // Add the simulation as a resource
    commands.insert_resource(simulation);
}


#[derive(Component)]
pub struct ParticleId(usize);

pub fn update_simulation(
    mut sim_res: ResMut<CosmologicalSimulation>,
    mut query: Query<(&mut Transform, &mut Visibility, &ParticleId)>,
) {


    // Advance the simulation one time step
    sim_res.step();

    // Get particle data for rendering
    let particles = sim_res.get_particles();

    // Define batch size for processing entities
    let batch_size = 8_192;

    let visible_radius = 1440.0_f64;
    // Use batched processing to reduce memory pressure
    let mut items = query.iter_mut().collect::<Vec<_>>();
    for batch in items.chunks_mut(batch_size) {
        batch.par_iter_mut().for_each(|(ref mut transform,ref mut visibility,  particle_id)| {
            let particle = &particles[particle_id.0];

            if visibility.eq(&Visibility::Hidden) {
                return;
            }

            // Check if particle is worth rendering
            if particle.position.0.powi(2) + particle.position.1.powi(2) > visible_radius.powi(2) {
                // Make invisible to skip rendering
                visibility.toggle_visible_hidden();

                return;
            }


            // Scale based on mass and density for visual effect
            let scale_factor = (particle.mass.log10() * 0.75).max(1.0).min(10.0) as f32;

            // Update position
            transform.translation = Vec3::new(
                particle.position.0 as f32,
                particle.position.1 as f32,
                -2.0
            );

            // Update rotation based on particle direction and spin
            let direction = particle.velocity.direction();
            let rotation_angle = direction.x.atan2(direction.y) as f32;
            transform.rotation = Quat::from_rotation_z(rotation_angle + (particle.spin as f32 * 0.75));

            // Regular circular scale for larger particles
            transform.scale = Vec3::splat(scale_factor);

        });
    }
}

pub fn spawn_particles(
    mut commands: Commands,
    sim_res: Res<CosmologicalSimulation>,
) {
    let particles = sim_res.get_particles();
    let particle_count = sim_res.get_particle_count();

    // Use batch spawning to reduce memory pressure
    let batch_size = 8_192;

    for batch_start in (0..particle_count).step_by(batch_size) {
        let batch_end = (batch_start + batch_size).min(particle_count);

        for i in batch_start..batch_end {
            let particle = &particles[i];

            // Calculate color based on particle properties
            let hue = 360.0 * (i as f32 / particle_count as f32);
            let saturation = (particle.density as f32 * 0.5).clamp(0.35, 0.65);
            let lightness = ((particle.age as f32 * 0.01) + 0.5).clamp(0.65, 1.0);
            let color = Color::hsl(hue, saturation, lightness);

            // Initial position
            let x = particle.position.0 as f32;
            let y = particle.position.1 as f32;

            // Initial scale based on mass
            let scale = (particle.mass.log10() * 0.85).max(1.0).min(6.0) as f32;

            commands.spawn((
                Sprite {
                    color,
                    custom_size: Some(Vec2::new(1.0, 1.0)),
                    anchor: Anchor::Center,
                    ..Default::default()
                },
                Transform {
                    translation: Vec3::new(x, y, -2.0),
                    scale: Vec3::splat(scale),
                    ..Default::default()
                },
                ParticleId(i),
            ));
        }
    }
}