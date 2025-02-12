use nalgebra::Point3;

use crate::PixelType;

#[derive(Clone)]
pub struct Point {
    pub position: Point3<f64>,
    pub color: PixelType,
}

impl Point {
    pub fn new(position: Point3<f64>, color: PixelType) -> Self {
        Self { position, color }
    }

    pub fn bounding_box(points: &[Self]) -> (Point3<f64>, Point3<f64>) {
        let mut min_x = f64::INFINITY;
        let mut max_x = f64::NEG_INFINITY;
        let mut min_y = f64::INFINITY;
        let mut max_y = f64::NEG_INFINITY;
        let mut min_z = f64::INFINITY;
        let mut max_z = f64::NEG_INFINITY;

        // Find the bounding box of the point cloud
        points.iter().for_each(|point| {
            let x = point.position.x;
            let y = point.position.y;
            let z = point.position.z;

            min_x = min_x.min(x);
            max_x = max_x.max(x);
            min_y = min_y.min(y);
            max_y = max_y.max(y);
            min_z = min_z.min(z);
            max_z = max_z.max(z);
        });

        (
            Point3::new(min_x, min_y, min_z),
            Point3::new(max_x, max_y, max_z),
        )
    }
}

impl Default for Point {
    fn default() -> Self {
        Self {
            position: Point3::new(0.0, 0.0, 0.0),
            color: PixelType::from([0, 0, 0, 255]),
        }
    }
}
