use nalgebra::{Point3, Vector3};
use rand::{Rng, rng};

use crate::{point::Point, screen::Screen};

/// Defines the direction the camera can move in
pub enum Direction {
    Forward,
    Backward,
    Right,
    Left,
    Up,
    Down,
}

/// A camera reference frame with position, direction, up and right vectors
pub struct CameraReferenceFrame {
    /// The position of the camera
    pub position: Point3<f32>,
    /// The target point the camera is looking at
    pub target: Point3<f32>,
    /// The up vector of the camera
    pub up: Vector3<f32>,
    /// The right vector of the camera
    pub right: Vector3<f32>,
}

impl CameraReferenceFrame {
    /// Set the position of the camera
    pub fn with_position(mut self, position: Point3<f32>) -> Self {
        self.position = position;
        self
    }

    /// Set the direction the camera is looking at
    pub fn with_up(mut self, up: Vector3<f32>) -> Self {
        self.up = up.normalize();
        self
    }

    /// Set the target point the camera is looking at
    pub fn with_target(mut self, target: Point3<f32>) -> Self {
        self.target = target;
        self
    }

    /// Look at a target point from the camera position
    pub fn look_at(&mut self, target: Point3<f32>) {
        self.target = target;
        self.right = self.look_direction().cross(&self.up).normalize();
    }

    pub fn update_position_x(&mut self, x: f32) {
        self.position.x = x;
        self.look_at(self.target);
    }

    pub fn update_position_y(&mut self, y: f32) {
        self.position.y = y;
        self.look_at(self.target);
    }

    pub fn update_position_z(&mut self, z: f32) {
        self.position.z = z;
        self.look_at(self.target);
    }

    /// Compute the direction the camera is looking at
    pub fn look_direction(&self) -> Vector3<f32> {
        (self.target - self.position).normalize()
    }

    /// Move the camera position in a given direction
    pub fn move_position(&mut self, distance: f32, direction: Direction) {
        self.position += match direction {
            Direction::Forward => self.look_direction() * distance,
            Direction::Backward => -self.look_direction() * distance,
            Direction::Right => self.right * distance,
            Direction::Left => -self.right * distance,
            Direction::Up => self.up * distance,
            Direction::Down => -self.up * distance,
        };
        self.look_at(self.target);
    }

    /// Move the camera target in a given direction
    pub fn move_target(&mut self, distance: f32, direction: Direction) {
        self.target += match direction {
            Direction::Forward => Vector3::z() * distance,
            Direction::Backward => -Vector3::z() * distance,
            Direction::Right => Vector3::x() * distance,
            Direction::Left => -Vector3::x() * distance,
            Direction::Up => Vector3::y() * distance,
            Direction::Down => -Vector3::y() * distance,
        };
        self.look_at(self.target);
    }
}

impl Default for CameraReferenceFrame {
    fn default() -> Self {
        Self {
            position: Point3::origin() - Vector3::new(0.0, 0.0, 10.0),
            target: Point3::origin(),
            up: Vector3::y(),
            right: Vector3::x(),
        }
    }
}

/// A camera with a reference frame, configuration and screen
pub struct Camera {
    /// The reference frame of the camera
    pub reference_frame: CameraReferenceFrame,
    /// The horizontal field of view in degrees
    fov: f32,
    /// The horizontal field of view in radians
    fov_rad: f32,
    /// The aspect ratio of the screen
    pub aspect_ratio: f32,
    /// The distance from the camera to the screen
    pub screen_distance: f32,
    /// The width of the screen
    pub screen_width: f32,
    /// The height of the screen
    pub screen_height: f32,
    /// The screen of the camera
    pub screen: Screen,
}

impl Camera {
    /// Create a new camera with a reference frame and configuration
    pub fn new(
        reference_frame: CameraReferenceFrame,
        fov: f32,
        screen_distance: f32,
        screen_resolution: (usize, usize),
    ) -> Self {
        let screen = Screen::new(screen_resolution);
        let aspect_ratio = screen_resolution.0 as f32 / screen_resolution.1 as f32;
        let mut camera = Self {
            reference_frame,
            fov: 0.0,
            fov_rad: 0.0,
            aspect_ratio,
            screen_distance,
            screen_width: 0.0,
            screen_height: 0.0,
            screen,
        };
        camera.update_fov(fov);
        camera
    }

    /// Recalculate screen dimensions based on the latest FOV and screen distance
    ///
    /// The fov is in degrees
    pub fn update_fov(&mut self, fov: f32) {
        self.fov = fov;
        self.fov_rad = fov.to_radians();
        self.screen_width = 2.0 * (self.fov_rad / 2.0).tan() * self.screen_distance;
        self.screen_height = self.screen_width / self.aspect_ratio;
    }

