use e57::{CartesianCoordinate, E57Reader};
use rand::Rng;
use rayon::iter::{ParallelBridge, ParallelIterator};

use crate::point::Point;

/// Generates a random point cloud with the given number of points.
pub fn generate_random_point_cloud(
    num_points: usize,
    range_x: (f32, f32),
    range_y: (f32, f32),
    range_z: (f32, f32),
) -> Vec<Point> {
    let mut rng = rand::rng();
    let mut points = Vec::with_capacity(num_points);

    (0..num_points).for_each(|_| {
        let position = [
            rng.random_range(range_x.0..range_x.1),
            rng.random_range(range_y.0..range_y.1),
            rng.random_range(range_z.0..range_z.1),
        ];

        let color = [
            rng.random_range(0..=255),
            rng.random_range(0..=255),
            rng.random_range(0..=255),
            255,
        ];

        points.push(Point::new(position, color));
    });

    points
}

/// Reads a point cloud from an E57 file and returns the points.
pub fn read_e57(path: &str) -> Result<Vec<Point>, &'static str> {
    // Open E57 input file for reading
    let mut file = match E57Reader::from_file(path) {
        Ok(file) => file,
        Err(_) => return Err("Failed to open E57 file"),
    };

    let mut points = Vec::new();

    // Loop over all point clouds in the E57 file
    for pointcloud in file.pointclouds().into_iter() {
        let mut iter = match file.pointcloud_simple(&pointcloud) {
            Ok(iter) => iter,
            Err(_) => return Err("Failed to read point cloud"),
        };

        // Set point iterator options
        iter.spherical_to_cartesian(true);
        iter.cartesian_to_spherical(false);
        iter.intensity_to_color(true);
        iter.apply_pose(true);

        // Iterate over all points in point cloud
        let mut cloud_points: Vec<Point> = iter
            .par_bridge()
            .filter_map(|p| {
                let p = match p {
                    Ok(p) => p,
                    Err(_) => return None,
                };

                let mut point = Point::default();

                // Write XYZ data to output file
                // We use the Z-up coordinate system,
                // so we swap the Y and Z coordinates
                if let CartesianCoordinate::Valid { x, y, z } = p.cartesian {
                    point.position[0] = -x as f32;
                    point.position[1] = z as f32;
                    point.position[2] = y as f32;
                } else {
                    return None;
                }

                // If available, write RGB color or intensity color values
                if let Some(color) = p.color {
                    point.set_color_f32([color.red, color.green, color.blue, 1.0]);
                }

                Some(point)
            })
            .collect();

        points.append(&mut cloud_points);
    }

    Ok(points)
}
