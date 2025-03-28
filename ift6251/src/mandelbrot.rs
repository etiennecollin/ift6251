use std::sync::Mutex;

use ift6251::{
    get_save_path,
    utils::{
        images::{create_texture, equalize, recalibrate},
        mandelbrot::{get_shift_speed, is_in_mandelbrot, shift, zoom},
    },
};
use indicatif::{ProgressBar, ProgressStyle};
use nannou::{
    color::{IntoColor, encoding::Srgb},
    image::{self, ImageBuffer, RgbaImage},
    noise::{NoiseFn, Perlin},
    prelude::*,
};
use nannou_egui::{
    Egui, FrameCtx,
    egui::{self},
};
use rayon::iter::{IntoParallelIterator, ParallelBridge, ParallelIterator};

fn main() {
    nannou::app(model).update(update).run()
}

struct State {
    redraw: bool,
    continuous_redraw: bool,
    image: ImageBuffer<image::Rgba<u8>, Vec<u8>>,
    delta: f64,
    x_range: (f64, f64),
    y_range: (f64, f64),
    zoom_speed: f64,
    shift_speed: u32,
    max_iterations: usize,
    select_in_mandelbrot: bool,
    plot_trajectory: bool,
    noise: Perlin,
    hue_scale: f64,
    saturation: f32,
    noise_scale_x: f64,
    noise_scale_y: f64,
    noise_scale_z: f64,
}

struct Model {
    egui: Egui,
    state: State,
}

fn model(app: &App) -> Model {
    let window_id = app
        .new_window()
        .size(512, 512)
        .view(view)
        .raw_event(raw_window_event)
        .key_pressed(key_pressed)
        .mouse_wheel(mouse_wheel)
        .build()
        .unwrap();

    let window = app.window(window_id).unwrap();
    let (width, height) = window.rect().w_h();
    let state = State {
        redraw: true,
        continuous_redraw: false,
        image: ImageBuffer::new(width as u32, height as u32),
        delta: 0.50,
        x_range: (-2.0, 0.50),
        y_range: (-1.25, 1.25),
        zoom_speed: 0.001,
        shift_speed: 100,
        max_iterations: 100,
        select_in_mandelbrot: false,
        plot_trajectory: false,
        noise: Perlin::new(),
        hue_scale: 0.0,
        noise_scale_x: 1.35,
        noise_scale_y: 0.75,
        noise_scale_z: 1.0,
        saturation: 0.5,
    };

    let egui = Egui::from_window(&window);

    Model { egui, state }
}

fn update_egui(ctx: FrameCtx, state: &mut State, app: &App) {
    // Generate the settings window
    egui::Window::new("Settings")
        .default_width(0.0)
        .show(&ctx, |ui| {
            ui.label("Delta:");
            ui.add(egui::Slider::new(&mut state.delta, 0.05..=1.0));
            // Round delta to be a divisor of 1.0
            state.delta = 1.0 / (1.0 / state.delta).round();

            ui.label("Max iterations:");
            ui.add(egui::Slider::new(&mut state.max_iterations, 10..=10000));

            ui.label("Zoom speed:");
            ui.add(egui::Slider::new(&mut state.zoom_speed, 0.0001..=0.1));

            ui.label("Shift speed:");
            ui.add(egui::Slider::new(&mut state.shift_speed, 10..=100));

            ui.separator();

            ui.label("Hue scale:");
            ui.add(egui::Slider::new(&mut state.hue_scale, 0.0..=1.0));

            ui.label("Saturation:");
            ui.add(egui::Slider::new(&mut state.saturation, 0.0..=1.0));

            ui.label("Noise scale x:");
            ui.add(egui::Slider::new(&mut state.noise_scale_x, 0.50..=1.5));

            ui.label("Noise scale y:");
            ui.add(egui::Slider::new(&mut state.noise_scale_y, 0.00..=0.75));

            ui.label("Noise scale z:");
            ui.add(egui::Slider::new(&mut state.noise_scale_z, 0.00..=1.0));

            ui.separator();

            ui.checkbox(&mut state.select_in_mandelbrot, "Select in Mandelbrot");
            ui.checkbox(&mut state.plot_trajectory, "Plot Trajectory");
            ui.checkbox(&mut state.continuous_redraw, "Continuous Redraw");

            let update = ui.button("Update").clicked();
            if update {
                state.redraw = true;
            }

            let save = ui.button("Save").clicked();
            if save {
                state
                    .image
                    .save(get_save_path(&app.exe_name().unwrap()))
                    .unwrap();
            }
        });
}

