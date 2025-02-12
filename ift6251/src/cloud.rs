use std::cell::Ref;

use nannou::{
    image::{self, ImageBuffer},
    noise::{NoiseFn, Perlin},
    prelude::*,
};
use nannou_egui::{
    egui::{self},
    Egui, FrameCtx,
};
use point_cloud_renderer::{
    camera::{Camera, CameraReferenceFrame, Direction},
    point::Point,
    render::{generate_random_point_cloud, read_e57, render_image},
};
use rayon::iter::{IndexedParallelIterator, IntoParallelRefMutIterator, ParallelIterator};
use wgpu::WithDeviceQueuePair;

fn main() {
    nannou::app(model).update(update).run()
}

struct State {
    camera: Camera,
    file_path: String,
    initial_points: Vec<Point>,
    points: Vec<Point>,
    image: ImageBuffer<image::Rgba<u8>, Vec<u8>>,
    movement_speed: f64,
    noise: Perlin,
    noise_scale: f64,
    wind_strength: f64,
    spring_constant: f64,
}

struct Model {
    egui: Egui,
    state: State,
}

fn random_points() -> Vec<Point> {
    let range_x = (-100.0, 100.0);
    let range_y = (-100.0, 100.0);
    let range_z = (-100.0, 100.0);
    generate_random_point_cloud(500000, range_x, range_y, range_z)
}

fn model(app: &App) -> Model {
    let window_id = app
        .new_window()
        .fullscreen()
        .view(view)
        .raw_event(raw_window_event)
        .key_pressed(key_pressed)
        .build()
        .unwrap();

    let window = app.window(window_id).unwrap();
    let (width, height) = window.rect().w_h();

    // Define camera position and orientation
    let reference_frame = CameraReferenceFrame::default();

    // Create the camera
    let mut camera = Camera::new(
        reference_frame,
        120.0,
        1.0,
        (width as usize, height as usize),
    );

    // Generate a random point cloud
    let points = random_points();
    camera.fit_points(&points);

    let state = State {
        camera,
        file_path: "./data/union_station.e57".to_owned(),
        initial_points: points.clone(),
        points,
        image: ImageBuffer::new(width as u32, height as u32),
        movement_speed: 5.0,
        noise: Perlin::new(),
        noise_scale: 0.0,
        wind_strength: 0.2,
        spring_constant: 0.002,
    };

    let egui = Egui::from_window(&window);

    Model { egui, state }
}

fn update(_app: &App, model: &mut Model, update: Update) {
    let egui = &mut model.egui;
    let state = &mut model.state;
    let time = update.since_start.secs();

    egui.set_elapsed_time(update.since_start);
    let ctx = egui.begin_frame();
    update_egui(ctx, state);

    flow(state, time);
    let image = render_image(&state.camera, &state.points);
    let resolution = state.camera.screen.resolution;
    let image = nannou::image::ImageBuffer::from_raw(
        resolution.0 as u32,
        resolution.1 as u32,
        image.into_raw(),
    )
    .unwrap();
    state.image = image;
}

fn flow(state: &mut State, time: f64) {
    let noise = &mut state.noise;
    let points = &mut state.points;
    let initial_points = &state.initial_points; // Use the initial positions
    let scale = state.noise_scale;
    let wind_strength = state.wind_strength;
    let spring_constant = state.spring_constant;

    points.par_iter_mut().enumerate().for_each(|(i, point)| {
        let initial_position = &initial_points[i].position; // Get the original position of the point
        let x = point.position.x;
        let y = point.position.y;
        let z = point.position.z;

        // Simulate wind-like vector field using noise or another function
        let wind_x = noise.get([x * scale, y * scale, z * scale, time * scale]);
        let wind_y = noise.get([y * scale, z * scale, x * scale, time * scale]);
        let wind_z = noise.get([z * scale, x * scale, y * scale, time * scale]);

        // Apply wind force to the point's position
        point.position.x += wind_strength * wind_x;
        point.position.y += wind_strength * wind_y;
        point.position.z += wind_strength * wind_z;

        // Calculate the distance from the original position
        let displacement_x = point.position.x - initial_position.x;
        let displacement_y = point.position.y - initial_position.y;
        let displacement_z = point.position.z - initial_position.z;

        // Apply the spring-like restorative force
        point.position.x -= spring_constant * displacement_x;
        point.position.y -= spring_constant * displacement_y;
        point.position.z -= spring_constant * displacement_z;
    });
}

