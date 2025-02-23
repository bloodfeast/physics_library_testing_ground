mod actors;

use bevy::prelude::*;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_systems(Startup, (actors::particles::setup, actors::player::setup_player))
        .add_systems(PostStartup, actors::particles::spawn_particles)
        .add_systems(Update,(
            actors::player::player_input,
            actors::player::player_movement,
            actors::player::camera_movement,
            actors::particles::update_simulation
        ))
        .run();
}
