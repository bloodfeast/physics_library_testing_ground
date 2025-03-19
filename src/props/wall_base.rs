use bevy::prelude::*;
use rs_physics::models::ObjectIn2D;
use rs_physics::utils::PhysicsConstants;

pub enum WallShape {
    Rigid,
    Convex(f32),
    Concave(f32),
    SpaceTimeRip,
}

pub enum WallInteractionError {
    CalculationError(String)
}

pub trait WallInteractions {
    /// Abstraction layer over [elastic_collision_2d](rs_physics::interactions::interactions_2d::elastic_collision_2d)
    /// To make it more convenient, I decided to accept f32 values and handle the f64 casts internally
    /// Additionally, a [ObjectIn2D](rs_physics::models::object_2d::ObjectIn2D) will be created based on the `Wall` component as an immovable object
    /// and used as the `obj2` parameter
    fn calculate_collision(
        &self,
        constants: &PhysicsConstants,
        obj1: &mut ObjectIn2D,
        angle: f32,
        duration: f64,
        drag_coefficient: f32,
        cross_sectional_area: f32
    ) -> Result<(), &'static str>;

    /// Helper to find the collision angle of some point on a wall's surface
    fn calculate_wall_face_angle_by_position(
        &self,
        position_x: f32,
        position_y: f32
    ) -> Result<f32, WallInteractionError>;
}

#[derive(Component)]
pub struct Wall {
    pub center_x: f32,
    pub center_y: f32,
    pub height: f32,
    pub width: f32,
    pub rotation_angle: f32,
    pub wall_shape: WallShape,
}

impl Wall {
    // Helper function to create a space-time rip wall
    pub fn new_space_time_rip(center_x: f32, center_y: f32, width: f32, height: f32, rotation: f32) -> Self {
        Wall {
            center_x,
            center_y,
            height,
            width,
            rotation_angle: rotation,
            wall_shape: WallShape::SpaceTimeRip,
        }
    }

    // Create a standard rigid wall
    pub fn new_rigid(center_x: f32, center_y: f32, width: f32, height: f32, rotation: f32) -> Self {
        Wall {
            center_x,
            center_y,
            height,
            width,
            rotation_angle: rotation,
            wall_shape: WallShape::Rigid,
        }
    }

    // Utility function to get corner positions of this wall
    pub fn get_corners(&self) -> [Vec2; 4] {
        let half_width = self.width / 2.0;
        let half_height = self.height / 2.0;

        let cos_angle = self.rotation_angle.cos();
        let sin_angle = self.rotation_angle.sin();

        // Calculate the four corners
        let top_left = Vec2::new(
            self.center_x - half_width * cos_angle + half_height * sin_angle,
            self.center_y - half_width * sin_angle - half_height * cos_angle
        );

        let top_right = Vec2::new(
            self.center_x + half_width * cos_angle + half_height * sin_angle,
            self.center_y + half_width * sin_angle - half_height * cos_angle
        );

        let bottom_right = Vec2::new(
            self.center_x + half_width * cos_angle - half_height * sin_angle,
            self.center_y + half_width * sin_angle + half_height * cos_angle
        );

        let bottom_left = Vec2::new(
            self.center_x - half_width * cos_angle - half_height * sin_angle,
            self.center_y - half_width * sin_angle + half_height * cos_angle
        );

        [top_left, top_right, bottom_right, bottom_left]
    }
}