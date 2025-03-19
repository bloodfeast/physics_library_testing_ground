// Distortion shader for the background particles
#import bevy_sprite::mesh2d_vertex_output::VertexOutput as MeshVertexOutput

@group(1) @binding(0)
var target_texture: texture_2d<f32>;
@group(1) @binding(1)
var target_sampler: sampler;

struct DistortionSettings {
    player_position: vec2<f32>,
    distortion_strength: f32,
    screen_dimensions: vec2<f32>,
    time: f32,
}

@group(2) @binding(0)
var<uniform> settings: DistortionSettings;

// Function to calculate gravitational lens distortion
fn calculate_distortion(uv: vec2<f32>, center: vec2<f32>, strength: f32, time: f32) -> vec2<f32> {
    // Vector from center to current position
    let dir = center - uv;
    let dist = length(dir);

    // Avoid division by zero
    let safe_dist = max(dist, 0.001);

    // The closer to the center, the stronger the distortion
    let distortion_intensity = strength / (safe_dist * safe_dist * 40.0);

    // Add a subtle time-based movement to make the distortion appear dynamic
    let time_factor = sin(time * 0.5) * 0.1;
    let adjusted_strength = distortion_intensity * (1.0 + time_factor);

    // Apply gravitational lens effect (pulling towards center)
    let distorted_uv = uv + normalize(dir) * adjusted_strength;

    // Add a subtle rotation near the center
    if (dist < 0.3) {
        let rotation_speed = time * 0.2;
        let rotation_angle = rotation_speed * (1.0 - dist / 0.3);
        let s = sin(rotation_angle);
        let c = cos(rotation_angle);
        let rotated_dir = vec2<f32>(
            dir.x * c - dir.y * s,
            dir.x * s + dir.y * c
        );
        return center - rotated_dir * (1.0 - distortion_intensity * 0.5);
    }

    return distorted_uv;
}

@fragment
fn fragment(in: MeshVertexOutput) -> @location(0) vec4<f32> {
    // Get UV coordinates
    let uv = in.uv;

    // Calculate center position in UV space (0-1)
    let center_position = settings.player_position / settings.screen_dimensions;

    // Apply the distortion effect
    let distorted_uv = calculate_distortion(uv, center_position, settings.distortion_strength, settings.time);

    // Sample the texture with the distorted coordinates
    let color = textureSample(target_texture, target_sampler, distorted_uv);

    // Add a subtle glow near the black hole center
    let dist_to_center = length(uv - center_position);
    let glow_intensity = smoothstep(0.3, 0.1, dist_to_center) * 0.2;
    let glow_color = vec4<f32>(0.2, 0.5, 1.0, glow_intensity);

    // Combine the distorted image with the glow
    return mix(color, glow_color, glow_intensity);
}