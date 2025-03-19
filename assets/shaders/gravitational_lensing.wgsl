#import bevy_sprite::mesh2d_vertex_output::VertexOutput

@group(2) @binding(0)
var texture: texture_2d<f32>;
@group(2) @binding(1)
var texture_sampler: sampler;

struct LensingProperties {
    center: vec2<f32>,
    strength: f32,
    rotation_speed: f32,
    time: f32,
    radius: f32,
}

@group(1) @binding(0)
var<uniform> properties: LensingProperties;

@fragment
fn fragment(in: VertexOutput) -> @location(0) vec4<f32> {
    // Original UV coordinates
    let uv = in.uv;
    let center = properties.center;

    // Direction from center
    let dir = uv - center;
    let dist = length(dir);

    // Calculate distortion factor based on distance
    // Inverse square falloff (stronger closer to center)
    let distortion_factor = properties.strength / max(dist * dist, 0.001);

    // Add rotation based on distance and time
    let rotation_angle = properties.time * properties.rotation_speed * (1.0 - smoothstep(0.0, properties.radius * 2.0, dist));
    let s = sin(rotation_angle);
    let c = cos(rotation_angle);

    // Apply rotation to the direction vector
    let rotated_dir = vec2<f32>(
        dir.x * c - dir.y * s,
        dir.x * s + dir.y * c
    );

    // Apply the lens distortion (pulling towards center)
    let distorted_uv = uv - normalize(dir) * distortion_factor * smoothstep(properties.radius * 2.0, 0.0, dist);

    // Sample the texture with distorted coordinates
    let color = textureSample(texture, texture_sampler, distorted_uv);

    // Fade out the effect at the edges
    let edge_blend = smoothstep(properties.radius * 3.0, properties.radius, dist);

    // Return the distorted color
    return mix(textureSample(texture, texture_sampler, uv), color, edge_blend);
}