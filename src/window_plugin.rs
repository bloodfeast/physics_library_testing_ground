use bevy::prelude::*;
use bevy::window::{PresentMode, Window, WindowMode, WindowPlugin, WindowResolution};

/// Configuration struct for window settings
#[derive(Clone)]
pub struct WindowConfig {
    pub width: f32,
    pub height: f32,
    pub title: String,
    pub resizable: bool,
    pub decorations: bool,
    pub mode: WindowMode,
    pub position: WindowPosition,
    pub present_mode: PresentMode,
}

impl Default for WindowConfig {
    fn default() -> Self {
        WindowConfig {
            width: 1920.0,
            height: 1080.0,
            title: "I Am Black Hole".to_string(),
            resizable: true,
            decorations: true,
            mode: WindowMode::Windowed,
            position: WindowPosition::Automatic,
            present_mode: PresentMode::AutoNoVsync,
        }
    }
}

pub struct CustomWindowPlugin {
    config: WindowConfig,
}

impl CustomWindowPlugin {
    pub fn new(config: WindowConfig) -> Self {
        CustomWindowPlugin { config }
    }
}

impl Plugin for CustomWindowPlugin {
    fn build(&self, app: &mut App) {

        // Add our custom window plugin with configured window
        app.add_plugins(WindowPlugin {
            primary_window: Some(Window {
                resolution: WindowResolution::new(self.config.width, self.config.height),
                title: self.config.title.clone(),
                resizable: self.config.resizable,
                decorations: self.config.decorations,
                mode: self.config.mode,
                position: self.config.position,
                present_mode: self.config.present_mode,
                ..default()
            }),
            ..default()
        });
    }
}
