use point_cloud_renderer::{
    camera::{Camera, CameraReferenceFrame},
    render::{generate_random_point_cloud, render_image},
};

pub fn main() {
    // Define camera position and orientation
    let reference_frame = CameraReferenceFrame::default();

    // Create the camera
    let mut camera = Camera::new(reference_frame, 120.0, 1.0, (800, 450));

    // Generate a random point cloud
    let range_x = (-100.0, 10.0);
    let range_y = (-100.0, 10.0);
    let range_z = (-100.0, 10.0);
    let points = generate_random_point_cloud(50000, range_x, range_y, range_z);

    // Fit the camera to the point cloud
    camera.fit_points(&points);

    // Render the image
    let image = render_image(&camera, &points);

    // Save the image as a PNG file
    image.save("point_cloud.png").unwrap();
    println!("Image saved as 'point_cloud.png'");
}
