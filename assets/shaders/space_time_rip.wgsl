#import bevy_sprite::mesh2d_vertex_output::VertexOutput

struct SpaceTimeRipProperties {
    start_point: vec2<f32>,
    end_point: vec2<f32>,
    width: f32,
    glow_intensity: f32,
    distortion_strength: f32,
    time: f32,
    glow_color: vec4<f32>,
    animation_speed: f32,
}

struct SpaceTimeRipMaterial {
    properties: SpaceTimeRipProperties,
}
@group(1) @binding(0)
var<storage, read> material: SpaceTimeRipMaterial;
@group(2) @binding(0)
var<uniform> properties: SpaceTimeRipProperties;

fn hash21(p: vec2<f32>) -> f32 {
    var p3 = fract(vec3<f32>(p.x, p.y, p.x) * vec3<f32>(443.897, 441.423, 437.195));
    p3 = p3 + dot(p3, p3.yzx + 18.19);
    return fract((p3.x + p3.y) * p3.z);
}

fn noise(p: vec2<f32>) -> f32 {
    let i = floor(p);
    let f = fract(p);

    // Improved smoothing
    let u = f * f * (3.0 - 2.0 * f);

    let a = hash21(i);
    let b = hash21(i + vec2<f32>(1.0, 0.0));
    let c = hash21(i + vec2<f32>(0.0, 1.0));
    let d = hash21(i + vec2<f32>(1.0, 1.0));

    return mix(mix(a, b, u.x), mix(c, d, u.x), u.y);
}

fn fractal_noise(p: vec2<f32>, octaves: i32) -> f32 {
    var value = 0.0;
    var amplitude = 0.5;
    var frequency = 1.0;

    for (var i = 0; i < octaves; i++) {
        value += amplitude * noise(p * frequency);
        amplitude *= 0.6;
        frequency *= 2.0;
    }

    return value;
}

// Improved jagged tear function
fn calculate_jagged_tear(uv: vec2<f32>, time: f32) -> f32 {
    // Get distance to central horizontal line
    let vertical_dist = abs(uv.y - 0.5);

    // Calculate normalized position along the horizontal line (0 to 1)
    let t = clamp((uv.x - properties.start_point.x) / (properties.end_point.x - properties.start_point.x), -0.5, 1.5);

    // Base width profile - thicker in the middle, thinner at edges
    let width_profile = sin(t * 3.14159) * 1.2 + 0.2;

    // Create jagged edges with various noise frequencies
    let detail_noise = vec2<f32>(
        time * 0.5,         // Slow time component
        time * 1.2          // Faster time component
    );

    // Small high-frequency jagged edges
    let micro_jagged = fractal_noise(vec2<f32>(t * 60.0 + detail_noise.x, vertical_dist * 80.0), 3) * 0.3;

    // Medium frequency jaggedess
    let med_jagged = fractal_noise(vec2<f32>(t * 30.0 + detail_noise.y, vertical_dist * 40.0), 2) * 0.5;

    // Larger distinct tears/spikes at intervals
    let spike_freq = 36.0;
    let spike_phase = t * spike_freq + time * 0.8;
    let spike_amt = pow(abs(sin(spike_phase)), 16.0) * 0.4; // Creates sharp distinct spikes

    // Width modulation along the tear
    let width_mod = properties.width * 0.004 * width_profile;

    // Core vertical distance from central line
    let thickness = width_mod * (1.0 + micro_jagged + med_jagged + spike_amt);

    // Create the tear with edge tapering
    let taper = smoothstep(-0.2, 0.2, t) * smoothstep(1.2, 0.8, t);
    return vertical_dist - thickness * taper;
}

@fragment
fn fragment(in: VertexOutput) -> @location(0) vec4<f32> {
    // Get UV coordinates
    let uv = in.uv;

    // Animation time
    let time = properties.time * properties.animation_speed;

    // Calculate tear shape
    let tear_dist = calculate_jagged_tear(uv, time);

    // Core tear parameters
    let tear_width = properties.width * 0.006;

    // Core dark void with proper tapering
    let void_mask = smoothstep(-0.001, 0.0001, tear_dist) * 0.95;

    // Electric edge glow calculation
    let glow_gradient = exp(-max(0.0, tear_dist * tear_dist) / (tear_width * tear_width * 2.0));

    // Parameter along the tear (0 to 1)
    let t = clamp((uv.x - properties.start_point.x) / (properties.end_point.x - properties.start_point.x), 0.0, 1.0);

    // Electric effect calculation
    let electric_scale = 40.0;
    let electric_speed = time * 4.0;
    let electric_pos = vec2<f32>(t * electric_scale + electric_speed, uv.y * electric_scale - electric_speed * 0.35);
    let electric_noise = fractal_noise(electric_pos, 3);

    // Apply electric effect only near the tear
    let electric_mask = smoothstep(tear_width * 4.0, 0.0, abs(tear_dist));
    let electric_effect = electric_noise * electric_mask * 0.4;

    // Streaks emanating from the tear
    let streak_dir = normalize(vec2<f32>(1.0, 0.0)); // Horizontal tear
    let perp_dir = vec2<f32>(0.0, 1.0);              // Perpendicular direction

    // Create vertical energy streaks
    let streak_scale = 40.0;
    let streak_phase = uv.y * streak_scale - time * 3.0;
    let streak_noise = fractal_noise(vec2<f32>(t * 5.0, time * 0.5),4);

    // Make streaks sharper and more electric-like
    let streaks = pow(abs(sin(streak_phase + streak_noise * 3.0)), 20.0) *
                 smoothstep(tear_width * 5.0, 0.0, abs(tear_dist)) * 0.7;

    // Calculate occasional flares/pulses along the tear
    let flare_time = floor(time * 0.7);
    let flare_strength = hash21(vec2<f32>(flare_time, flare_time * 0.3)) * 0.7 + 0.3;
    let flare_pos = hash21(vec2<f32>(flare_time * 0.5, flare_time * 0.7));
    let flare_t = mix(0.1, 0.9, flare_pos); // Position along the tear

    // Position the flare somewhere along the tear
    let flare_center = vec2<f32>(
        mix(properties.start_point.x, properties.end_point.x, flare_t),
        0.5
    );

    // Calculate flare intensity
    let flare_dist = distance(uv, flare_center);
    let flare = flare_strength * exp(-flare_dist * flare_dist * 20.0);

    // Combine all lighting effects
    let combined_glow = glow_gradient + electric_effect + streaks + flare;

    // Add pulsing effect
    let pulse = (sin(time * 1.6) * 0.15 + 0.85) * properties.glow_intensity;

    // Calculate color based on properties and add subtle variations
    let time_variation = vec4<f32>(
        sin(time * 0.2) * 0.1 + 0.9,
        cos(time * 0.3) * 0.1 + 0.9,
        sin(time * 0.4) * 0.1 + 0.9,
        1.0
    );

    // Energy effect color
    let energy_color = properties.glow_color * time_variation * pulse;

    // Final color calculation with dark void in center
    let combined_color = energy_color * combined_glow;
    let final_color = mix(combined_color, vec4<f32>(0.0, 0.0, 0.0, 1.0), void_mask);

    // Calculate alpha with distance-based falloff
    let falloff_dist = 8.0;
    let alpha_mask = smoothstep(tear_width * falloff_dist, 0.0, abs(tear_dist));

    // Apply void transparency carefully
    let final_alpha = alpha_mask * (1.0 - void_mask * 0.98);

    return vec4<f32>(final_color.rgb, final_alpha);
}