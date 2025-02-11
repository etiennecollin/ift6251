use image::Pixel;
use nalgebra::Point3;
use rand::Rng;
use rayon::iter::{IntoParallelRefIterator, ParallelIterator};

use crate::{camera::Camera, point::Point, ImageType, PixelType};

pub fn generate_random_point_cloud(
    num_points: usize,
    range_x: (f64, f64),
    range_y: (f64, f64),
    range_z: (f64, f64),
) -> Vec<Point> {
    let mut rng = rand::rng();
    let mut points = Vec::with_capacity(num_points);

    (0..num_points).for_each(|_| {
        let x = rng.random_range(range_x.0..range_x.1);
        let y = rng.random_range(range_y.0..range_y.1);
        let z = rng.random_range(range_z.0..range_z.1);

        let color = PixelType::from([
            rng.random_range(0..255),
            rng.random_range(0..255),
            rng.random_range(0..255),
            255,
        ]);

        points.push(Point::new(Point3::new(x, y, z), color));
    });

    points
}

pub fn render_image(camera: &Camera, points: &[Point]) -> ImageType {
    let mut image = ImageType::new(
        camera.screen.resolution.0 as u32,
        camera.screen.resolution.1 as u32,
    );

    let collision_list: Vec<_> = points
        .par_iter()
        .filter_map(|point| {
            camera
                .intersect_screen(point)
                // .intersect_screen_dof(point, 0.5, 10)
                .map(|position| (position, point.color))
        })
        .collect();

    collision_list
        .into_iter()
        .for_each(|((px, py), mut color)| {
            let current_color = image.get_pixel(px as u32, py as u32);
            color.blend(current_color);
            image.put_pixel(px as u32, py as u32, color);
        });

    image
}