fn update(app: &App, model: &mut Model, update: Update) {
    let egui = &mut model.egui;
    let state = &mut model.state;
    let (width, height) = app.window_rect().w_h();

    egui.set_elapsed_time(update.since_start);
    let ctx = egui.begin_frame();
    update_egui(ctx, state, app);

    if state.redraw || state.continuous_redraw {
        let mut mandelbrot_array = compute_mandelbrot_array(width as usize, height as usize, state);
        recalibrate(&mut mandelbrot_array);
        equalize(&mut mandelbrot_array, 0.0);
        let image = to_image(mandelbrot_array, state);
        state.image = image;
        state.redraw = false;
    }
}

fn raw_window_event(_app: &App, model: &mut Model, event: &nannou::winit::event::WindowEvent) {
    // Let egui handle things like keyboard and mouse input.
    model.egui.handle_raw_event(event);
}

fn key_pressed(app: &App, model: &mut Model, key: Key) {
    let state = &mut model.state;
    match key {
        Key::Left => {
            let shift_x = get_shift_speed(state.x_range, state.shift_speed);
            state.x_range = shift(state.x_range, -shift_x);
            state.redraw = true;
        }
        Key::Right => {
            let shift_x = get_shift_speed(state.x_range, state.shift_speed);
            state.x_range = shift(state.x_range, shift_x);
            state.redraw = true;
        }
        Key::Up => {
            let shift_y = get_shift_speed(state.y_range, state.shift_speed);
            state.y_range = shift(state.y_range, -shift_y);
            state.redraw = true;
        }
        Key::Down => {
            let shift_y = get_shift_speed(state.y_range, state.shift_speed);
            state.y_range = shift(state.y_range, shift_y);
            state.redraw = true;
        }
        Key::Plus | Key::Equals => {
            let zoom_factor = 1.0 - 10.0 * state.zoom_speed;
            (state.x_range, state.y_range) = zoom(state.x_range, state.y_range, zoom_factor);
            state.redraw = true;
        }
        Key::Minus => {
            let zoom_factor = 1.0 + 10.0 * state.zoom_speed;
            (state.x_range, state.y_range) = zoom(state.x_range, state.y_range, zoom_factor);
            state.redraw = true;
        }
        Key::Q => app.quit(),
        Key::S => model
            .state
            .image
            .save(get_save_path(&app.exe_name().unwrap()))
            .unwrap(),
        Key::Return => model.state.redraw = true,
        _other_key => {}
    }
}

fn mouse_wheel(_app: &App, model: &mut Model, delta: MouseScrollDelta, _phase: TouchPhase) {
    let state = &mut model.state;

    match delta {
        MouseScrollDelta::LineDelta(_, y) => {
            let zoom_factor = 1.0 + y as f64 * state.zoom_speed;
            (state.x_range, state.y_range) = zoom(state.x_range, state.y_range, zoom_factor);
        }
        MouseScrollDelta::PixelDelta(pos) => {
            let zoom_factor = 1.0 + pos.y * state.zoom_speed;
            (state.x_range, state.y_range) = zoom(state.x_range, state.y_range, zoom_factor);
        }
    }
    model.state.redraw = true;
}