fn view(app: &App, model: &Model, frame: Frame) {
    // Setup the drawing context
    let draw = app.draw();
    let state = &model.state;

    frame.clear(BLACK);

    let texture = create_texture(app.main_window(), state.image.clone());
    draw.texture(&texture);

    draw.to_frame(app, &frame).unwrap();
    model.egui.draw_to_frame(&frame).unwrap();
}

fn create_texture(
    window: Ref<'_, Window>,
    image: ImageBuffer<image::Rgba<u8>, Vec<u8>>,
) -> wgpu::Texture {
    let usage = nannou::wgpu::TextureUsages::COPY_SRC
        | nannou::wgpu::TextureUsages::COPY_DST
        | nannou::wgpu::TextureUsages::RENDER_ATTACHMENT
        | nannou::wgpu::TextureUsages::TEXTURE_BINDING;

    window.with_device_queue_pair(|device, queue| {
        wgpu::Texture::load_from_image_buffer(device, queue, usage, &image)
    })
}

fn update_egui(ctx: FrameCtx, state: &mut State) {
    let camera = &mut state.camera;
    // Generate the settings window
    egui::Window::new("Settings")
        .default_width(0.0)
        .show(&ctx, |ui| {
            ui.label("fov:");
            ui.add(egui::Slider::new(&mut camera.fov, 1.0..=180.0));

            ui.label("screen_distance:");
            ui.add(egui::Slider::new(&mut camera.screen_distance, 1.0..=10.0));

            ui.label("noise_scale:");
            ui.add(egui::Slider::new(&mut state.noise_scale, 0.0..=0.1));

            ui.label("wind_strength:");
            ui.add(egui::Slider::new(&mut state.wind_strength, 0.0..=0.5));

            ui.label("spring_constant:");
            ui.add(egui::Slider::new(&mut state.spring_constant, 0.0..=0.5));

            ui.label("movement_speed:");
            ui.add(egui::Slider::new(&mut state.movement_speed, 1.0..=50.0));

            ui.label("E57 path:");
            ui.text_edit_singleline(&mut state.file_path);

            let update = ui.button("Load file").clicked();
            if update {
                // Get the points from the E57 file if possible
                let points = if state.file_path.is_empty() {
                    random_points()
                } else {
                    match read_e57(&state.file_path) {
                        Ok(points) => points,
                        Err(_) => random_points(),
                    }
                };

                // Update the camera and points
                camera.fit_points(&points);
                state.initial_points = points.clone();
                state.points = points;
            }
        });
}

fn raw_window_event(_app: &App, model: &mut Model, event: &nannou::winit::event::WindowEvent) {
    // Let egui handle things like keyboard and mouse input.
    model.egui.handle_raw_event(event);
}

fn key_pressed(app: &App, model: &mut Model, key: Key) {
    let state = &mut model.state;
    let distance = state.movement_speed;
    match key {
        Key::X => app.quit(),
        Key::Up => {
            state
                .camera
                .reference_frame
                .move_position(distance, Direction::Forward);
        }
        Key::Down => {
            state
                .camera
                .reference_frame
                .move_position(distance, Direction::Backward);
        }
        Key::Left => {
            state
                .camera
                .reference_frame
                .move_position(distance, Direction::Left);
        }
        Key::Right => {
            state
                .camera
                .reference_frame
                .move_position(distance, Direction::Right);
        }
        Key::Period => {
            state
                .camera
                .reference_frame
                .move_position(distance, Direction::Up);
        }
        Key::Comma => {
            state
                .camera
                .reference_frame
                .move_position(distance, Direction::Down);
        }

        Key::W => {
            state
                .camera
                .reference_frame
                .move_target(distance, Direction::Forward);
        }
        Key::R => {
            state
                .camera
                .reference_frame
                .move_target(distance, Direction::Backward);
        }
        Key::A => {
            state
                .camera
                .reference_frame
                .move_target(distance, Direction::Left);
        }
        Key::S => {
            state
                .camera
                .reference_frame
                .move_target(distance, Direction::Right);
        }
        Key::F => {
            state
                .camera
                .reference_frame
                .move_target(distance, Direction::Up);
        }
        Key::Q => {
            state
                .camera
                .reference_frame
                .move_target(distance, Direction::Down);
        }
        _other_key => {}
    }
}
