use nannou::{glam::EulerRot, prelude::*};

use crate::point::Point;

/// Defines the direction the camera can move in
pub enum Direction {
    Forward,
    Backward,
    Right,
    Left,
    Up,
    Down,
}

/// A simple first person camera.
pub struct Camera {
    /// The position of the camera.
    pub position: Point3,
    /// Rotation around the x axis in radians.
    pub pitch: f32,
    /// Rotation around the y axis in radians.
    pub yaw: f32,
    /// The configuration for the camera.
    pub config: CameraConfig,
}

impl Camera {
    const MAX_PITCH: f32 = std::f32::consts::PI * 0.5 - 0.0001;
    const MIN_PITCH: f32 = -Self::MAX_PITCH;
    const COORD_SCALE: f32 = 0.01;

    /// Creates a new camera at the given position.
    pub fn new(config: CameraConfig) -> Self {
        Self {
            position: Point3::new(0.0, 0.0, -1.0),
            pitch: 0.0,
            yaw: -std::f32::consts::PI * 0.5,
            config,
        }
    }

    /// Sets the position of the camera.
    pub fn with_position(mut self, position: Point3) -> Self {
        self.position = position;
        self
    }

    /// Calculates the direction vector from the pitch and yaw.
    pub fn direction(&self) -> Vec3 {
        Self::pitch_yaw_to_direction(self.pitch, self.yaw)
    }

    /// Converts a pitch and yaw to a direction vector.
    ///
    /// The pitch and yaw are in radians.
    fn pitch_yaw_to_direction(pitch: f32, yaw: f32) -> Vec3 {
        let xz_unit_len = pitch.cos();
        let x = xz_unit_len * yaw.cos();
        let y = pitch.sin();
        let z = xz_unit_len * (-yaw).sin();
        vec3(x, y, z).normalize()
    }

    /// Given a point cloud, choose the proper camera position and direction to fit all points
    pub fn fit_points(&mut self, points: &[Point]) {
        let (min, max) = Point::bounding_box(points);

        // Compute the center of the bounding box
        let center = (min + max) / 2.0;

        // Compute the radius of the bounding sphere
        let radius = (max - center).length();

        // Compute the distance required to fit the entire bounding sphere
        let angle = self.config.fov_y / 2.0;
        let distance = radius / angle.tan();

        // Move the camera backward along the new forward direction
        self.position = (center - Vec3::Z * distance) * Self::COORD_SCALE;
        // Look at the center first to get the correct forward direction
        self.look_at(center);
    }

    /// Sets the pitch and yaw of the camera to look at a target.
    pub fn look_at(&mut self, target: Point3) {
        let current_direction = self.direction();
        let target_direction = (target - self.position).normalize();

        let rotation = Quat::from_rotation_arc(current_direction, target_direction);

        // Extract pitch and yaw from the quaternion
        let (yaw, pitch, _) = rotation.to_euler(EulerRot::YXZ);
        self.yaw += yaw;
        self.pitch += pitch;
    }

    /// Increments the pitch and yaw of the camera by a given delta.
    ///
    /// The pitch is clamped to prevent the camera from flipping.
    pub fn update_pitch(&mut self, pitch_delta: f32) {
        self.pitch = (self.pitch + pitch_delta).clamp(Self::MIN_PITCH, Self::MAX_PITCH);
    }

    /// Increments the yaw of the camera by a given delta.
    ///
    /// The yaw wraps around when it reaches 2*PI.
    pub fn update_yaw(&mut self, yaw_delta: f32) {
        self.yaw = (self.yaw + yaw_delta) % (std::f32::consts::PI * 2.0);
    }

    /// Sets the position of the camera.
    pub fn set_position(&mut self, position: Point3) {
        self.position = position;
    }

    /// Moves the camera in the given direction by the given amount.
    pub fn move_towards(&mut self, direction: Direction, amount: f32) {
        let direction = match direction {
            Direction::Forward => self.direction(),
            Direction::Backward => -self.direction(),
            Direction::Left => {
                let pitch = 0.0;
                let yaw = self.yaw + std::f32::consts::PI * 0.5;
                Camera::pitch_yaw_to_direction(pitch, yaw)
            }
            Direction::Right => {
                let pitch = 0.0;
                let yaw = self.yaw - std::f32::consts::PI * 0.5;
                Camera::pitch_yaw_to_direction(pitch, yaw)
            }
            Direction::Down => {
                let pitch = self.pitch - std::f32::consts::PI * 0.5;
                Camera::pitch_yaw_to_direction(pitch, self.yaw)
            }
            Direction::Up => {
                let pitch = self.pitch + std::f32::consts::PI * 0.5;
                Camera::pitch_yaw_to_direction(pitch, self.yaw)
            }
        };
        self.position += direction * amount;
    }

    /// The projection matrix for the camera.
    pub fn projection(&self) -> Mat4 {
        Mat4::perspective_rh_gl(
            self.config.fov_y,
            self.config.aspect_ratio,
            self.config.near,
            self.config.far,
        )
    }

    /// The camera's "view" matrix.
    pub fn view(&self) -> Mat4 {
        let direction = self.direction();
        let up = Vec3::Y;
        Mat4::look_to_rh(self.position, direction, up)
    }

    /// The uniforms for the camera.
    pub fn uniforms(&self) -> CameraTransforms {
        let scale = Mat4::from_scale(Vec3::splat(Self::COORD_SCALE));

        CameraTransforms {
            world: self.config.rotation,
            view: (self.view() * scale),
            proj: self.projection(),
        }
    }
}

/// The configuration for a camera.
pub struct CameraConfig {
    rotation: Mat4,
    aspect_ratio: f32,
    fov_y: f32,
    near: f32,
    far: f32,
}

impl CameraConfig {
    /// Creates a new camera configuration.
    ///
    /// The fov_y is in degrees.
    pub fn new((width, height): (u32, u32), fov_y: f32, (near, far): (f32, f32)) -> Self {
        Self {
            rotation: Mat4::from_rotation_y(0f32),
            aspect_ratio: width as f32 / height as f32,
            fov_y: fov_y.to_radians(),
            near,
            far,
        }
    }

    /// Sets the angle of rotation around the y axis.
    ///
    /// The angle is in degrees.
    pub fn with_rotation(mut self, angle: f32) -> Self {
        self.rotation = Mat4::from_rotation_y(angle.to_radians());
        self
    }

    /// Sets the aspect ratio of the camera.
    pub fn with_aspect_ratio(mut self, width: u32, height: u32) -> Self {
        self.aspect_ratio = width as f32 / height as f32;
        self
    }

    /// Sets the z-near and z-far of the camera.
    pub fn with_range(mut self, near: f32, far: f32) -> Self {
        self.near = near;
        self.far = far;
        self
    }
}

impl Default for CameraConfig {
    fn default() -> Self {
        Self::new((800, 600), 120.0, (0.001, 100.0))
    }
}

/// Contains the various transformations of a camera.
#[repr(C)]
#[derive(Copy, Clone, Debug)]
pub struct CameraTransforms {
    pub world: Mat4,
    pub view: Mat4,
    pub proj: Mat4,
}

impl CameraTransforms {
    /// Returns the struct as a byte slice.
    pub fn as_bytes(&self) -> &[u8] {
        unsafe { wgpu::bytes::from(self) }
    }
}
