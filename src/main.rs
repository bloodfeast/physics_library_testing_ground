mod actors;
mod state;
mod hud;

use bevy::prelude::*;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_systems(Startup, (actors::player::setup_player, actors::particles::setup, state::setup_game_state))
        .add_systems(PostStartup, (actors::particles::spawn_particles, hud::setup_hud))
        .add_systems(FixedUpdate, (actors::particles::update_forces, state::refresh_player_energy))
        .add_systems(PreUpdate, actors::player::player_movement_physics)
        .add_systems(Update,(
            actors::player::player_input,
            actors::particles::update_simulation,
            actors::player::camera_movement,
        ))
        .add_systems(PostUpdate, (actors::player::update_player_movement, hud::update_energy))
        .run();
}
