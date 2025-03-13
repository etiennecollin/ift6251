use nannou::prelude::*;

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
}

impl Camera {
    const MAX_PITCH: f32 = std::f32::consts::PI * 0.5 - 0.0001;
    const MIN_PITCH: f32 = -Self::MAX_PITCH;

    /// Creates a new camera at the given position.
    pub fn new(eye: Point3) -> Self {
        Self {
            position: eye,
            pitch: 0.0,
            yaw: std::f32::consts::PI * 0.5,
        }
    }

    /// Calculates the direction vector from the pitch and yaw.
    pub fn direction(&self) -> Vec3 {
        Self::pitch_yaw_to_direction(self.pitch, self.yaw)
    }

    /// The camera's "view" matrix.
    pub fn view(&self) -> Mat4 {
        let direction = self.direction();
        let up = Vec3::Y;
        Mat4::look_to_rh(self.position, direction, up)
    }

    /// Converts a pitch and yaw to a direction vector.
    ///
    /// The pitch and yaw are in radians.
    pub fn pitch_yaw_to_direction(pitch: f32, yaw: f32) -> Vec3 {
        let xz_unit_len = pitch.cos();
        let x = xz_unit_len * yaw.cos();
        let y = pitch.sin();
        let z = xz_unit_len * (-yaw).sin();
        vec3(x, y, z)
    }

    /// Increment the pitch and yaw of the camera by a given delta.
    ///
    /// The pitch is clamped to prevent the camera from flipping.
    pub fn update_pitch(&mut self, pitch_delta: f32) {
        self.pitch = (self.pitch + pitch_delta).clamp(Self::MIN_PITCH, Self::MAX_PITCH);
    }

    /// Increment the yaw of the camera by a given delta.
    ///
    /// The yaw wraps around when it reaches 2*PI.
    pub fn update_yaw(&mut self, yaw_delta: f32) {
        self.yaw = (self.yaw + yaw_delta) % (std::f32::consts::PI * 2.0);
    }

    /// Move the camera in the given direction by the given amount.
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
}

/// Contains the various transformations of a camera.
#[repr(C)]
#[derive(Copy, Clone, Debug)]
pub struct Uniforms {
    pub world: Mat4,
    pub view: Mat4,
    pub proj: Mat4,
}

impl Uniforms {
    /// Returns the uniforms as a byte slice.
    pub fn as_bytes(&self) -> &[u8] {
        unsafe { wgpu::bytes::from(self) }
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

    /// The projection matrix for the camera.
    pub fn projection(&self) -> Mat4 {
        Mat4::perspective_rh_gl(self.fov_y, self.aspect_ratio, self.near, self.far)
    }

    /// The uniforms for the camera.
    pub fn uniform(&self, view: Mat4) -> Uniforms {
        let proj = self.projection();
        let scale = Mat4::from_scale(Vec3::splat(0.01));

        Uniforms {
            world: self.rotation,
            view: (view * scale).into(),
            proj: proj.into(),
        }
    }
}

impl Default for CameraConfig {
    fn default() -> Self {
        Self::new((800, 600), 120.0, (0.01, 100.0))
    }
}
