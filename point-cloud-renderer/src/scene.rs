use crate::{camera::Camera, point::Point};

/// A scene with a list of points and a camera
pub struct Scene {
    /// The list of points in the scene
    pub points: Vec<Point>,
    /// The camera of the scene
    pub camera: Camera,
}

impl Scene {
    /// Create a new scene with a list of points and a camera
    pub fn new(points: Vec<Point>, camera: Camera) -> Self {
        Self { points, camera }
    }
}
