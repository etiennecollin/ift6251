use std::{
    cell::Ref,
    sync::{Arc, Mutex},
};

use fastnoise_lite::{FastNoiseLite, NoiseType};
use nannou::{
    image::{self, ImageBuffer},
    prelude::*,
};
use nannou_audio::{Buffer, Host, Stream};
use nannou_egui::{
    Egui, FrameCtx,
    egui::{self},
};
use point_cloud_renderer::{
    camera::{Camera, CameraReferenceFrame, Direction},
    loader::{generate_random_point_cloud, read_e57},
    point::Point,
    render::render_image,
};
use rayon::iter::{
    IndexedParallelIterator, IntoParallelRefIterator, IntoParallelRefMutIterator, ParallelIterator,
};
use spectrum_analyzer::{FrequencyLimit, samples_fft_to_spectrum, windows::hann_window};
use wgpu::WithDeviceQueuePair;

const AUDIO_PATH: &str = "./data/audio.wav";

fn main() {
    nannou::app(model).update(update).run()
}

struct State {
    camera: Camera,
    file_path: String,
    initial_points: Vec<Point>,
    points: Vec<Point>,
    image: ImageBuffer<image::Rgba<u8>, Vec<u8>>,
    movement_speed: f32,
    noise: FastNoiseLite,
    noise_scale: f32,
    wind_strength: f32,
    spring_constant: f32,
    // This will be accessed by the audio thread.
    _volume: Arc<Mutex<f32>>,
    fft_output: Arc<Mutex<f32>>,
}

struct Audio {
    sounds: Vec<audrey::read::BufFileReader>,
    volume: Arc<Mutex<f32>>,
    fft_output: Arc<Mutex<f32>>,
}

struct Model {
    egui: Egui,
    state: State,
    _stream: Stream<Audio>,
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

    // Initialise the state that we want to live on the audio thread.
    let audio_host = Host::new();
    let volume = Arc::new(Mutex::new(0.0));
    let fft_output = Arc::new(Mutex::new(0.0));
    let audio_model = Audio {
        sounds: vec![],
        fft_output: Arc::clone(&fft_output),
        volume: Arc::clone(&volume),
    };

    // Create audio stream
    let stream = audio_host
        .new_output_stream(audio_model)
        .render(audio)
        .build()
        .unwrap();
    let sound = audrey::open(AUDIO_PATH).expect("Failed to load sound");
    stream
        .send(move |audio| {
            audio.sounds.push(sound);
        })
        .ok();
    stream.play().unwrap();

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

    // Create noise
    let mut noise = FastNoiseLite::new();
    noise.set_noise_type(Some(NoiseType::Perlin));

    let state = State {
        camera,
        file_path: "./data/union_station.e57".to_owned(),
        initial_points: points.clone(),
        points,
        image: ImageBuffer::new(width as u32, height as u32),
        movement_speed: 5.0,
        noise,
        noise_scale: 0.0,
        wind_strength: 0.2,
        spring_constant: 0.002,
        // This will be accessed by the audio thread.
        _volume: volume,
        fft_output,
    };

    // Create the GUI
    let egui = Egui::from_window(&window);

    Model {
        egui,
        state,
        _stream: stream,
    }
}

fn audio(audio: &mut Audio, buffer: &mut Buffer) {
    let mut have_ended = vec![];
    let len_frames = buffer.len_frames();
    let mut rms_volume = 0.0;

    // Sum all of the sounds onto the buffer.
    for (i, sound) in audio.sounds.iter_mut().enumerate() {
        let mut frame_count = 0;
        let file_frames = sound.frames::<[f32; 2]>().filter_map(Result::ok);
        for (frame, file_frame) in buffer.frames_mut().zip(file_frames) {
            let mut frame_rms = 0.0; // Compute the root mean square of the frame
            for (sample, file_sample) in frame.iter_mut().zip(&file_frame) {
                *sample += *file_sample;
                frame_rms += file_sample.powi(2);
            }
            rms_volume += (frame_rms / 2.0).sqrt();
            frame_count += 1;
        }

        // If the sound yielded less samples than are in the buffer, it must have ended.
        if frame_count < len_frames {
            have_ended.push(i);
        }
    }

    // Remove all sounds that have ended.
    for i in have_ended.into_iter().rev() {
        audio.sounds.remove(i);
    }

    // Normalize the volume
    let volume = rms_volume / len_frames as f32 * 100.0;
    // Update the volume value
    *audio.volume.lock().unwrap() = volume;

    // Merge the audio channels
    let fft_input: Vec<f32> = buffer.frames().flatten().cloned().collect();

    // Apply hann window for smoothing; length must be a power of 2 for the FFT
    let hann_window = hann_window(&fft_input);

    // Compute the FFT and get the spectrum
    let spectrum = samples_fft_to_spectrum(
        &hann_window,
        buffer.sample_rate(),
        FrequencyLimit::All,
        None,
    )
    .ok();

    // Compute the sum of the magnitudes
    let magnitude = match spectrum {
        Some(s) => s.data().par_iter().map(|f| f.1.val()).sum::<f32>().max(1.0),
        None => 1.0,
    };

    // Update the audio strength value
    {
        *audio.fft_output.lock().unwrap() = magnitude;
    }
}

fn update(_app: &App, model: &mut Model, update: Update) {
    let egui = &mut model.egui;
    let state = &mut model.state;
    let time = update.since_start.secs();

    // Update GUI
    egui.set_elapsed_time(update.since_start);
    let ctx = egui.begin_frame();
    update_egui(ctx, state);

    // Get the audio strength
    let audio_strength = *state.fft_output.lock().unwrap();
    flow(state, time as f32, audio_strength);

    // Render the image
    let image = render_image(&state.camera, &state.points);
    let resolution = state.camera.screen.resolution;
    let image = nannou::image::ImageBuffer::from_raw(
        resolution.0 as u32,
        resolution.1 as u32,
        image.into_raw(),
    )
    .unwrap();

    // Update the image in the state
    state.image = image;
}

fn flow(state: &mut State, time: f32, audio_strength: f32) {
    let noise = &mut state.noise;
    let points = &mut state.points;
    let initial_points = &state.initial_points; // Use the initial positions
    let scale = state.noise_scale;
    let scaled_time = time * scale;
    let wind_strength = state.wind_strength * audio_strength;
    let spring_constant = state.spring_constant;

    if wind_strength == 0.0 && spring_constant == 0.0 {
        return;
    }

    points.par_iter_mut().enumerate().for_each(|(i, point)| {
        let initial_position = initial_points[i].position; // Get the original position of the point
        let x = point.position.x * scale;
        let y = point.position.y * scale;
        let z = point.position.z * scale;

        // Simulate wind-like vector field using noise or another function
        let wind_x = noise.get_noise_3d(x, y, scaled_time);
        let wind_y = noise.get_noise_3d(y, z, scaled_time);
        let wind_z = noise.get_noise_3d(z, x, scaled_time);

        // Apply wind force to the point's position
        point.position.x += wind_x * wind_strength;
        point.position.y += wind_y * wind_strength;
        point.position.z += wind_z * wind_strength;

        // Calculate the distance from the original position
        let displacement = point.position - initial_position;

        // Apply the spring-like restorative force
        point.position -= spring_constant * displacement;
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

pub fn create_texture(
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
            ui.add(egui::Slider::new(&mut state.movement_speed, 0.01..=50.0));

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
