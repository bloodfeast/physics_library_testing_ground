use bevy::prelude::*;
use crate::props::wall_base::Wall;
use crate::actors::space_time_rip::SpaceTimeRipPlugin;

// Example plugin that adds the space-time rip walls to your game
pub struct WallsPlugin;

impl Plugin for WallsPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(SpaceTimeRipPlugin)
            .add_systems(PreStartup, spawn_space_time_walls);
    }
}

// System to spawn walls with space-time rips
pub fn spawn_space_time_walls(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    println!("Spawning space-time rip walls...");

    // Create a few example walls

    // Vertical wall with space-time rip
    spawn_wall(
        &mut commands,
        &mut meshes,
        &mut materials,
        Vec3::new(-500.0, 0.0, 0.0),       // Position
        Vec2::new(300.0, 10.0),           // Size (width, height)
        std::f32::consts::FRAC_PI_2 + std::f32::consts::FRAC_PI_8,                              // Rotation (radians)
        Color::srgba(0.3, 0.3, 0.35, 0.0),       // Color
        true                              // Has space-time rip
    );

    // Vertical wall with space-time rip
    spawn_wall(
        &mut commands,
        &mut meshes,
        &mut materials,
        Vec3::new(50.0, 350.0, 0.0),       // Position
        Vec2::new(400.0, 10.0),           // Size (width, height)
        std::f32::consts::FRAC_PI_2 + std::f32::consts::FRAC_PI_8,                              // Rotation (radians)
        Color::srgba(0.3, 0.3, 0.35, 0.0),       // Color
        true                              // Has space-time rip
    );

    // Horizontal wall with space-time rip
    spawn_wall(
        &mut commands,
        &mut meshes,
        &mut materials,
        Vec3::new(-450.0, 250.0, 0.0),     // Position
        Vec2::new(300.0, 10.0),           // Size
        0.0,                              // Rotation
        Color::srgba(0.3, 0.3, 0.35, 0.0),         // Color
        true                              // Has space-time rip
    );

    // Horizontal wall with space-time rip
    spawn_wall(
        &mut commands,
        &mut meshes,
        &mut materials,
        Vec3::new(150.0, 250.0, 0.0),     // Position
        Vec2::new(300.0, 10.0),           // Size
        0.0,                              // Rotation
        Color::srgba(0.3, 0.3, 0.35, 0.0),         // Color
        true                              // Has space-time rip
    );

    // Angled wall with space-time rip
    spawn_wall(
        &mut commands,
        &mut meshes,
        &mut materials,
        Vec3::new(200.0, -500.0, 0.0),      // Position
        Vec2::new(400.0, 10.0),           // Size
        std::f32::consts::FRAC_PI_6,      // Rotation (30 degrees)
        Color::srgba(0.3, 0.3, 0.35, 0.0),    // Color
        true                              // Has space-time rip
    );

    // Angled wall with space-time rip
    spawn_wall(
        &mut commands,
        &mut meshes,
        &mut materials,
        Vec3::new(400.0, -300.0, 0.0),      // Position
        Vec2::new(200.0, 10.0),           // Size
        std::f32::consts::FRAC_PI_6 + std::f32::consts::FRAC_PI_8,      // Rotation (30 degrees)
        Color::srgba(0.3, 0.3, 0.35, 0.0),    // Color
        true                              // Has space-time rip
    );

}

// Helper function to spawn a wall entity
fn spawn_wall(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<ColorMaterial>>,
    position: Vec3,
    size: Vec2,
    rotation: f32,
    color: Color,
    has_space_time_rip: bool,
) {
    // Create the wall component
    let wall = if has_space_time_rip {
        Wall::new_space_time_rip(
            position.x,
            position.y,
            size.x,
            size.y,
            rotation
        )
    } else {
        Wall::new_rigid(
            position.x,
            position.y,
            size.x,
            size.y,
            rotation
        )
    };

    // Spawn the wall entity
    commands.spawn((
        wall,
        Mesh2d(meshes.add(Rectangle::new(size.x, size.y))),
        MeshMaterial2d(materials.add(color)),
        Transform {
            translation: position,
            rotation: Quat::from_rotation_z(rotation),
            ..Default::default()
        },
    ));
}