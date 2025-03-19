mod actors;
mod state;
mod hud;
mod props;
mod window_plugin;

use bevy::prelude::*;
use bevy::render::RenderPlugin;
use bevy::render::settings::{Backends, MemoryHints, RenderCreation, WgpuSettings};
use crate::actors::black_hole::{BlackHolePlugin};
use crate::actors::distortion::{DistortionPostProcessPlugin};
use crate::props::walls::WallsPlugin;
use crate::window_plugin::{CustomWindowPlugin, WindowConfig};

fn main() {
    let mut app = App::new();
    let default_window_config = WindowConfig::default();
    let window_plugin = CustomWindowPlugin::new(default_window_config);

    app.add_plugins(
        DefaultPlugins
            .set(
                RenderPlugin {
                    render_creation: RenderCreation::Automatic(WgpuSettings {
                        backends: Some(Backends::DX12),
                        memory_hints: MemoryHints::Performance,
                        ..default()
                    }),
                    ..default()
                },
            )
            .disable::<WindowPlugin>()
            .add(window_plugin)
    );
    app.add_plugins(BlackHolePlugin);
    app.add_plugins(DistortionPostProcessPlugin);
    app.add_plugins(WallsPlugin);


    app
        .add_systems(PreStartup, (
            hud::setup_hud,
            state::setup_game_state,
            actors::particles::setup,
        ))
        .add_systems(Startup, (
            actors::player::setup_camera,
            actors::player::setup_player,
            actors::particles::spawn_particles,
            props::walls::spawn_space_time_walls,
        ))
        .add_systems(FixedUpdate, (
            actors::enemy::spawn_enemy,
            state::refresh_player_energy,
            state::refresh_player_shield,
        ))
        .add_systems(PreUpdate, actors::player::player_movement_physics)
        .add_systems(Update,(
            actors::enemy::update_enemy,
            actors::player::update_player_movement,
            actors::player::camera_movement,
            actors::particles::update_simulation,
        ))
        .add_systems(PostUpdate, (
            actors::player::player_input,
            hud::update_energy,
            hud::update_hp,
            hud::update_shield,
            hud::update_score,
        ))
        .run();
}
