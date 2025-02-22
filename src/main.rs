mod actors;

use bevy::prelude::*;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_systems(Startup, (actors::player::setup_player))
        .add_systems(Update,(actors::player::player_input, actors::player::player_movement))
        .run();
}
