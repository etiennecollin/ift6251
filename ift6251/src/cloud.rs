use std::cell::Ref;

use nannou::{
    image::{self, ImageBuffer},
    prelude::*,
};
use nannou_egui::{
    egui::{self},
    Egui, FrameCtx,
};
use point_cloud_renderer::{
    camera::{Camera, CameraReferenceFrame},
    point::Point,
    render::{generate_random_point_cloud, render_image},
};
use wgpu::WithDeviceQueuePair;

fn main() {
    nannou::app(model).update(update).run()
}

struct State {
    camera: Camera,
    points: Vec<Point>,
    image: ImageBuffer<image::Rgba<u8>, Vec<u8>>,
    movement_speed: f64,
}

struct Model {
    egui: Egui,
    state: State,
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
    let range_x = (-100.0, 100.0);
    let range_y = (-100.0, 100.0);
    let range_z = (-100.0, 100.0);
    let points = generate_random_point_cloud(500000, range_x, range_y, range_z);
    camera.fit_points(&points);

    let state = State {
        camera,
        points,
        image: ImageBuffer::new(width as u32, height as u32),
        movement_speed: 1.0,
    };

    let egui = Egui::from_window(&window);

    Model { egui, state }
}

fn update_egui(ctx: FrameCtx, state: &mut State) {
    let camera = &mut state.camera;
    // Generate the settings window
    egui::Window::new("Settings")
        .default_width(0.0)
        .show(&ctx, |ui| {
            ui.label("x:");
            let mut x = camera.reference_frame.position.x;
            ui.add(egui::Slider::new(&mut x, -150.0..=150.0));
            camera.reference_frame.update_position_x(x);

            ui.label("y:");
            let mut y = camera.reference_frame.position.y;
            ui.add(egui::Slider::new(&mut y, -150.0..=150.0));
            camera.reference_frame.update_position_y(y);

            ui.label("z:");
            let mut z = camera.reference_frame.position.z;
            ui.add(egui::Slider::new(&mut z, -150.0..=150.0));
            camera.reference_frame.update_position_z(z);

            ui.separator();

            ui.label("fov:");
            ui.add(egui::Slider::new(&mut camera.fov, 0.0..=180.0));

            ui.label("screen_distance:");
            ui.add(egui::Slider::new(&mut camera.screen_distance, 0.0..=10.0));

            ui.separator();

            ui.label("movement_speed:");
            ui.add(egui::Slider::new(&mut state.movement_speed, 0.0..=25.0));
        });
}

fn update(_app: &App, model: &mut Model, update: Update) {
    let egui = &mut model.egui;
    let state = &mut model.state;

    egui.set_elapsed_time(update.since_start);
    let ctx = egui.begin_frame();
    update_egui(ctx, state);

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

fn raw_window_event(_app: &App, model: &mut Model, event: &nannou::winit::event::WindowEvent) {
    // Let egui handle things like keyboard and mouse input.
    model.egui.handle_raw_event(event);
}

fn key_pressed(app: &App, model: &mut Model, key: Key) {
    let state = &mut model.state;
    match key {
        Key::Q => app.quit(),
        Key::Up => {
            let current_z = state.camera.reference_frame.position.z;
            state
                .camera
                .reference_frame
                .update_position_z(current_z + state.movement_speed);
        }
        Key::Down => {
            let current_z = state.camera.reference_frame.position.z;
            state
                .camera
                .reference_frame
                .update_position_z(current_z - state.movement_speed);
        }
        Key::Left => {
            let current_x = state.camera.reference_frame.position.x;
            state
                .camera
                .reference_frame
                .update_position_x(current_x - state.movement_speed);
        }
        Key::Right => {
            let current_x = state.camera.reference_frame.position.x;
            state
                .camera
                .reference_frame
                .update_position_x(current_x + state.movement_speed);
        }
        Key::Period => {
            let current_y = state.camera.reference_frame.position.y;
            state
                .camera
                .reference_frame
                .update_position_y(current_y + state.movement_speed);
        }
        Key::Comma => {
            let current_y = state.camera.reference_frame.position.y;
            state
                .camera
                .reference_frame
                .update_position_y(current_y - state.movement_speed);
        }
        _other_key => {}
    }
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