fn view(app: &App, model: &Model, frame: Frame) {
    // Setup the drawing context
    let draw = app.draw();
    let state = &model.state;

    let texture = create_texture(app.main_window(), state.image.clone());
    draw.texture(&texture);

    draw.to_frame(app, &frame).unwrap();
    model.egui.draw_to_frame(&frame).unwrap();
}

fn compute_mandelbrot_array(width: usize, height: usize, state: &State) -> Vec<Vec<f64>> {
    let delta = state.delta;
    let max_iterations = state.max_iterations;
    let select_in_mandelbrot = state.select_in_mandelbrot;
    let plot_trajectory = state.plot_trajectory;

    // Display sub-fractal of mandelbrot set
    let iterations_per_row = (width as f64 / delta) as u64;
    let pb = ProgressBar::new((height as f64 / delta) as u64 * iterations_per_row)
        .with_message("Rendering")
        .with_style(
            ProgressStyle::default_bar()
                .template(
                    "[{elapsed}] [ETA: {eta}] [{wide_bar}] [{percent}%] {human_pos}/{human_len} {msg}",
                )
                .unwrap(),
        );

    // Create a 2D array to store the pixel values
    let array = Mutex::new(vec![vec![0.0; width]; height]);

    // Iterate over the rows of the image
    (0..(height as f64 / delta) as usize)
        .into_par_iter()
        .for_each(|y| {
            // Store the pixel values for the visited pixels
            // This prvents locking the array for each pixel
            let mut pixel_array = Vec::new();

            // Iterate over the columns of the row
            (0..(width as f64 / delta) as usize).for_each(|x| {
                let x = x as f64 * delta;
                let y = y as f64 * delta;

                // Store list of x,y coordinates at each iteration
                let (in_mandelbrot, pixels) = is_in_mandelbrot(
                    x,
                    y,
                    width,
                    height,
                    state.x_range,
                    state.y_range,
                    max_iterations,
                );

                // Skip the pixel or not
                if in_mandelbrot.is_none() == select_in_mandelbrot {
                    if plot_trajectory {
                        // Increment the pixel value for the visited pixels
                        pixel_array.append(
                            &mut pixels
                                .into_iter()
                                .map(|(x, y)| (x, y, in_mandelbrot.unwrap_or(1)))
                                .collect(),
                        );
                    } else {
                        pixel_array.push((
                            x as usize,
                            y as usize,
                            in_mandelbrot.unwrap_or(max_iterations),
                        ));
                    }
                }
            });

            // Increment the pixel value for the visited pixels
            let mut array_lock = array.lock().unwrap();
            pixel_array.into_iter().for_each(|(x, y, v)| {
                array_lock[y][x] += v as f64;
            });

            // Update the progress bar
            pb.inc(iterations_per_row);
        });

    // Finish the progress bar
    pb.finish_with_message("Rendered");

    // Return the array
    let array_lock = array.lock().unwrap();
    array_lock.clone()
}

fn to_image(array: Vec<Vec<f64>>, state: &mut State) -> ImageBuffer<image::Rgba<u8>, Vec<u8>> {
    let width = array[0].len() as u32;
    let height = array.len() as u32;
    let height_half = height as f64 / 2.0;
    let noise = &mut state.noise;

    let mut image: RgbaImage = RgbaImage::new(width, height);
    image
        .enumerate_pixels_mut()
        .par_bridge()
        .for_each(|(x, y, pixel)| {
            let symmetry_y = (y as f64 / height_half - 1.0).abs();

            let lightness = array[y as usize][x as usize] / 255.0;
            let hue = (lightness * state.hue_scale
                + noise.get([
                    lightness * state.noise_scale_z,
                    x as f64 / width as f64 * state.noise_scale_x,
                    symmetry_y * state.noise_scale_y,
                ])) as f32;

            let (r, g, b) = hsl(hue, state.saturation, lightness as f32)
                .into_rgb::<Srgb>()
                .into_format::<u8>()
                .into_components();

            *pixel = image::Rgba([r, g, b, 255])
        });
    image
}
