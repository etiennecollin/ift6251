use crate::PixelType;

/// A screen with pixels to render to
pub struct Screen {
    /// The pixel buffer
    pub pixels: Vec<Vec<PixelType>>,
    /// The screen resolution in pixels
    pub resolution: (usize, usize),
}

impl Screen {
    /// Creates a new screen with a given resolution and distance from the camera
    pub fn new(resolution: (usize, usize)) -> Self {
        let pixels = vec![vec![PixelType::from([0, 0, 0, 255]); resolution.0]; resolution.1];

        Self { pixels, resolution }
    }

    /// Computes the physical screen dimensions based on FOV, aspect ratio and distance from the
    /// camera
    pub fn dimensions(&self, fov: f64, aspect_ratio: f64, distance: f64) -> (f64, f64) {
        let fov_rad = fov.to_radians();
        let screen_width = 2.0 * (fov_rad / 2.0).tan() * distance;
        let screen_height = screen_width / aspect_ratio;
        (screen_width, screen_height)
    }

    /// Converts normalized 2D point on the screen (-1 to 1) into pixel coordinates
    pub fn to_pixel_coords(&self, normalized_x: f64, normalized_y: f64) -> Option<(usize, usize)> {
        let (width, height) = self.resolution;

        let pixel_x = ((normalized_x + 1.0) * 0.5 * (width as f64)) as usize;
        let pixel_y = ((1.0 - normalized_y) * 0.5 * (height as f64)) as usize;

        // Ensure coordinates are within bounds
        if pixel_x < width && pixel_y < height {
            Some((pixel_x, pixel_y))
        } else {
            None
        }
    }
}
