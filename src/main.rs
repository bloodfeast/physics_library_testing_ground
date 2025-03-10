mod actors;
mod state;
mod hud;

use bevy::prelude::*;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_systems(PreStartup, (hud::setup_hud, state::setup_game_state, actors::particles::setup))
        .add_systems(Startup, actors::player::setup_player)
        .add_systems(PostStartup, actors::particles::spawn_particles)
        .add_systems(FixedUpdate, (
            actors::enemy::spawn_enemy,
            actors::particles::update_forces,
            state::refresh_player_energy,
            state::refresh_player_shield,
        ))
        .add_systems(PreUpdate, actors::player::player_movement_physics)
        .add_systems(Update,(
            actors::enemy::update_enemy,
            actors::player::player_input,
            actors::particles::update_simulation,
            actors::player::camera_movement,
        ))
        .add_systems(PostUpdate, (actors::player::update_player_movement, hud::update_energy, hud::update_hp, hud::update_shield))
        .run();
}
