// Minimal working shader for Bevy 0.15 Material2d
#import bevy_sprite::mesh2d_types
#import bevy_sprite::mesh2d_view_types

@group(1) @binding(0)
var<uniform> glow_color: vec4<f32>;

struct FragmentInput {
    @builtin(position) position: vec4<f32>,
    @location(0) world_position: vec4<f32>,
    @location(1) uv: vec2<f32>,
}

@fragment
fn fragment(
    @builtin(position) position: vec4<f32>,
    #import bevy_sprite::mesh2d_vertex_output::VertexOutput
) -> @location(0) vec4<f32> {
    return glow_color;
}