use nalgebra::{Point3, Vector3};
use rand::{rng, Rng};

use crate::{point::Point, screen::Screen};

/// A camera reference frame with position, direction, up and right vectors
pub struct CameraReferenceFrame {
    /// The position of the camera
    pub position: Point3<f64>,
    /// The target point the camera is looking at
    pub target: Point3<f64>,
    /// The up vector of the camera
    pub up: Vector3<f64>,
    /// The right vector of the camera
    pub right: Vector3<f64>,
}

impl CameraReferenceFrame {
    /// Set the position of the camera
    pub fn with_position(mut self, position: Point3<f64>) -> Self {
        self.position = position;
        self
    }

    /// Set the direction the camera is looking at
    pub fn with_up(mut self, up: Vector3<f64>) -> Self {
        self.up = up.normalize();
        self
    }

    /// Set the target point the camera is looking at
    pub fn with_target(mut self, target: Point3<f64>) -> Self {
        self.target = target;
        self
    }

    /// Look at a target point from the camera position
    pub fn look_at(&mut self, target: Point3<f64>) {
        self.target = target;
        self.right = self.look_direction().cross(&self.up).normalize();
    }

    pub fn update_position_x(&mut self, x: f64) {
        self.position.x = x;
        self.look_at(self.target);
    }

    pub fn update_position_y(&mut self, y: f64) {
        self.position.y = y;
        self.look_at(self.target);
    }

    pub fn update_position_z(&mut self, z: f64) {
        self.position.z = z;
        self.look_at(self.target);
    }

    /// Compute the direction the camera is looking at
    pub fn look_direction(&self) -> Vector3<f64> {
        (self.target - self.position).normalize()
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
    pub fov: f64,
    /// The aspect ratio of the screen
    pub aspect_ratio: f64,
    /// The distance from the camera to the screen
    pub screen_distance: f64,
    /// The screen of the camera
    pub screen: Screen,
}

impl Camera {
    /// Create a new camera with a reference frame and configuration
    pub fn new(
        reference_frame: CameraReferenceFrame,
        fov: f64,
        screen_distance: f64,
        screen_resolution: (usize, usize),
    ) -> Self {
        let screen = Screen::new(screen_resolution);
        let aspect_ratio = screen_resolution.0 as f64 / screen_resolution.1 as f64;
        Self {
            reference_frame,
            fov,
            aspect_ratio,
            screen_distance,
            screen,
        }
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
        let angle_x = self.fov / 2.0;
        let distance_x = width / angle_x.to_radians().tan();
        let angle_y = angle_x / self.aspect_ratio;
        let distance_y = height / angle_y.to_radians().tan();

        // Choose the maximum distance to fit the entire point cloud
        let distance = distance_x.max(distance_y);

        // Compute the camera position
        let position = center - Vector3::z() * distance;

        self.reference_frame.position = position;
        self.reference_frame.look_at(center);
    }

    /// Intersect a point in 3D space with the screen and return the pixel coordinates
    pub fn intersect_screen(&self, point: &Point) -> Option<(usize, usize)> {
        // Direction from camera to the point in world space
        let ray_direction = (point.position - self.reference_frame.position).normalize();
        let look_direction = self.reference_frame.look_direction();

        // Make sure the point is in front of the camera
        let alignment = ray_direction.dot(&look_direction);
        if alignment <= 0.0 {
            return None;
        }

        // Recalculate screen dimensions based on the latest FOV and screen distance
        let fov_rad = self.fov.to_radians();
        let screen_width = 2.0 * (fov_rad / 2.0).tan() * self.screen_distance;
        let screen_height = screen_width / self.aspect_ratio;

        // Compute the distance from the camera to the screen along the camera's view direction
        let distance = self.screen_distance / alignment;

        // Compute the intersection point in world space
        let intersection = self.reference_frame.position + ray_direction * distance;

        // Convert the intersection point to screen space coordinates (2D)
        let screen_origin = self.reference_frame.position + look_direction * self.screen_distance;

        // Calculate relative coordinates on the screen
        let relative_position = intersection - screen_origin;
        let normalized_x =
            relative_position.dot(&self.reference_frame.right) / (screen_width / 2.0);
        let normalized_y = relative_position.dot(&self.reference_frame.up) / (screen_height / 2.0);

        // Convert the normalized device coordinates to pixel coordinates
        self.screen.to_pixel_coords(normalized_x, normalized_y)
    }

    /// Intersect a point in 3D space with the screen using depth of field
    pub fn intersect_screen_dof(
        &self,
        point: &Point,
        aperture_size: f64,
        samples: usize,
    ) -> Option<(usize, usize)> {
        let mut rng = rng();
        let mut pixel_sum = (0.0, 0.0);
        let mut valid_samples = 0;

        // Recalculate screen dimensions based on the latest FOV
        let fov_rad = self.fov.to_radians();
        let screen_width = 2.0 * (fov_rad / 2.0).tan() * self.screen_distance;
        let screen_height = screen_width / self.aspect_ratio;

        for _ in 0..samples {
            // Generate random offsets within the aperture size
            let jitter_x: f64 = rng.random_range(-aperture_size..aperture_size);
            let jitter_y: f64 = rng.random_range(-aperture_size..aperture_size);

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
                relative_position.dot(&self.reference_frame.right) / (screen_width / 2.0);
            let normalized_y =
                relative_position.dot(&self.reference_frame.up) / (screen_height / 2.0);

            // Convert to pixel coordinates
            if let Some((px, py)) = self.screen.to_pixel_coords(normalized_x, normalized_y) {
                pixel_sum.0 += px as f64;
                pixel_sum.1 += py as f64;
                valid_samples += 1;
            }
        }

        // Average the results to simulate blur
        if valid_samples > 0 {
            let avg_x = (pixel_sum.0 / valid_samples as f64).round() as usize;
            let avg_y = (pixel_sum.1 / valid_samples as f64).round() as usize;
            Some((avg_x, avg_y))
        } else {
            None
        }
    }
}
