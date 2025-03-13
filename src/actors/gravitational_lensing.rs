use bevy::asset::RenderAssetUsages;
use bevy::prelude::*;
use bevy::render::camera::RenderTarget;
use bevy::render::render_resource::{AsBindGroup, Extent3d, ShaderRef, ShaderType, TextureDimension, TextureFormat, TextureUsages};
use bevy::render::view::RenderLayers;
use bevy::sprite::{Material2d, Material2dPlugin, MaterialMesh2dBundle};
use crate::actors::player::Player;

// Component to link the lensing effect to the player
#[derive(Component)]
pub struct LensingEffect {
    pub material_handle: Handle<GravitationalLensingMaterial>,
}

// Resource to store the render target handle
#[derive(Resource)]
pub struct LensingRenderTarget(pub Handle<Image>);

#[derive(Asset, AsBindGroup, Debug, Clone, TypePath)]
pub struct GravitationalLensingMaterial {
    #[uniform(0)]
    pub properties: LensingProperties,

    #[texture(1)]
    #[sampler(2)]
    pub source_image: Handle<Image>,
}

#[derive(Clone, Debug, ShaderType)]
pub struct LensingProperties {
    pub center: Vec2,
    pub strength: f32,
    pub rotation_speed: f32,
    pub time: f32,
    pub radius: f32,
}

impl Material2d for GravitationalLensingMaterial {
    fn fragment_shader() -> ShaderRef {
        "shaders/gravitational_lensing.wgsl".into()
    }
}

pub fn setup_lensing_effect(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<GravitationalLensingMaterial>>,
    mut images: ResMut<Assets<Image>>,
    player_query: Query<Entity, With<Player>>,
) {
    println!("Setting up lensing effect");
    // Create a render target
    let size = Extent3d {
        width: 800,
        height: 600,
        depth_or_array_layers: 1,
    }; // Adjust to your window size
    let mut render_target = Image::new_fill(
        size,
        TextureDimension::D2,
        &[0, 0, 0, 0],
        TextureFormat::Rgba8UnormSrgb,
        RenderAssetUsages::RENDER_WORLD,
    );
    render_target.texture_descriptor.usage =
        TextureUsages::TEXTURE_BINDING | TextureUsages::RENDER_ATTACHMENT;
    let render_target_handle = images.add(render_target);

    // Create the distortion material
    let material_handle = materials.add(GravitationalLensingMaterial {
        properties: LensingProperties {
            center: Vec2::new(0.5, 0.5),
            strength: 0.5,            // Start with subtle distortion
            rotation_speed: 0.2,       // Slow rotation
            time: 0.0,
            radius: 0.15,              // Matches your black hole radius
        },
        source_image: render_target_handle.clone(),
    });

    // Add to player
    if let Ok(player_entity) = player_query.get_single() {
        commands.entity(player_entity).insert(LensingEffect {
            material_handle: material_handle.clone(),
        });
        println!("Lensing effect added to player");
    }

    commands.spawn(MaterialMesh2dBundle {
        mesh: Mesh2d(meshes.add(Rectangle::new(
            size.width as f32,
            size.height as f32,
        ))),
        material: MeshMaterial2d(material_handle.clone()),
        transform: Transform::from_xyz(0.0, 0.0, 2.0),
        ..Default::default()
    });

    // Setup the camera with the render target
    commands.insert_resource(LensingRenderTarget(render_target_handle));
    println!("Lensing effect setup complete");
}

pub fn update_lensing_effect(
    mut materials: ResMut<Assets<GravitationalLensingMaterial>>,
    time: Res<Time>,
    player_query: Query<(&Transform, &LensingEffect)>,
) {
    for (transform, effect) in player_query.iter() {
        if let Some(material) = materials.get_mut(&effect.material_handle) {
            // Update the time value for animation
            material.properties.time += time.delta_secs();

            // Update center position based on player's position
            // Convert world position to UV coordinates (0-1 range)
            // This requires knowing your viewport dimensions
            let viewport_size = Vec2::new(800., 600.0); // Adjust to your window size
            material.properties.center = Vec2::new(
                (transform.translation.x + viewport_size.x * 0.5) / viewport_size.x,
                (transform.translation.y + viewport_size.y * 0.5) / viewport_size.y,
            );
        }
    }
}

pub struct LensingPlugin;

impl Plugin for LensingPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_plugins(Material2dPlugin::<GravitationalLensingMaterial>::default())
            .add_systems(PostStartup, setup_lensing_effect)
            .add_systems(Update, update_lensing_effect);
    }
}

