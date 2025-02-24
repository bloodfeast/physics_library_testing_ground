mod actors;

use bevy::prelude::*;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_systems(Startup, (actors::player::setup_player, actors::particles::setup))
        .add_systems(PostStartup, actors::particles::spawn_particles)
        .add_systems(FixedUpdate, actors::particles::update_forces)
        .add_systems(PostUpdate, actors::player::player_movement_physics)
        .add_systems(Update,(
            actors::player::player_input,
            actors::player::update_player_movement,
            actors::particles::update_simulation,
            actors::player::camera_movement,
        ))
        .run();
}