    /// Return the field of view of the camera in degrees
    pub fn fov(&self) -> f32 {
        self.fov
    }

    /// Given a point cloud, choose the proper camera position and direction to fit all points
    pub fn fit_points(&mut self, points: &[Point]) {
        let (min, max) = Point::bounding_box(points);

        // Compute the width, height and depth of the bounding box
        let width = max.x - min.x;
        let height = max.y - min.y;
        let depth = max.z - min.z;

        // Compute the center of the bounding box
        let center = min + Vector3::new(width / 2.0, height / 2.0, depth / 2.0);

        // Compute the distance required to fit the entire point cloud in x and y
        let angle_x = self.fov_rad / 2.0;
        let distance_x = width / angle_x.tan();
        let angle_y = angle_x / self.aspect_ratio;
        let distance_y = height / angle_y.tan();

        // Choose the maximum distance to fit the entire point cloud
        let distance = distance_x.max(distance_y);

        // Compute the camera position
        let position = center - Vector3::z() * distance;

        self.reference_frame.position = position;
        self.reference_frame.look_at(center);
    }

    /// Intersect a point in 3D space with the screen and return the pixel coordinates
    #[inline]
    pub fn intersect_screen(&self, point: &Point) -> Option<(f32, (usize, usize))> {
        // Direction from camera to the point in world space
        let ray_direction = (point.position - self.reference_frame.position).normalize();
        let look_direction = self.reference_frame.look_direction();

        // Make sure the point is in front of the camera
        let alignment = ray_direction.dot(&look_direction);
        if alignment <= 0.0 {
            return None;
        }

        // Compute the distance from the camera to the screen along the camera's view direction
        let distance = self.screen_distance / alignment;

        // Compute the intersection point in world space
        let intersection = self.reference_frame.position + ray_direction * distance;

        // Convert the intersection point to screen space coordinates (2D)
        let screen_origin = self.reference_frame.position + look_direction * self.screen_distance;

        // Calculate relative coordinates on the screen
        let relative_position = intersection - screen_origin;
        let normalized_x =
            (relative_position.dot(&self.reference_frame.right) * 2.0) / self.screen_width;
        let normalized_y =
            (relative_position.dot(&self.reference_frame.up) * 2.0) / self.screen_height;

        // Convert the normalized device coordinates to pixel coordinates
        self.screen
            .to_pixel_coords(normalized_x, normalized_y)
            .map(|position| (distance, position))
    }

    /// Intersect a point in 3D space with the screen using depth of field
    pub fn intersect_screen_dof(
        &self,
        point: &Point,
        aperture_size: f32,
        samples: usize,
    ) -> Option<(f32, (usize, usize))> {
        let mut rng = rng();
        let mut pixel_sum = (0.0, 0.0);
        let mut valid_samples = 0;
        let mut distance_sum = 0.0;

        for _ in 0..samples {
            // Generate random offsets within the aperture size
            let jitter_x: f32 = rng.random_range(-aperture_size..aperture_size);
            let jitter_y: f32 = rng.random_range(-aperture_size..aperture_size);

            // Jittered camera position
            let jittered_position = self.reference_frame.position
                + self.reference_frame.right * jitter_x
                + self.reference_frame.up * jitter_y;

            // Compute the ray direction from the jittered camera position
            let ray_direction = (point.position - jittered_position).normalize();
            let look_direction = self.reference_frame.look_direction();

            // Ensure the point is in front of the camera
            let alignment = ray_direction.dot(&look_direction);
            if alignment <= 0.0 {
                continue;
            }

            // Compute distance to screen
            let distance = self.screen_distance / alignment;
            let intersection = jittered_position + ray_direction * distance;

            // Convert to screen-space coordinates
            let screen_origin = jittered_position + look_direction * self.screen_distance;
            let relative_position = intersection - screen_origin;

            let normalized_x =
                relative_position.dot(&self.reference_frame.right) / (self.screen_width / 2.0);
            let normalized_y =
                relative_position.dot(&self.reference_frame.up) / (self.screen_height / 2.0);

            // Convert to pixel coordinates
            if let Some((px, py)) = self.screen.to_pixel_coords(normalized_x, normalized_y) {
                pixel_sum.0 += px as f32;
                pixel_sum.1 += py as f32;
                distance_sum += distance;
                valid_samples += 1;
            }
        }

        // Average the results to simulate blur
        if valid_samples > 0 {
            let avg_x = (pixel_sum.0 / valid_samples as f32).round() as usize;
            let avg_y = (pixel_sum.1 / valid_samples as f32).round() as usize;
            let avg_distance = distance_sum / valid_samples as f32;
            Some((avg_distance, (avg_x, avg_y)))
        } else {
            None
        }
    }
}
