// Add this to particles.rs to immediately improve performance

use std::ops::DerefMut;
use std::sync::atomic::AtomicPtr;
use bevy::log::tracing_subscriber::fmt::time;
use bevy::prelude::*;
use bevy::sprite::Anchor;
use rand::random as rand_random;
use rayon::prelude::*;
use rs_physics::particles::{
    ParticleData, Quad, build_tree, compute_net_force
};
use rs_physics::particles::particle_interactions_barnes_hut_cosmological;
use rs_physics::utils::{DEFAULT_PHYSICS_CONSTANTS, PhysicsConstants};

// Use the standard Barnes-Hut implementation which is more performant
// Our models will bridge between the two implementations

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

    pub fn step(&mut self) {
        // Use the standard (more performant) Barnes-Hut implementation
        // First, build the Barnes-Hut tree with standard particles
        let tree = build_tree(&self.standard_particles, self.standard_bounds);

        // Calculate forces and update particles in parallel
        let forces: Vec<(f64, f64)> = self.standard_particles.par_iter()
            .map(|p| compute_net_force(&tree, *p, self.theta, self.g))
            .collect();

        let a_ptr_mut = AtomicPtr::new(self);
        let a_ptr = unsafe{ &mut *a_ptr_mut.load(std::sync::atomic::Ordering::Relaxed) };
        // Update our cosmological particles with the computed forces
        a_ptr.particles
            .par_iter_mut()
            .enumerate()
            .for_each(|(i, p)| {

                let (fx, fy) = forces[i];



                // Apply force to update velocity
                p.apply_force(fx, fy, self.dt);

                // add drag force to slow down particles
                let drag_force = p.velocity.magnitude().log(1.089) * 0.1;
                let drag_angle = p.velocity.direction();
                let drag_fx = -drag_force * drag_angle.x;
                let drag_fy = -drag_force * drag_angle.y;

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
        let chunk_size = 4_096;

        let a_ptr_mut = AtomicPtr::new(self);

        for chunk_start in (0..self.particles.len()).step_by(chunk_size) {
            let chunk_end = (chunk_start + chunk_size).min(self.particles.len());

            (chunk_start..chunk_end).into_par_iter().for_each(|i| {
                // Create some super massive particles
                let extra_dense = rand_random::<f64>() < 1.0/128.0;
                let a_ptr = unsafe{ &mut *a_ptr_mut.load(std::sync::atomic::Ordering::Relaxed) };
                if extra_dense {
                    a_ptr.particles[i].mass = std::f64::consts::PI * rand_random::<f64>().mul_add(2400.0, 360.0);
                    a_ptr.particles[i].spin *= 50.0; // Increase spin for massive particles

                    // Update standard particle mass to match
                    a_ptr.standard_particles[i].mass = a_ptr.particles[i].mass;
                }
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
        }
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
    let num_particles = 48_000;  // Reduced from 128,000 to improve initial loading
    let initial_radius = 2.0 * std::f64::consts::PI.ln_1p();
    let dt = time.timestep().as_secs_f64() * 0.064;
    let theta = 0.5;  // Increased from 0.5 for better approximation/performance balance
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
    let batch_size = 4_096;

    let visible_radius = 1440.0_f64;
    // Use batched processing to reduce memory pressure
    let mut items = query.iter_mut().collect::<Vec<_>>();
    for batch in items.chunks_mut(batch_size) {
        batch.par_iter_mut().for_each(|(ref mut transform,ref mut visibility,  particle_id)| {
            let particle = &particles[particle_id.0];

            // Check if particle is worth rendering
            if particle.position.0.powi(2) + particle.position.1.powi(2) > visible_radius.powi(2) {
                // Make invisible to skip rendering
                transform.scale = Vec3::ZERO;
                return;
            }
            // Update position
            transform.translation = Vec3::new(
                particle.position.0 as f32,
                particle.position.1 as f32,
                -2.0
            );

            // Update rotation based on particle direction and spin
            let direction = particle.velocity.direction();
            let rotation_angle = direction.x.atan2(direction.y) as f32;
            transform.rotation = Quat::from_rotation_z(rotation_angle + (particle.spin as f32 * 0.1));

            // Scale based on mass and density for visual effect
            let scale_factor = (particle.mass.log10() * 0.5).max(5.0).min(1.0) as f32;
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
    let batch_size = 4_096;

    for batch_start in (0..particle_count).step_by(batch_size) {
        let batch_end = (batch_start + batch_size).min(particle_count);

        for i in batch_start..batch_end {
            let particle = &particles[i];

            // Calculate color based on particle properties
            let hue = 360.0 * (i as f32 / particle_count as f32);
            let saturation = (particle.density as f32 * 0.5).clamp(0.45, 1.0);
            let lightness = ((particle.age as f32 * 0.01) + 0.5).clamp(0.25, 1.0);
            let color = Color::hsl(hue, saturation, lightness);

            // Initial position
            let x = particle.position.0 as f32 + rand::random_range(-1.0..=1.0);
            let y = particle.position.1 as f32 + rand::random_range(-1.0..=1.0);

            // Initial scale based on mass
            let scale = (particle.mass.log10() * 0.85).max(5.0).min(1.0) as f32;

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