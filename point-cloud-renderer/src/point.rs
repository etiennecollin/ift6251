use nannou::wgpu;

#[repr(C)]
#[derive(Clone, Copy)]
pub struct Point {
    pub position: [f32; 3],
    pub color: [f32; 4],
}

impl Point {
    /// The vertex format for a point.
    pub const ATTRIBS: [wgpu::VertexAttribute; 2] =
        wgpu::vertex_attr_array![0 => Float32x3, 1 => Float32x4];

    /// Create a new point with a position and color.
    ///
    /// The color is in the range [0, 255].
    pub fn new(position: [f32; 3], color: [u8; 4]) -> Self {
        let color = color.map(|c| c as f32 / 255.0);
        Self { position, color }
    }

    /// Computes the bounding box of a point cloud.
    pub fn bounding_box(points: &[Self]) -> ([f32; 3], [f32; 3]) {
        let mut min_x = f32::INFINITY;
        let mut max_x = f32::NEG_INFINITY;
        let mut min_y = f32::INFINITY;
        let mut max_y = f32::NEG_INFINITY;
        let mut min_z = f32::INFINITY;
        let mut max_z = f32::NEG_INFINITY;

        // Find the bounding box of the point cloud
        points.iter().for_each(|point| {
            let x = point.position[0];
            let y = point.position[1];
            let z = point.position[2];

            min_x = min_x.min(x);
            max_x = max_x.max(x);
            min_y = min_y.min(y);
            max_y = max_y.max(y);
            min_z = min_z.min(z);
            max_z = max_z.max(z);
        });

        ([min_x, min_y, min_z], [max_x, max_y, max_z])
    }

    /// Set the position of the point.
    pub fn set_position(&mut self, position: [f32; 3]) {
        self.position = position;
    }

    /// Set the color of the point.
    ///
    /// The color is in the range [0, 255].
    pub fn set_color(&mut self, color: [u8; 4]) {
        self.color = color.map(|c| c as f32 / 255.0);
    }

    /// Set the color of the point.
    ///
    /// The color is in the range [0, 1].
    pub fn set_color_f32(&mut self, color: [f32; 4]) {
        self.color = color;
    }

    /// Convert a slice of points to a byte slice.
    pub fn as_bytes(points: &[Point]) -> &[u8] {
        unsafe { wgpu::bytes::from_slice(points) }
    }
}

impl Default for Point {
    fn default() -> Self {
        Self {
            position: [0.0, 0.0, 0.0],
            color: [0.0, 0.0, 0.0, 1.0],
        }
    }
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct CloudData {
    pub sound_amplitude: f32,
    pub wind_strength: f32,
    pub noise_scale: f32,
    pub spring_constant: f32,
}

impl CloudData {
    /// Returns the struct as a byte slice.
    pub fn as_bytes(&self) -> &[u8] {
        unsafe { wgpu::bytes::from(self) }
    }
}
