use bevy::ecs::system::SystemId;
use bevy::prelude::*;

#[derive(PartialEq, Clone, Debug)]
pub enum GameMode {
    Menu,
    GameRunning,
    GameOver,
}

#[derive(Resource)]
pub struct MainGameState {
    pub player_hp: f32,
    pub player_energy: f32,
    pub player_shield: f32,
    pub score: i32,
    pub mode: GameMode,
}

pub fn setup_game_state(mut commands: Commands) {
    commands.insert_resource(MainGameState {
        player_hp: 100.0,
        player_energy: 100.0,
        player_shield: 100.0,
        score: 0,
        mode: GameMode::GameRunning,
    });
}

pub fn refresh_player_energy(
    mut state: ResMut<MainGameState>,
) {
        if state.player_energy < 100.0 {
            state.player_energy += 0.15;
        };
}