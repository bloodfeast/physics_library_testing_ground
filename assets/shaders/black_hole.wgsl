#import bevy_sprite::mesh2d_vertex_output::VertexOutput

struct BlackHoleProperties {
    center: vec2<f32>,
    radius: f32,
    accretion_radius: f32,
    distortion_strength: f32,
    rotation_speed: f32,
    time: f32,
    glow_color: vec4<f32>,
}

struct BlackHoleMaterial {
    properties: BlackHoleProperties,
}

@group(1) @binding(0)
var<storage, read> material: BlackHoleMaterial;
@group(2) @binding(0)
var<uniform> properties: BlackHoleProperties;

// Function to create random stars/dots
fn random(st: vec2<f32>) -> f32 {
    return fract(sin(dot(st, vec2<f32>(12.9898, 78.233))) * 43758.5453);
}

// Function to create a starfield effect
fn stars(uv: vec2<f32>, seed: f32) -> f32 {
    // Create different scales of stars
    let star1 = step(0.98, random(floor(uv * 500.0 + seed)));
    let star2 = step(0.98, random(floor(uv * 200.0 + seed + 10.0)));
    let star3 = step(0.98, random(floor(uv * 100.0 + seed + 20.0)));

    // Combine stars of different sizes
    return star1 * 0.5 + star2 * 0.7 + star3;
}

@fragment
fn fragment(in: VertexOutput) -> @location(0) vec4<f32> {
    // Get center and UV coordinates
    let center_pt = vec2<f32>(0.5, 0.5);
    let uv = in.uv;
    let dist = distance(uv, center_pt);
    let dir = normalize(uv - center_pt);

    // Define radii
    let event_horizon_radius = properties.radius;
    let outer_radius = 0.5;
    let lensing_ring_start = event_horizon_radius;
    let lensing_ring_end = event_horizon_radius * 3.14;

    // Black hole center (event horizon)
    if (dist < event_horizon_radius) {
        return vec4<f32>(0.0, 0.0, 0.0, 1.0);
    }

    // Create the Einstein ring effect (the bright ring of distorted light)
    let ring_width = 0.002;
    let ring_intensity = smoothstep(lensing_ring_start, lensing_ring_start + ring_width/2.0, dist) *
                        (1.0 - smoothstep(lensing_ring_start + ring_width/2.0, lensing_ring_start + ring_width, dist));

    // Make ring pulsate slightly
    let pulse = (sin(properties.time * 1.0) * 0.2 + 0.9);

    // Make Einstein ring orange instead of white
    let ring_color = vec4<f32>(1.0, 0.7, 0.3, 1.0) * pulse * 0.7;

    // Create the accretion disk
    let accretion_start = event_horizon_radius * 0.2;
    let accretion_end = properties.accretion_radius;
    let accretion_disk = smoothstep(accretion_start, accretion_start + 0.01, dist) *
                        (1.0 - smoothstep(accretion_end - 0.01, accretion_end, dist));

    // Create animated pattern for accretion disk
    let angle = atan2(dir.y, dir.x);
    let pattern_time = properties.time * properties.rotation_speed;
    let disk_pattern = sin(angle * 10.0 + pattern_time) * 0.5 + 0.5;

    // CRITICAL: Use much stronger colors with reduced brightness
    // Pure white tends to dominate when colors add together
    let deep_orange = vec4<f32>(1.0, 0.4, 0.0, 1.0) * 0.6;  // Deep orange with reduced intensity
    let off_white = vec4<f32>(1.0, 0.9, 0.8, 1.0) * 0.7;    // Slightly off-white with reduced intensity

    // Use non-linear gradient for more obvious transition
    let gradient_pos = pow((dist - event_horizon_radius) / (properties.accretion_radius - event_horizon_radius), 2.0);

    // Apply gradient with stronger orange bias
    let disk_color = mix(
        off_white,       // Inner (slightly off-white)
        deep_orange,     // Outer (deep orange)
        clamp(gradient_pos * 1.5, 0.0, 1.0)
    );

    // Create visual lensing effect
    let lensing_strength = properties.distortion_strength * 0.001;
    let lensing_region = smoothstep(lensing_ring_start, lensing_ring_end, dist) *
                        (1.0 - smoothstep(lensing_ring_end, lensing_ring_end * 2.0, dist));

    // Create simulated lensed stars
    // This creates the illusion of stars "wrapping" around the black hole
    let lensed_angle = angle + properties.time * 0.1; // Slowly rotating effect
    let lensed_uv = center_pt + vec2<f32>(
        cos(lensed_angle) * dist * (1.0 + lensing_strength) + cos(properties.rotation_speed * lensed_angle * 0.314),
        sin(lensed_angle) * dist * (1.0 - lensing_strength) - sin(properties.rotation_speed * lensed_angle * 0.314)
    );
    let lensed_stars = stars(lensed_uv, properties.time * 0.02) * lensing_region;
    let star_color = vec4<f32>(0.8, 0.8, 1.0, lensed_stars) * 0.4; // Reduced star brightness

    // Create outer glow effect
    let glow_start = event_horizon_radius * 2.0;
    let glow_end = event_horizon_radius * 4.0;
    let glow_intensity = smoothstep(glow_end, glow_start, dist) * 0.4; // Reduced intensity

    // Make glow strongly orange
    let glow_gradient = smoothstep(glow_start, glow_end, dist * 2.0);
    let glow_color = mix(off_white, deep_orange, glow_gradient);
    let glow = glow_color * glow_intensity;

    // Combine all effects with reduced intensity to prevent white saturation
    let accretion = disk_color * accretion_disk * (disk_pattern * 0.1 + 0.25);
    let einstein_ring = ring_color * ring_intensity;

    // Final composition
    let final_color = max(accretion, einstein_ring) + star_color + glow;

    // Make sure to add proper alpha for transparency
    let alpha = min(1.0, final_color.a + glow_intensity + ring_intensity + accretion_disk + lensed_stars);

    // If outside the maximum radius, fully transparent
    if (dist > outer_radius) {
        return vec4<f32>(0.0, 0.0, 0.0, 0.0);
    }

    return vec4<f32>(final_color.rgb, alpha);
}