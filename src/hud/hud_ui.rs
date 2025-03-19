use bevy::prelude::*;
use bevy::sprite::Anchor;
use crate::actors::player::Player;
use crate::state::MainGameState;

#[derive(Component)]
pub struct HpBar;

#[derive(Component)]
pub struct EnergyBar;

#[derive(Component)]
pub struct ShieldBar;

#[derive(Component)]
pub struct ScoreCounter;

pub fn setup_hud(
    mut commands: Commands,
    window_query: Query<&Window>,
) {
    let window = window_query.get_single().unwrap_or_else(|_| panic!("No window found"));
    let window_half_width = window.width() * 0.5;
    let window_half_height = window.height() * 0.5;

    // Shield Bar
    commands.spawn((
        ShieldBar,
        Sprite {
            color: Color::srgb(0.25, 0.65, 1.0).into(),
            custom_size: Some(Vec2::new(300.0, 10.0)),
            anchor: Anchor::CenterLeft,
            ..Default::default()
        },
        Transform {
            translation: Vec3::new(-window_half_width + 50.0, window_half_height - 50.0, 10.0),
            ..Default::default()
        },
    ));

    // HP Bar
    commands.spawn((
        HpBar,
        Sprite {
            color: Color::srgb(1.0, 0.0, 0.0).into(),
            custom_size: Some(Vec2::new(300.0, 10.0)),
            anchor: Anchor::CenterLeft,
            ..Default::default()
        },
        Transform {
            translation: Vec3::new(-window_half_width + 50.0, window_half_height - 70.0, 10.0),
            ..Default::default()
        },
    ));

    // Energy Bar
    commands.spawn((
        EnergyBar,
        Sprite {
            color: Color::srgb(0.0, 1.0, 0.0).into(),
            custom_size: Some(Vec2::new(300.0, 10.0)),
            anchor: Anchor::CenterLeft,
            ..Default::default()
        },
        Transform {
            translation: Vec3::new(-window_half_width + 50.0, window_half_height - 90.0,  10.0),
            ..Default::default()
        },
    ));

    commands.spawn((
        ScoreCounter,
        Text2d("Score ".to_string()),
        Transform {
            translation: Vec3::new(window_half_width - 100.0, window_half_height - 50.0,  10.0),
            ..Default::default()
        },
    ));
}

pub fn update_shield(
    mut query: Query<&mut Transform, With<ShieldBar>>,
    game_state: Res<MainGameState>,
) {
    let mut shield_transform = query.get_single_mut().unwrap();
    shield_transform.scale.x = (game_state.player_shield / 100.0).max(0.0);
}

pub fn update_hp(
    mut query: Query<&mut Transform, With<HpBar>>,
    game_state: Res<MainGameState>,
) {
    let mut hp_transform = query.get_single_mut().unwrap();
    hp_transform.scale.x = (game_state.player_hp / 100.0).max(0.0);
}

pub fn update_energy(
    mut query: Query<&mut Transform, With<EnergyBar>>,
    game_state: Res<MainGameState>,
) {
    let mut energy_transform = query.get_single_mut().unwrap();
    energy_transform.scale.x =  (game_state.player_energy / 100.0).max(0.0);
}

pub fn update_score(
    mut query: Query<&mut Text2d, With<ScoreCounter>>,
    game_state: Res<MainGameState>
) {
    let mut score_text = query.get_single_mut().unwrap();
    score_text.0 = format!("Score: {:?}", game_state.score);
}