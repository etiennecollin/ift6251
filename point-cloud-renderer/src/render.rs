use image::Pixel;
use rayon::iter::{IntoParallelRefIterator, ParallelIterator};

use crate::{ImageType, camera::Camera, point::Point};

/// Renders the point cloud using the given camera and returns the image.
#[inline]
pub fn render_image(camera: &Camera, points: &[Point]) -> ImageType {
    let width = camera.screen.resolution.0;
    let height = camera.screen.resolution.1;

    // Image and 2D depth buffer
    let mut image = ImageType::new(width as u32, height as u32);
    let mut depth_buffer = vec![vec![f32::INFINITY; width]; height];

    // Compute the intersections and render the points
    // points
    //     .iter()
    //     .filter_map(|point| {
    //         camera
    //             .intersect_screen(point)
    //             // .intersect_screen_dof(point, 0.5, 10)
    //             .map(|intersection| (intersection, point.color))
    //     })
    //     .for_each(|((distance, (px, py)), mut color)| {
    //         // Check if the point is behind another point
    //         if distance < depth_buffer[py][px] {
    //             depth_buffer[py][px] = distance;
    //         } else {
    //             return;
    //         }
    //
    //         let current_color = image.get_pixel(px as u32, py as u32);
    //         color.blend(current_color);
    //         image.put_pixel(px as u32, py as u32, color);
    //     });

    // Parallelize the rendering process
    let collision_list = points
        .par_iter()
        .filter_map(|point| {
            camera
                .intersect_screen(point)
                // .intersect_screen_dof(point, 0.5, 10)
                .map(|intersection| (intersection, point.color))
        })
        .collect_vec_list();

    // Update the image with the collision list
    collision_list.into_iter().for_each(|v| {
        v.into_iter().for_each(|((distance, (px, py)), mut color)| {
            // Check if the point is behind another point
            if distance < depth_buffer[py][px] {
                depth_buffer[py][px] = distance;
            } else {
                return;
            }

            let current_color = image.get_pixel(px as u32, py as u32);
            color.blend(current_color);
            image.put_pixel(px as u32, py as u32, color);
        })
    });

    image
}
